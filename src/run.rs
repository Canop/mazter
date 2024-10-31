use {
    crate::*,
    crokey::*,
    std::{
        io::Write,
        time::Duration,
    },
    termimad::{
        EventSource,
        EventSourceOptions,
        Ticker,
        crossbeam::channel::select,
        crossterm::event::Event,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tick {
    PlayerMoveAuto,
    Continue,
}

/// Run the game, assuming the terminal is already in alternate mode
pub fn run<W: Write>(
    w: &mut W,
    skin: &Skin,
    args: &Args,
) -> anyhow::Result<()> {
    let dim = Dim::terminal()?;
    debug!("terminal size: {dim:?}");
    let mut renderer = Renderer {
        display: Display::Alternate(dim),
        skin,
    };
    let user = if args.screen_saver {
        "screen-saver"
    } else {
        let user = args.user.as_str().trim();
        if user.is_empty() || user == "screen-saver" {
            anyhow::bail!("Invalid user name");
        }
        user
    };
    let mut levels_won = 0;
    let mut level = if let Some(level) = args.level {
        if Database::can_play(user, level)? {
            level
        } else {
            anyhow::bail!(
                "User {:?} must win the previous levels before trying level {}",
                user,
                level
            )
        }
    } else if args.screen_saver {
        // by default, the screen saver starts at level 1
        1
    } else {
        // normal users
        Database::first_not_won(user)?
    };

    let mut ticker = Ticker::new();
    let event_source = EventSource::with_options(EventSourceOptions {
        combine_keys: false,
        ..Default::default()
    })?;
    let user_events = event_source.receiver();

    loop {
        let specs = Specs::for_level(level);
        debug!("maze specs: {:#?}", &specs);
        let mut maze: Maze = time!(specs.into());
        let mut screen_saver_beam = if args.screen_saver {
            // requesting periodic automatic player moves
            Some(ticker.tick_infinitely(Tick::PlayerMoveAuto, Duration::from_millis(140)))
        } else {
            None
        };
        let mut events = EventList::default();
        while !(maze.is_won() || maze.is_lost()) {
            renderer.write(w, &maze)?;
            w.flush()?;
            select! {
                recv(user_events) -> user_event => {
                    match user_event?.event {
                        Event::Key(key_event) => match key_event.into() {
                            key!(q) | key!(ctrl-c) | key!(ctrl-q) => {
                                return Ok(());
                            }
                            key!(up) => maze.try_move(Dir::Up, &mut events),
                            key!(right) => maze.try_move(Dir::Right, &mut events),
                            key!(down) => maze.try_move(Dir::Down, &mut events),
                            key!(left) => maze.try_move(Dir::Left, &mut events),
                            key!(w) => maze.end_player_turn(&mut events),
                            key!(a) => maze.give_up(),
                            _ => {}
                        },
                        Event::Resize(w, h) => {
                            renderer.display = Display::Alternate(Dim::new(w as usize, h as usize));
                        }
                        _ => {}
                    }
                    event_source.unblock(false);
                }
                recv(ticker.tick_receiver) -> tick => {
                    if tick? == Tick::PlayerMoveAuto {
                        maze.move_player_auto(&mut events);
                    }
                }
            }
            if !events.is_empty() {
                renderer.animate_events(w, &maze, &events)?;
                events.clear();
            }
        }
        if let Some(beam) = screen_saver_beam.take() {
            ticker.stop_beam(beam);
        }
        if maze.is_won() {
            levels_won += 1;
            if let Some(levels) = args.levels {
                if levels_won >= levels {
                    return Ok(());
                }
            }
            let next_not_won_level = Database::advance(Achievement::new(user, level))?;
            level = if args.screen_saver {
                level + 1
            } else {
                next_not_won_level
            };
        } else {
            maze.highlight_path_to_exit(maze.start());
        }
        // waiting while the user is displayed that he won or lost
        renderer.write(w, &maze)?;
        w.flush()?;
        if args.screen_saver {
            if maze.is_won() {
                continue; // no need to wait
            }
            ticker.tick_once(Tick::Continue, Duration::from_secs(2));
        }
        loop {
            select! {
                recv(ticker.tick_receiver) -> tick => {
                    if tick? == Tick::Continue {
                        break;
                    }
                }
                recv(user_events) -> user_event => {
                    match user_event?.event {
                        Event::Key(key_event) => match key_event.into() {
                            key!(ctrl - c) | key!(ctrl - q) => {
                                return Ok(());
                            }
                            _ => {
                            }
                        }
                        Event::Resize(w, h) => {
                            renderer.display = Display::Alternate(Dim::new(w as usize, h as usize));
                        }
                        _ => {}
                    }
                    event_source.unblock(false);
                    break;
                }
            }
        }
    }
}
