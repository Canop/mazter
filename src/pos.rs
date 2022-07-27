use {
    crate::*,
    std::cmp::Ordering,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}
impl Pos {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    fn dim(a: Pos, b: Pos) -> Dim {
        let w = if a.x > b.x { a.x - b.x } else { b.x - a.x };
        let h = if a.y > b.y { a.y - b.y } else { b.y - a.y };
        Dim::new(w, h)
    }
    pub fn sq_euclidian_distance(a: Pos, b: Pos) -> usize {
        let Dim { w, h } = Self::dim(a, b);
        w * w + h * h
    }
    pub fn euclidian_distance(a: Pos, b: Pos) -> f32 {
        (Pos::sq_euclidian_distance(a, b) as f32).sqrt()
    }
    pub fn manhattan_distance(a: Pos, b: Pos) -> usize {
        let Dim { w, h } = Self::dim(a, b);
        w + h
    }
    pub fn sides(a: Pos, b: Pos) -> bool {
        Self::manhattan_distance(a, b) == 1
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ValuedPos {
    pub pos: Pos,
    pub score: i32,
}
impl ValuedPos {
    pub fn from(pos: Pos, score: i32) -> Self {
        ValuedPos { pos, score }
    }
}
impl Eq for ValuedPos {}
impl PartialEq for ValuedPos {
    fn eq(&self, other: &ValuedPos) -> bool {
        self.score == other.score
    }
}
// we order in reverse from score
impl Ord for ValuedPos {
    fn cmp(&self, other: &ValuedPos) -> Ordering {
        other.score.cmp(&self.score)
    }
}
impl PartialOrd for ValuedPos {
    fn partial_cmp(&self, other: &ValuedPos) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
