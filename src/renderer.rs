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

#[derive(Debug, Clone)]
struct Layout {
    content: Dim,
    margin: Dim, // top and left margin
    trim: Dim,
    double_sizes: bool,
}

impl<'s> Renderer<'s> {
    fn is_alternate(&self) -> bool {
        matches!(self.display, Display::Alternate { .. })
    }

    fn layout(
        &self,
        maze: &Maze,
    ) -> Layout {
        let content_width;
        let content_height;
        let mut left_trim = 0;
        let mut top_trim = 0;
        let left_margin;
        let top_margin;
        let double_sizes;
        // we assume maze.dim.h is fair (it must be)
        match self.display {
            Display::Alternate(Dim { w, h }) => {
                let available_width = w;
                let available_height = h - 3;
                double_sizes = 2 * maze.dim.w < available_width && maze.dim.h < available_height;
                if double_sizes {
                    content_width = 2 * maze.dim.w;
                    content_height = maze.dim.h;
                } else {
                    if maze.dim.w > available_width {
                        content_width = available_width;
                        if let Some(player) = maze.player() {
                            if player.x > available_width / 2 {
                                left_trim = (player.x - available_width / 2)
                                    .min(maze.dim.w - content_width);
                            }
                        }
                    } else {
                        content_width = maze.dim.w;
                    }
                    if maze.dim.h / 2 > available_height {
                        content_height = available_height;
                        if let Some(player) = maze.player() {
                            if player.y > available_height {
                                top_trim = (player.y - available_height)
                                    .min(maze.dim.h - 2 * content_height);
                            }
                        }
                    } else {
                        content_height = maze.dim.h / 2;
                    }
                };
                left_margin = (available_width - content_width) / 2;
                top_margin = (available_height - content_height) / 2;
            }
            Display::Standard => {
                left_margin = 1;
                top_margin = 0;
                double_sizes = maze.dim.w < 20 && maze.dim.h < 30;
                if double_sizes {
                    content_width = 2 * maze.dim.w;
                    content_height = maze.dim.h;
                } else {
                    content_width = maze.dim.w;
                    content_height = maze.dim.h / 2;
                };
            }
        }
        Layout {
            content: Dim::new(content_width, content_height),
            margin: Dim::new(left_margin, top_margin),
            trim: Dim::new(left_trim, top_trim),
            double_sizes,
        }
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

    /// Draw one of the step (in [0..8] of the animation) of a moving pos
    /// when the maze is rendered in double size.
    ///
    /// The maze is supposed already rendered, only the moving pos is drawn.
    /// The pos_move is also supposed taking place in valid positions.
    ///
    /// Note: for horizontal moves, we could have 16 positions, not just 8,
    /// but 8 looks good enough.
    fn draw_pos_move_step_double_size<W: Write>(
        &self,
        w: &mut W,
        layout: &Layout,
        pos_move: PosMove,
        av: usize,
    ) -> anyhow::Result<()> {
        // x and y are for the leftest one of the two starting cells
        let x = (layout.margin.w + 2 * pos_move.start.x) as u16;
        let y = (layout.margin.h + pos_move.start.y + 1) as u16;
        let start_bg = self.skin.real_color(pos_move.start_background_nature);
        let dest_bg = self.skin.real_color(pos_move.dest_background_nature);
        let fg = self.skin.real_color(pos_move.moving_nature);
        match pos_move.dir {
            Dir::Up => {
                // start pos, leaving
                draw_bicolor_vertical(w, x, y, fg, start_bg, av)?;
                draw_bicolor_vertical(w, x + 1, y, fg, start_bg, av)?;

                // dest pos, arriving
                draw_bicolor_vertical(w, x, y - 1, dest_bg, fg, av)?;
                draw_bicolor_vertical(w, x + 1, y - 1, dest_bg, fg, av)?;
            }
            Dir::Right => {
                let av_left = (av * 2).min(8); // left cell of each pos
                let av_right = if av <= 4 { 0 } else { (av - 4) * 2 };
                draw_bicolor_horizontal(w, x, y, start_bg, fg, av_left)?;
                draw_bicolor_horizontal(w, x + 1, y, start_bg, fg, av_right)?;
                draw_bicolor_horizontal(w, x + 2, y, fg, dest_bg, av_left)?;
                draw_bicolor_horizontal(w, x + 3, y, fg, dest_bg, av_right)?;
            }
            Dir::Down => {
                // start pos, leaving
                draw_bicolor_vertical(w, x, y, start_bg, fg, 8 - av)?;
                draw_bicolor_vertical(w, x + 1, y, start_bg, fg, 8 - av)?;

                ////// dest pos, arriving
                draw_bicolor_vertical(w, x, y + 1, fg, dest_bg, 8 - av)?;
                draw_bicolor_vertical(w, x + 1, y + 1, fg, dest_bg, 8 - av)?;
            }
            Dir::Left => {
                let av_left = if av <= 4 { 8 } else { 8 - (av - 4) * 2 };
                let av_right = 8 - (av * 2).min(8); // right cell of each pos
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
    /// The maze is supposed already rendered, only the moving pos are drawn.
    pub fn animate_moves<W: Write>(
        &mut self,
        w: &mut W,
        maze: &Maze,
        pos_moves: &[PosMove],
    ) -> anyhow::Result<bool> {
        let layout = self.layout(maze);
        if !layout.double_sizes {
            // we can't animate moves in half size
            return Ok(false);
        }
        if self.skin.room.is_none() {
            // block character animation is not possible without a room color
            return Ok(false);
        }
        w.queue(ResetColor)?;
        for av in 0..=8 {
            for pos_move in pos_moves {
                self.draw_pos_move_step_double_size(w, &layout, *pos_move, av)?;
            }
            w.queue(ResetColor)?;
            w.flush()?;
            thread::sleep(Duration::from_millis(13));
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
                // a terminal line is two maze rows
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
