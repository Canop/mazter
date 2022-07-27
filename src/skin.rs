use {
    crate::Nature,
    crossterm::style::Color,
};

pub struct Skin {
    pub wall: Color,
    pub player: Color,
    pub highlight: Color,
    pub monster: Color,
    pub potion: Color,
}
impl Default for Skin {
    fn default() -> Self {
        Self {
            wall: Color::AnsiValue(102),
            player: Color::AnsiValue(214),
            highlight: Color::AnsiValue(45),
            monster: Color::AnsiValue(196),
            potion: Color::AnsiValue(35),
        }
    }
}
impl Skin {
    pub fn color(&self, nature: Nature) -> Option<Color> {
        match nature {
            Nature::Wall => Some(self.wall),
            Nature::Monster => Some(self.monster),
            Nature::Player => Some(self.player),
            Nature::Potion => Some(self.potion),
            Nature::Highlight => Some(self.highlight),
            Nature::Room => None,
        }
    }
}
