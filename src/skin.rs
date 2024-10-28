use {
    crate::Nature,
    termimad::crossterm::style::Color,
};

pub struct Skin {
    pub wall: Color,
    pub player: Color,
    pub highlight: Color,
    pub monster: Color,
    pub potion: Color,
    pub room: Option<Color>,
}
impl Skin {
    pub fn build() -> Self {
        let room = terminal_light::background_color().ok().map(|c| c.into());
        Self {
            wall: Color::AnsiValue(102),
            player: Color::AnsiValue(214),
            highlight: Color::AnsiValue(45),
            monster: Color::AnsiValue(196),
            potion: Color::AnsiValue(35),
            room,
        }
    }
    pub fn color(
        &self,
        nature: Nature,
    ) -> Option<Color> {
        match nature {
            Nature::Wall => Some(self.wall),
            Nature::Monster => Some(self.monster),
            Nature::Player => Some(self.player),
            Nature::Potion => Some(self.potion),
            Nature::Highlight => Some(self.highlight),
            Nature::Room | Nature::InvisibleWall => self.room,
        }
    }
    pub fn real_color(
        &self,
        nature: Nature,
    ) -> Color {
        match nature {
            Nature::Wall => self.wall,
            Nature::Monster => self.monster,
            Nature::Player => self.player,
            Nature::Potion => self.potion,
            Nature::Highlight => self.highlight,
            Nature::Room | Nature::InvisibleWall => self.room.unwrap_or(Color::Black),
        }
    }
}
