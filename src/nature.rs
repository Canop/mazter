#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nature {
    Room,
    Wall,
    InvisibleWall,
    Player,
    Monster,
    Potion,
    Highlight,
}
