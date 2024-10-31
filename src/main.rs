#[macro_use]
extern crate cli_log;

mod achievements;
mod args;
mod cell_draw;
mod dim;
mod display;
mod events;
mod hof;
mod layout;
mod maze;
mod nature;
mod path;
mod pos;
mod pos_map;
mod renderer;
mod run;
mod skin;
mod specs;

use {
    clap::Parser,
    std::io::{
        self,
        Write,
    },
    termimad::crossterm::{
        QueueableCommand,
        cursor,
        event::{
            DisableMouseCapture,
            EnableMouseCapture,
        },
        terminal::{
            self,
            EnterAlternateScreen,
            LeaveAlternateScreen,
        },
    },
};

pub use {
    achievements::*,
    args::*,
    cell_draw::*,
    dim::*,
    display::*,
    events::*,
    layout::*,
    maze::*,
    nature::*,
    pos::*,
    pos_map::*,
    renderer::*,
    run::*,
    skin::*,
    specs::*,
};

/// play the game, runing level after level,
/// in an alternate terminal
fn play(args: &Args) -> anyhow::Result<()> {
    let skin = Skin::build();
    let mut w = std::io::BufWriter::new(std::io::stderr());
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    w.queue(EnableMouseCapture)?;
    terminal::enable_raw_mode()?;
    let r = run(&mut w, &skin, args);
    w.flush()?;
    terminal::disable_raw_mode()?;
    w.queue(DisableMouseCapture)?;
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
}

/// build a maze and print it on stdout
fn build(args: &Args) -> anyhow::Result<()> {
    let specs = if let Some(level) = args.level {
        let user = &args.user;
        if Database::can_play(user, level)? {
            Specs::for_level(level)
        } else {
            anyhow::bail!(
                "User {user:?} must win the previous levels before printing level {level}"
            )
        }
    } else {
        Specs::for_terminal_build()?
    };
    debug!("specs: {:#?}", &specs);
    let skin = Skin::build();
    let maze: Maze = specs.into();
    let renderer = Renderer {
        display: Display::Standard,
        skin: &skin,
    };
    renderer.write(&mut io::stdout(), &maze)
}

fn main() -> anyhow::Result<()> {
    init_cli_log!();
    let args = Args::parse();
    info!("launch args: {:#?}", &args);
    if args.hof {
        hof::print()
    } else if args.reset {
        Database::reset(&args.user, true)
    } else if args.build {
        build(&args)
    } else {
        play(&args)
    }
}
