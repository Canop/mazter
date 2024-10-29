use {
    crate::*,
    std::cmp::Ordering,
};

/// A position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}
impl Pos {
    pub fn new(
        x: usize,
        y: usize,
    ) -> Self {
        Self { x, y }
    }
    fn dim(
        a: Pos,
        b: Pos,
    ) -> Dim {
        let w = if a.x > b.x { a.x - b.x } else { b.x - a.x };
        let h = if a.y > b.y { a.y - b.y } else { b.y - a.y };
        Dim::new(w, h)
    }
    pub fn sq_euclidian_distance(
        a: Pos,
        b: Pos,
    ) -> usize {
        let Dim { w, h } = Self::dim(a, b);
        w * w + h * h
    }
    pub fn euclidian_distance(
        a: Pos,
        b: Pos,
    ) -> f32 {
        (Pos::sq_euclidian_distance(a, b) as f32).sqrt()
    }
    pub fn manhattan_distance(
        a: Pos,
        b: Pos,
    ) -> usize {
        let Dim { w, h } = Self::dim(a, b);
        w + h
    }
    pub fn sides(
        a: Pos,
        b: Pos,
    ) -> bool {
        Self::manhattan_distance(a, b) == 1
    }
    /// Return one of the 4 directions if the two positions are just one step away
    pub fn step_dir_to(
        self,
        other: Pos,
    ) -> Option<Dir> {
        if other.y == self.y {
            if other.x == self.x + 1 {
                Some(Dir::Right)
            } else if other.x + 1 == self.x {
                Some(Dir::Left)
            } else {
                None
            }
        } else if other.x == self.x {
            if other.y == self.y + 1 {
                Some(Dir::Down)
            } else if other.y + 1 == self.y {
                Some(Dir::Up)
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Assuming that other is in one of the 4 directions, return the direction
    /// (will return something bad if it's eg the same pos)
    pub fn dir_to(
        self,
        other: Pos,
    ) -> Dir {
        if other.x > self.x {
            Dir::Right
        } else if other.x < self.x {
            Dir::Left
        } else if other.y > self.y {
            Dir::Down
        } else {
            Dir::Up
        }
    }
    pub fn in_dir(
        self,
        dir: Dir,
    ) -> Option<Pos> {
        match dir {
            Dir::Up if self.y > 0 => Some(Pos::new(self.x, self.y - 1)),
            Dir::Right => Some(Pos::new(self.x + 1, self.y)),
            Dir::Down => Some(Pos::new(self.x, self.y + 1)),
            Dir::Left if self.x > 0 => Some(Pos::new(self.x - 1, self.y)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ValuedPos {
    pub pos: Pos,
    pub score: i32,
}
impl ValuedPos {
    pub fn from(
        pos: Pos,
        score: i32,
    ) -> Self {
        ValuedPos { pos, score }
    }
}
impl Eq for ValuedPos {}
impl PartialEq for ValuedPos {
    fn eq(
        &self,
        other: &ValuedPos,
    ) -> bool {
        self.score == other.score
    }
}
// we order in reverse from score
impl Ord for ValuedPos {
    fn cmp(
        &self,
        other: &ValuedPos,
    ) -> Ordering {
        other.score.cmp(&self.score)
    }
}
impl PartialOrd for ValuedPos {
    fn partial_cmp(
        &self,
        other: &ValuedPos,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}
