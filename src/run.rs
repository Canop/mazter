use {
    crate::*,
    crokey::key,
    crossterm::event::{
        self,
        Event,
    },
    std::io::Write,
};

/// Run the game, assuming the terminal is already in alternate mode
pub fn run<W: Write>(w: &mut W, skin: &Skin, args: &Args) -> anyhow::Result<()> {
    let dim = Dim::terminal()?;
    debug!("terminal size: {dim:?}");
    let mut renderer = Renderer {
        display: Display::Alternate(dim),
        skin,
    };
    let user = &args.user;
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
    } else {
        Database::first_not_won(user)?
    };
    loop {
        let specs = Specs::for_level(level);
        debug!("maze specs: {:#?}", &specs);
        let mut maze: Maze = time!(specs.into());
        while !(maze.is_won() || maze.is_lost()) {
            renderer.write(w, &maze)?;
            w.flush()?;
            let e = event::read();
            // debug!("event: {:?}", e);
            match e {
                Ok(Event::Key(key_event)) => match key_event {
                    key!(q) | key!(ctrl - c) | key!(ctrl - q) => {
                        return Ok(());
                    }
                    key!(up) => maze.try_move_up(),
                    key!(right) => maze.try_move_right(),
                    key!(down) => maze.try_move_down(),
                    key!(left) => maze.try_move_left(),
                    key!(a) => maze.give_up(),
                    _ => {}
                },
                Ok(Event::Resize(w, h)) => {
                    renderer.display = Display::Alternate(Dim::new(w as usize, h as usize));
                }
                _ => {}
            }
        }
        if maze.is_won() {
            level = Database::advance(Achievement::new(user, level))?;
        } else {
            maze.highlight_path_to_exit(maze.start());
        }
        // waiting while the user is displayed that he won or lost
        renderer.write(w, &maze)?;
        w.flush()?;
        let e = event::read();
        debug!("event: {:?}", e);
        match e {
            Ok(Event::Key(key_event)) => match key_event {
                key!(ctrl - c) | key!(ctrl - q) => {
                    return Ok(());
                }
                _ => {}
            },
            Ok(Event::Resize(w, h)) => {
                renderer.display = Display::Alternate(Dim::new(w as usize, h as usize));
            }
            _ => {}
        }
    }
}
