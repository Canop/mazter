use {
    crate::*,
    std::{
        io::Write,
        thread,
        time::Duration,
    },
    termimad::crossterm::{
        QueueableCommand,
        cursor,
        style::{
            Colors,
            Print,
            ResetColor,
            SetColors,
            SetForegroundColor,
        },
        terminal::{
            Clear,
            ClearType,
        },
    },
};

/// Renders mazes on the set display
pub struct Renderer<'s> {
    pub skin: &'s Skin,
    pub display: Display,
}

impl<'s> Renderer<'s> {
    fn is_alternate(&self) -> bool {
        matches!(self.display, Display::Alternate { .. })
    }

    fn layout(
        &self,
        maze: &Maze,
    ) -> Layout {
        Layout::compute(maze.dim, maze.player(), self.display)
    }

    fn write_game_header<W: Write>(
        &self,
        w: &mut W,
        layout: &Layout,
        maze: &Maze,
    ) -> anyhow::Result<()> {
        w.queue(cursor::MoveTo(0, layout.margin.h as u16))?;
        self.spaces(w, layout.margin.w)?;
        w.queue(Print(&maze.name))?;
        let lives = if maze.lives > 3 {
            format!(" {} ■", maze.lives)
        } else {
            " ■".repeat(maze.lives as usize)
        };
        if layout.content.w > maze.name.len() + 6 {
            self.spaces(w, layout.content.w - maze.name.len() - 6)?;
        }
        w.queue(SetColors(Colors {
            foreground: Some(self.skin.potion),
            background: None,
        }))?;
        w.queue(Print(lives))?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    fn write_game_status<W: Write>(
        &self,
        w: &mut W,
        layout: &Layout,
        maze: &Maze,
    ) -> anyhow::Result<()> {
        w.queue(cursor::MoveTo(
            0,
            (1 + layout.margin.h + layout.content.h) as u16,
        ))?;
        self.spaces(w, layout.margin.w)?;
        w.queue(Print(maze.status()))?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    fn draw_teleport_half_size<W: Write>(
        &self,
        w: &mut W,
        layout: &Layout,
        maze: &Maze,
        teleport: &Teleport,
    ) -> anyhow::Result<()> {
        for &pos in &teleport.possible_jumps {
            let Some((x, y)) = layout.maze_to_screen(pos) else {
                continue;
            };
            let colors = if pos.y % 2 == 0 {
                let bottom_is_jump = teleport
                    .possible_jumps
                    .iter()
                    .any(|p| p.x == pos.x && p.y == pos.y + 1);
                let bottom_nature = if bottom_is_jump {
                    Nature::Monster
                } else {
                    maze.visible_nature(Pos::new(pos.x, pos.y + 1))
                };
                Colors {
                    foreground: self.skin.color(Nature::Monster),
                    background: self.skin.color(bottom_nature),
                }
            } else {
                let top_nature = if pos.y > 0 {
                    let top_is_jump = teleport
                        .possible_jumps
                        .iter()
                        .any(|p| p.x == pos.x && p.y == pos.y - 1);
                    if top_is_jump {
                        Nature::Monster
                    } else {
                        maze.visible_nature(Pos::new(pos.x, pos.y - 1))
                    }
                } else {
                    Nature::Wall
                };
                Colors {
                    foreground: self.skin.color(top_nature),
                    background: self.skin.color(Nature::Monster),
                }
            };
            w.queue(cursor::MoveTo(x, y))?;
            w.queue(SetColors(colors))?;
            w.queue(Print('▀'))?;
        }
        Ok(())
    }
    fn draw_teleport_double_size<W: Write>(
        &self,
        w: &mut W,
        layout: &Layout,
        teleport: &Teleport,
    ) -> anyhow::Result<()> {
        for &pos in &teleport.possible_jumps {
            // x and y are for the leftest one of the two cells
            let (x, y) = layout.maze_to_screen_double_size(pos);
            w.queue(cursor::MoveTo(x, y))?;
            w.queue(SetForegroundColor(self.skin.real_color(Nature::Monster)))?;
            w.queue(Print("██"))?;
        }
        Ok(())
    }

    /// Draw one of the step (in [0..16] of the animation) of a moving pos
    /// when the maze is rendered in double size.
    ///
    /// The maze is supposed already rendered, only the moving pos is drawn.
    /// The pos_move is also supposed taking place in valid positions.
    fn draw_pos_move_step_double_size<W: Write>(
        &self,
        w: &mut W,
        layout: &Layout,
        pos_move: PosMove,
        av: usize, // in [0..16]
    ) -> anyhow::Result<()> {
        // x and y are for the leftest one of the two starting cells
        let (x, y) = layout.maze_to_screen_double_size(pos_move.start);
        let start_bg = self.skin.real_color(pos_move.start_background_nature);
        let dest_bg = self.skin.real_color(pos_move.dest_background_nature);
        let fg = self.skin.real_color(pos_move.moving_nature);
        match pos_move.dir {
            Dir::Up => {
                let av = av / 2; // 8 real steps
                // start pos, leaving
                draw_bicolor_vertical(w, x, y, fg, start_bg, av)?;
                draw_bicolor_vertical(w, x + 1, y, fg, start_bg, av)?;

                // dest pos, arriving
                draw_bicolor_vertical(w, x, y - 1, dest_bg, fg, av)?;
                draw_bicolor_vertical(w, x + 1, y - 1, dest_bg, fg, av)?;
            }
            Dir::Right => {
                let av_left = av.min(8); // left cell of each pos
                let av_right = if av <= 8 { 0 } else { av - 8 };
                draw_bicolor_horizontal(w, x, y, start_bg, fg, av_left)?;
                draw_bicolor_horizontal(w, x + 1, y, start_bg, fg, av_right)?;
                draw_bicolor_horizontal(w, x + 2, y, fg, dest_bg, av_left)?;
                draw_bicolor_horizontal(w, x + 3, y, fg, dest_bg, av_right)?;
            }
            Dir::Down => {
                let av = av / 2; // 8 real steps
                // start pos, leaving
                draw_bicolor_vertical(w, x, y, start_bg, fg, 8 - av)?;
                draw_bicolor_vertical(w, x + 1, y, start_bg, fg, 8 - av)?;

                ////// dest pos, arriving
                draw_bicolor_vertical(w, x, y + 1, fg, dest_bg, 8 - av)?;
                draw_bicolor_vertical(w, x + 1, y + 1, fg, dest_bg, 8 - av)?;
            }
            Dir::Left => {
                let av_left = if av <= 8 { 8 } else { 8 - (av - 8) / 2 };
                let av_right = 8 - av.min(8); // right cell of each pos
                draw_bicolor_horizontal(w, x, y, fg, start_bg, av_left)?;
                draw_bicolor_horizontal(w, x + 1, y, fg, start_bg, av_right)?;
                draw_bicolor_horizontal(w, x - 2, y, dest_bg, fg, av_left)?;
                draw_bicolor_horizontal(w, x - 1, y, dest_bg, fg, av_right)?;
            }
        }
        Ok(())
    }

    /// Animate moves. Return true when the animation occured, false
    /// if it didn't.
    ///
    /// When there was no animation, this function is instant and the animation
    /// may have to be replaced by a wait.
    ///
    /// The maze is supposed already rendered, only the changes are drawn.
    pub fn animate_events<W: Write>(
        &mut self,
        w: &mut W,
        maze: &Maze,
        events: &EventList,
    ) -> anyhow::Result<bool> {
        let layout = self.layout(maze);
        if !self.is_alternate() {
            // a move can only be animated in alternate mode
            return Ok(false);
        }
        if self.skin.room.is_none() {
            // block character animation is not possible without a room color
            return Ok(false);
        }
        w.queue(ResetColor)?;
        // teleports are drawn only once
        for event in &events.events {
            let Event::Teleport(teleport) = event else {
                continue;
            };
            if layout.double_sizes {
                self.draw_teleport_double_size(w, &layout, teleport)?;
            } else {
                self.draw_teleport_half_size(w, &layout, maze, teleport)?;
            }
            w.flush()?;
        }
        if layout.double_sizes {
            // moves are drawn step by step, but only in double size
            for av in 1..=16 {
                for event in &events.events {
                    let Event::Move(pos_move) = event else {
                        continue;
                    };
                    self.draw_pos_move_step_double_size(w, &layout, *pos_move, av)?;
                }
                w.queue(ResetColor)?;
                w.flush()?;
                thread::sleep(Duration::from_millis(8));
            }
        } else {
            // in half size, we just wait
            w.queue(ResetColor)?;
            w.flush()?;
            thread::sleep(Duration::from_millis(120));
        }
        Ok(true)
    }

    // the rendering when the maze is very small and we can afford using 2 characters
    // side by side for each game pos
    fn write_maze_double_size<W: Write>(
        &self,
        w: &mut W,
        layout: &Layout,
        maze: &Maze,
    ) -> anyhow::Result<()> {
        for y in 0..maze.dim.h {
            if self.is_alternate() {
                w.queue(cursor::MoveTo(0, (y + 1 + layout.margin.h) as u16))?;
            }
            self.spaces(w, layout.margin.w)?;
            for x in 0..maze.dim.w {
                let pos = Pos::new(x, y);
                let color = self.skin.color(maze.visible_nature(pos));
                if color.is_some() {
                    let colors = Colors {
                        foreground: color,
                        background: None,
                    };
                    w.queue(SetColors(colors))?;
                    w.queue(Print("██"))?;
                    w.queue(ResetColor)?;
                } else {
                    w.queue(Print("  "))?;
                }
            }
            if self.is_alternate() {
                w.queue(Clear(ClearType::UntilNewLine))?;
            } else {
                writeln!(w)?;
            }
        }
        Ok(())
    }

    fn write_maze_half_size<W: Write>(
        &self,
        w: &mut W,
        layout: &Layout,
        maze: &Maze,
    ) -> anyhow::Result<()> {
        for l in 0..layout.content.h {
            if self.is_alternate() {
                // a terminal line is two maze rows, and there's a +1 for the top texts
                w.queue(cursor::MoveTo(0, (l + 1 + layout.margin.h) as u16))?;
            }
            self.spaces(w, layout.margin.w)?;
            for i in 0..layout.content.w {
                let x = i + layout.trim.w;
                let top_pos = Pos::new(x, 2 * l + layout.trim.h);
                let bot_pos = Pos::new(x, 2 * l + layout.trim.h + 1);
                let top = self.skin.color(maze.visible_nature(top_pos));
                let bot = self.skin.color(maze.visible_nature(bot_pos));
                let (shape, colors) = match (top.is_some(), bot.is_some()) {
                    (true, true) => ('▀', Colors {
                        foreground: top,
                        background: bot,
                    }),
                    (true, false) => ('▀', Colors {
                        foreground: top,
                        background: None,
                    }),
                    (false, true) => ('▄', Colors {
                        foreground: bot,
                        background: None,
                    }),
                    (false, false) => (' ', Colors {
                        foreground: None,
                        background: None,
                    }),
                };
                w.queue(SetColors(colors))?;
                w.queue(Print(shape))?;
                w.queue(ResetColor)?;
            }
            if self.is_alternate() {
                w.queue(Clear(ClearType::UntilNewLine))?;
            } else {
                writeln!(w)?;
            }
        }
        Ok(())
    }

    fn spaces<W: Write>(
        &self,
        w: &mut W,
        n: usize,
    ) -> anyhow::Result<()> {
        for _ in 0..n {
            w.queue(Print(' '))?;
        }
        Ok(())
    }

    /// Render the maze (with title and lives count) for the TUI,
    /// assuming a buffered writer in an alternate
    pub fn write<W: Write>(
        &self,
        w: &mut W,
        maze: &Maze,
    ) -> anyhow::Result<()> {
        let layout = self.layout(maze);
        for i in 0..layout.margin.h {
            if self.is_alternate() {
                w.queue(cursor::MoveTo(0, i as u16))?;
                w.queue(Clear(ClearType::UntilNewLine))?;
            } else {
                writeln!(w)?;
            }
        }
        if self.is_alternate() {
            self.write_game_header(w, &layout, maze)?;
        }
        if layout.double_sizes {
            self.write_maze_double_size(w, &layout, maze)?;
        } else {
            self.write_maze_half_size(w, &layout, maze)?;
        }
        if self.is_alternate() {
            self.write_game_status(w, &layout, maze)?;
            w.queue(Clear(ClearType::FromCursorDown))?;
        }
        Ok(())
    }
}
