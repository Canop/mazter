use {
    crate::Dim,
    rand::{
        Rng,
        thread_rng,
    },
};

/// Definition of a maze to build
#[derive(Debug, Clone, Hash)]
pub struct Specs {
    pub name: String,
    pub dim: Dim,
    pub cuts: usize,
    pub potions: usize,
    pub monsters: usize,
    pub lives: i32,
    pub status: &'static str,
    pub disk: bool,
}

impl Specs {
    pub fn for_level(level: usize) -> Self {
        let name = format!("Level {level}");
        let width;
        let height;
        let mut lives;
        let potions;
        let monsters;
        let mut cuts = None;
        if level == 1 {
            width = 20;
            height = 15;
            lives = 1;
            potions = 0;
            monsters = 0;
        } else if level == 5 {
            width = 65;
            height = 45;
            lives = 10;
            monsters = 1;
            potions = 0;
        } else if level == 15 {
            width = 90;
            height = 60;
            lives = 1;
            monsters = 1;
            potions = 30;
        } else if level == 25 {
            width = 100;
            height = 65;
            lives = 1;
            monsters = 1;
            potions = 50;
        } else if level == 35 {
            width = 110;
            height = 80;
            lives = 3;
            monsters = 2;
            potions = 50;
        } else if level == 45 {
            width = 130;
            height = 90;
            lives = 5;
            monsters = 3;
            potions = 50;
        } else if level % 10 == 0 {
            // multiples of 10 are without monster
            width = (29 + level * 5 / 3).min(600);
            height = width * 2 / 3;
            lives = 1;
            potions = 0;
            monsters = 0;
        } else if level % 10 == 3 && level > 3 {
            // no potions, but many cuts
            width = (20 + level * 5 / 3).min(600);
            height = width * 2 / 3;
            lives = 1;
            potions = 0;
            monsters = 1 + level / 20;
            cuts = Some((width * height) / 20);
        } else if level > 15 && level % 7 == 0 {
            // easy levels, for a change
            width = 10 + level % 13;
            height = width * 2 / 3;
            monsters = 3;
            potions = 5;
            lives = 1;
        } else {
            width = (12 + level * 4 / 3).min(400);
            height = (9 + level).min(300);
            lives = if level < 6 || level % 11 == 1 { 3 } else { 1 };
            potions = match level {
                2..=4 => 4 + level,
                5..=20 => 3 + (width * height) / 200,
                _ => (width * height) / 180,
            };
            monsters = 1 + (level / 10).min(2) + (level / 50).min(4);
        }
        let height = (height / 2) * 2;
        let mut cuts = cuts.unwrap_or(match level % 7 {
            4 => (width * height) / 50,
            7 => (width * height) / 120,
            _ => 1 + (width * height) / 150,
        });
        let disk = if level % 23 == 0 && width > 10 && height > 10 {
            cuts /= 2;
            lives += 3;
            true
        } else {
            false
        };
        let status = match level {
            1 => "Use arrow keys to move and exit the maze",
            2 | 4 => "Red monsters teleport you",
            3 => "Pick lives on green squares",
            5 | 7 | 10 => "You can abandon with key 'a'",
            15 => "Try resize your terminal",
            _ => "",
        };
        Self {
            name,
            dim: Dim::new(width, height),
            cuts,
            potions,
            monsters,
            lives,
            status,
            disk,
        }
    }
    pub fn for_terminal_build() -> std::io::Result<Self> {
        let mut rng = thread_rng();
        let double = rng.gen_range(0..3) == 0;
        let dim = if double {
            Dim::new(rng.gen_range(8..35), rng.gen_range(7..20))
        } else {
            let mut d = Dim::terminal()?;
            d.w -= 2;
            d.h = d.h * 2 - 3;
            d
        };
        let cuts = match rng.gen_range(0..3) {
            0 => (dim.w * dim.h) / 2300,
            1 => (dim.w * dim.h) / 500,
            _ => (dim.w * dim.h) / 60, // should be only 2
        };
        Ok(Self {
            name: "random".to_string(),
            dim,
            cuts,
            potions: 0,
            monsters: 0,
            lives: 0,
            status: "",
            disk: rng.gen_range(0..20) == 0,
        })
    }
}
