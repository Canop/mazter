use {
    crate::*,
    termimad::crossterm::{
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
        QueueableCommand,
    },
    std::io::Write,
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

    fn layout(&self, maze: &Maze) -> Layout {
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

    // the rendering when the maze is very small and we can afford
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
                    (true, true) => ('▀', Colors { foreground: top, background: bot }),
                    (true, false) => ('▀', Colors { foreground: top, background: None }),
                    (false, true) => ('▄', Colors { foreground: bot, background: None }),
                    (false, false) => (' ', Colors { foreground: None, background: None }),
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

    fn spaces<W: Write>(&self, w: &mut W, n: usize) -> anyhow::Result<()> {
        for _ in 0..n {
            w.queue(Print(' '))?;
        }
        Ok(())
    }

    /// Render the maze (with title and lives count) for the TUI,
    /// assuming a buffered writer in an alternate
    pub fn write<W: Write>(&self, w: &mut W, maze: &Maze) -> anyhow::Result<()> {
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
