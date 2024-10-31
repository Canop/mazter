use {
    crate::*,
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
    pub fill: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SizeSpec {
    Tiny,
    Small,
    Normal,
    Large,
    Huge,
}
impl SizeSpec {
    fn dim(
        self,
        level: usize,
    ) -> Dim {
        match self {
            Self::Tiny => Dim::new(13 + twist(level, 12), MIN_DIM + 1 + twist(level, 7)),
            Self::Small => Dim::new(20 + twist(level, level / 10), 18 + twist(level, level / 12)),
            Self::Normal => Dim::new(25 + twist(level, level / 9), 20 + twist(level, level / 11)),
            Self::Large => Dim::new(30 + twist(level, level / 8), 24 + twist(level, level / 10)),
            Self::Huge => Dim::new(40 + twist(level, level / 4), 31 + twist(level, level / 6)),
        }
    }
}

/// Return a pseudo-pseudo-random number, capped and reproductible
///  (seed can be eg the level)
fn twist(
    seed: usize,
    max: usize,
) -> usize {
    if max == 0 {
        return 0;
    }
    (seed * 27 + (max + 173) * 347 + (seed * 293)) % max
}

impl Specs {
    pub fn for_level(level: usize) -> Self {
        let name = format!("Level {level}");
        let dim_spec = match level % 11 {
            1 | 4 => SizeSpec::Tiny,
            2 | 6 | 8 => SizeSpec::Small,
            3 | 10 => SizeSpec::Large,
            7 => SizeSpec::Huge,
            _ => SizeSpec::Normal,
        };
        let mut dim = dim_spec.dim(level);
        let disk = level % 7 == 5;
        if disk {
            dim.w = 24.max(dim.w);
            dim.h = 24.max(dim.h);
        } else if level % 13 == 7 {
            dim.verticalize();
        }
        let s = dim.w * dim.h;
        let fill = !disk && !(level % 4 == 1 && level > 6);

        let lives;
        let potions;
        let monsters;
        let cuts;
        // A cycle of progressively harder levels over 10 turns
        match level % 10 {
            1 => {
                // simple walk
                lives = 1;
                monsters = 0;
                potions = 0;
                cuts = 1 + s / 200;
            }
            2 => {
                // super easy
                lives = 3;
                monsters = 1;
                potions = 5 + s / (40 + level);
                cuts = 1 + s / 100;
            }
            3 if level > 10 => {
                //
                lives = 4;
                monsters = 2 + level / 60;
                potions = 2 + s / (100 + level);
                cuts = 1 + s / 160;
            }
            4 if level > 10 => {
                //
                lives = 2;
                monsters = 2;
                potions = 5 + s / (100 + level);
                cuts = 1 + s / 200;
            }
            5 if level > 20 => {
                // lot of cuts, few potions and lives
                lives = 1;
                monsters = 2;
                potions = 4 + s / (420 + level);
                cuts = 1 + s / 100;
            }
            6 if level > 30 => {
                lives = 1;
                monsters = 5 + level / 90;
                potions = 1 + s / 150;
                cuts = 1 + s / 150;
            }
            7 if level > 30 => {
                lives = 2;
                monsters = 3 + level / 100;
                potions = 5 + s / 100;
                cuts = 1 + s / 200;
            }
            8 if level > 40 => {
                lives = 2;
                monsters = 4;
                potions = 1 + s / (150 + level);
                cuts = 1 + s / 200;
            }
            9 if level > 50 => {
                //
                lives = 2;
                monsters = 5 + level / 100;
                potions = 1 + s / (200 + 2 * level);
                cuts = 1 + s / 100;
            }
            _ => {
                //
                lives = 1;
                monsters = 2;
                potions = 7 + s / (30 + 4 * level);
                cuts = 1 + s / (60 + 2 * level);
            }
        }
        let status = match level {
            1 => "Use arrow keys to move and exit the maze",
            2 | 4 => "Red monsters teleport you",
            3 | 6 => "Pick lives on green squares",
            5 | 8 | 12 => "You can abandon with key 'a'",
            10 | 14 | 17 => "Hit 'w' to wait",
            11 => "Sometimes there's no monster, just find the exit",
            _ => "",
        };
        Self {
            name,
            dim,
            cuts,
            potions,
            monsters,
            lives,
            status,
            disk,
            fill,
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
        let fill = rng.gen_range(0..5) < 4;
        Ok(Self {
            name: "random".to_string(),
            dim,
            cuts,
            potions: 0,
            monsters: 0,
            lives: 0,
            status: "",
            disk: rng.gen_range(0..20) == 0,
            fill,
        })
    }
}
