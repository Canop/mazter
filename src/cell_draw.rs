//! fonctions dedicated to drawing cells (i.e. char positions in the terminal)
use {
    std::io::Write,
    termimad::crossterm::{
        QueueableCommand,
        cursor,
        style::{
            Color,
            Colors,
            Print,
            SetColors,
        },
    },
};

// block characterss
static HORIZONTAL_BC: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
static VERTICAL_BC: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub struct CellPos {
    pub x: u16,
    pub y: u16,
}

/// Draw a cell with two colors, one for the left part, one for the right part
pub fn draw_bicolor_horizontal<W: Write>(
    w: &mut W,
    x: u16,
    y: u16,
    left_color: Color,
    right_color: Color,
    av: usize, // part of the left color, in [0, 8] (0 is 100% right_color)
) -> anyhow::Result<()> {
    w.queue(cursor::MoveTo(x, y))?;
    w.queue(SetColors(Colors {
        foreground: Some(left_color),
        background: Some(right_color),
    }))?;
    w.queue(Print(HORIZONTAL_BC[av]))?;
    Ok(())
}

/// Draw a cell with two colors, one for the top part, one for the bottom part
pub fn draw_bicolor_vertical<W: Write>(
    w: &mut W,
    x: u16,
    y: u16,
    top_color: Color,
    bottom_color: Color,
    av: usize, // part of the top color, in [0, 8] (0 is 100% bottom_color)
) -> anyhow::Result<()> {
    w.queue(cursor::MoveTo(x, y))?;
    w.queue(SetColors(Colors {
        foreground: Some(bottom_color),
        background: Some(top_color),
    }))?;
    w.queue(Print(VERTICAL_BC[av]))?;
    Ok(())
}
