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
    fn dim(self, level: usize) -> Dim {
        match self {
            Self::Tiny => Dim::new(13 + twist(level, 10), MIN_DIM + 1 + twist(level, 5)),
            Self::Small => Dim::new(20 + twist(level, level/10), 15 + twist(level, level/12)),
            Self::Normal => Dim::new(22 + twist(level, level/9), 16 + twist(level, level/11)),
            Self::Large => Dim::new(24 + twist(level, level/8), 18 + twist(level, level/10)),
            Self::Huge => Dim::new(30 + twist(level, level/4), 18 + twist(level, level/6)),
        }
    }
}

/// Return a pseudo-pseudo-random number, capped and reproductible
///  (seed can be eg the level)
fn twist(seed: usize, max: usize) -> usize {
    if max == 0 {
        return 0;
    }
    (seed * 23 + (max + 173) * 347 + (seed * 293)) % max
}

#[test]
fn test_thing() {
    for level in 1..=100 {
        let dim = Specs::for_level(level).dim;
        println!("{} -> {:?}", level, dim);
    }
    todo!();
}

// special:
// - no monster @ l%17 == 1
// - vertical size @ l%20 == 6
// - lot of monster and lives @ l%23 == 0
// - circle @ 1%13 == 5
// -

impl Specs {
    pub fn for_level(level: usize) -> Self {

        let name = format!("Level {level}");
        let mut dim_spec = match level%11 {
            1 | 4 => SizeSpec::Tiny,
            2 | 6 | 8 => SizeSpec::Small,
            3 | 10 if level > 12 => SizeSpec::Large,
            7 if level > 20 => SizeSpec::Huge,
            _ => SizeSpec::Normal,
        };
        let mut dim = dim_spec.dim(level);
        let disk = dim.w > 10 && dim.h > 10 && level % 5 == 0;
        if !disk && level % 13 == 7 {
            dim.verticalize();
        }

        let fill = false; // !(level % 4 == 1 && level > 6);

        let mut lives;
        let potions;
        let monsters;

        lives = 1;
        potions = 0;
        monsters = 0;
        let cuts = match level % 7 {
            0 | 5 => (dim.w * dim.h) / 20,
            2 | 4  => (dim.w * dim.h) / 80,
            _ => (dim.w * dim.h) / 150,
        };

        //let mut lives;
        //let potions;
        //let monsters;
        //let mut cuts = None;
        //if level == 1 {
        //    width = 20;
        //    height = 15;
        //    lives = 1;
        //    potions = 0;
        //    monsters = 0;
        //} else if level == 5 {
        //    width = 65;
        //    height = 45;
        //    lives = 10;
        //    monsters = 1;
        //    potions = 0;
        //} else if level == 15 {
        //    width = 90;
        //    height = 60;
        //    lives = 1;
        //    monsters = 1;
        //    potions = 30;
        //} else if level == 25 {
        //    width = 100;
        //    height = 65;
        //    lives = 1;
        //    monsters = 1;
        //    potions = 50;
        //} else if level == 35 {
        //    width = 110;
        //    height = 80;
        //    lives = 3;
        //    monsters = 2;
        //    potions = 50;
        //} else if level == 45 {
        //    width = 130;
        //    height = 90;
        //    lives = 5;
        //    monsters = 3;
        //    potions = 50;
        //} else if level % 10 == 0 {
        //    // multiples of 10 are without monster
        //    width = (29 + level * 5 / 3).min(600);
        //    height = width * 2 / 3;
        //    lives = 1;
        //    potions = 0;
        //    monsters = 0;
        //} else if level % 10 == 3 && level > 3 {
        //    // no potions, but many cuts
        //    width = (20 + level * 5 / 3).min(600);
        //    height = width * 2 / 3;
        //    lives = 1;
        //    potions = 0;
        //    monsters = 1 + level / 20;
        //    cuts = Some((width * height) / 20);
        //} else if level > 15 && level % 7 == 0 {
        //    // easy levels, for a change
        //    width = 10 + level % 13;
        //    height = width * 2 / 3;
        //    monsters = 3;
        //    potions = 5;
        //    lives = 1;
        //} else {
        //    width = (12 + level * 4 / 3).min(400);
        //    height = (9 + level).min(300);
        //    lives = if level < 6 || level % 11 == 1 { 3 } else { 1 };
        //    potions = match level {
        //        2..=4 => 4 + level,
        //        5..=20 => 3 + (width * height) / 200,
        //        _ => (width * height) / 180,
        //    };
        //    monsters = 1 + (level / 10).min(2) + (level / 50).min(4);
        //}
        //let height = (height / 2) * 2;
        //    cuts /= 2;
        //    lives += 3;
        //    true
        //} else {
        //    false
        //};
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
            dim,
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
