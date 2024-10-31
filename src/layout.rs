use crate::*;

#[derive(Debug, Clone)]
pub struct Layout {
    pub content: Dim,
    pub margin: Dim, // top and left margin for the whole (including texts)
    pub trim: Dim,   // what part of the maze, left and top, is out of screen
    pub double_sizes: bool,
}

impl Layout {
    /// return the canonical cell position from a maze position.
    ///
    /// If the layout is double_sizes, this is the leftest one of the two
    ///   cells which make the position.
    /// If the layout is half size, only one half of the cell belongs to
    ///   the provided maze position.
    /// Return None if the position is out of the maze rendering area (can't
    ///   happen in double_sizes mode)
    pub fn maze_to_screen(
        &self,
        pos: Pos,
    ) -> Option<(u16, u16)> {
        if self.double_sizes {
            Some(self.maze_to_screen_double_size(pos))
        } else {
            if pos.x < self.trim.w || pos.y < self.trim.h {
                None
            } else {
                let x = pos.x - self.trim.w;
                let y = pos.y - self.trim.h;
                if x >= self.content.w || y >= self.content.h {
                    None
                } else {
                    let x = self.margin.w + x;
                    let y = self.margin.h + 1 + y / 2; // 1 for the top texts
                    Some((x as u16, y as u16))
                }
            }
        }
    }
    /// Assuming the layout is in double_sizes mode, return the leftest
    /// cell for tye maze position
    pub fn maze_to_screen_double_size(
        &self,
        pos: Pos,
    ) -> (u16, u16) {
        let x = self.margin.w + 2 * pos.x;
        let y = self.margin.h + pos.y + 1; // 1 for the top texts
        (x as u16, y as u16)
    }
    pub fn compute(
        maze_dim: Dim,
        player_pos: Option<Pos>,
        display: Display,
    ) -> Self {
        let content_width;
        let content_height;
        let mut left_trim = 0;
        let mut top_trim = 0;
        let left_margin;
        let top_margin;
        let double_sizes;
        // we assume maze_dim.h is fair (it must be)
        match display {
            Display::Alternate(Dim { w, h }) => {
                let available_width = w;
                let available_height = h - 3;
                double_sizes = 2 * maze_dim.w < available_width && maze_dim.h < available_height;
                if double_sizes {
                    content_width = 2 * maze_dim.w;
                    content_height = maze_dim.h;
                } else {
                    if maze_dim.w > available_width {
                        content_width = available_width;
                        if let Some(player) = player_pos {
                            if player.x > available_width / 2 {
                                left_trim = (player.x - available_width / 2)
                                    .min(maze_dim.w - content_width);
                            }
                        }
                    } else {
                        content_width = maze_dim.w;
                    }
                    if maze_dim.h / 2 > available_height {
                        content_height = available_height;
                        if let Some(player) = player_pos {
                            if player.y > available_height {
                                top_trim = (player.y - available_height)
                                    .min(maze_dim.h - 2 * content_height);
                            }
                        }
                    } else {
                        content_height = maze_dim.h / 2;
                    }
                };
                left_margin = (available_width - content_width) / 2;
                top_margin = (available_height - content_height) / 2;
            }
            Display::Standard => {
                left_margin = 1;
                top_margin = 0;
                double_sizes = maze_dim.w < 20 && maze_dim.h < 30;
                if double_sizes {
                    content_width = 2 * maze_dim.w;
                    content_height = maze_dim.h;
                } else {
                    content_width = maze_dim.w;
                    content_height = maze_dim.h / 2;
                };
            }
        }
        Self {
            content: Dim::new(content_width, content_height),
            margin: Dim::new(left_margin, top_margin),
            trim: Dim::new(left_trim, top_trim),
            double_sizes,
        }
    }
}
