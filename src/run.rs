use {
    crate::*,
    crokey::*,
    std::{
        io::Write,
        time::Duration,
    },
    termimad::{
        crossbeam::channel::select,
        crossterm::event::{
            Event,
        },
        EventSource,
        EventSourceOptions,
        Ticker,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tick {
    PlayerMoveAuto,
    Continue,
}

/// Run the game, assuming the terminal is already in alternate mode
pub fn run<W: Write>(w: &mut W, skin: &Skin, args: &Args) -> anyhow::Result<()> {
    let dim = Dim::terminal()?;
    debug!("terminal size: {dim:?}");
    let mut renderer = Renderer {
        display: Display::Alternate(dim),
        skin,
    };
    let mut ticker = Ticker::new();


    let (user, mut level) = if args.screen_saver {
        // in the future allow to choose the level (if already won by "screen-saver")
        ("screen-saver", 1)
    } else {
        let user = args.user.as_str();
        let level = if let Some(level) = args.level {
            if Database::can_play(user, level)? {
                level
            } else {
                anyhow::bail!(
                    "User {:?} must win the previous levels before trying level {}",
                    user,
                    level
                )
            }
        } else {
            Database::first_not_won(user)?
        };
        (user, level)
    };

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
            Some(ticker.tick_infinitely(Tick::PlayerMoveAuto, Duration::from_millis(140)))
        } else {
            None
        };

        while !(maze.is_won() || maze.is_lost()) {
            renderer.write(w, &maze)?;
            w.flush()?;
            select! {
                recv(ticker.tick_receiver) -> tick => {
                    if tick? == Tick::PlayerMoveAuto {
                        maze.move_player_auto();
                    }
                }
                recv(user_events) -> user_event => {
                    match user_event?.event {
                        Event::Key(key_event) => match key_event.into() {
                            key!(q) | key!(ctrl-c) | key!(ctrl-q) => {
                                return Ok(());
                            }
                            key!(up) => maze.try_move_up(),
                            key!(right) => maze.try_move_right(),
                            key!(down) => maze.try_move_down(),
                            key!(left) => maze.try_move_left(),
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
            }
        }
        if let Some(beam) = screen_saver_beam.take() {
            ticker.stop_beam(beam);
        }
        if maze.is_won() {
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
        select! {
            recv(ticker.tick_receiver) -> _ => {}
            recv(user_events) -> user_event => {
                match user_event?.event {
                    Event::Key(key_event) => match key_event.into() {
                        key!(ctrl - c) | key!(ctrl - q) => {
                            return Ok(());
                        }
                        _ => {}
                    }
                    Event::Resize(w, h) => {
                        renderer.display = Display::Alternate(Dim::new(w as usize, h as usize));
                    }
                    _ => {}
                }
                event_source.unblock(false);
            }
        }
    }
}
