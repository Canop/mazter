use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct PosMove {
    pub start: Pos,
    pub dir: Dir,
    pub moving_nature: Nature,
    pub start_background_nature: Nature,
    pub dest_background_nature: Nature,
}
