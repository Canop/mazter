use {
    crate::*,
    std::io,
    termimad::crossterm::terminal,
};

/// a couple of usize intended as dimensions
/// (screen, maze, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dim {
    pub w: usize,
    pub h: usize,
}
impl Dim {
    pub fn new(
        w: usize,
        h: usize,
    ) -> Self {
        Self { w, h }
    }
    pub fn terminal() -> io::Result<Self> {
        #[allow(unused_mut)]
        let (mut width, mut height) = terminal::size()?;
        #[cfg(windows)]
        {
            width -= 1;
            height -= 1;
        }
        Ok(Self::new(width as usize, height as usize))
    }
    pub fn idx(
        self,
        p: Pos,
    ) -> usize {
        p.x + self.w * p.y
    }
    pub fn verticalize(&mut self) {
        let w = self.h;
        self.h += self.w;
        self.w = (w / 2).max(MIN_DIM);
    }
}
