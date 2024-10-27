use {
    crate::*,
    rand::{thread_rng, Rng},
    smallvec::SmallVec,
};

pub const MIN_JUMP: usize = 2;
pub const BLAST_RADIUS: usize = 4; // must be greater than MIN_JUMP
pub const MIN_DIM: usize = 7; // must be greater than BLAST_RADIUS + 2

/// A maze and the state of the game (player
/// and monster positions, etc.)
pub struct Maze {
    pub name: String,
    pub dim: Dim,
    rooms: PosSet,
    openings: Vec<Pos>, // used in growth: where it's possible to dig a new cell
    exit: Option<Pos>,
    start: Option<Pos>,
    player: Option<Pos>,
    cuts: Vec<Pos>,
    highlights: PosSet,
    monsters: Vec<Pos>,
    turn: usize,         // a counter incremented at every end_turn
    next_monster: usize, // turn at which a new monster should appear
    pub lives: i32,
    monsters_period: usize,
    potions: PosSet,
    max_monsters: usize,
    pub default_status: &'static str,
    squared_radius: Option<usize>,
}
impl Maze {
    pub fn new<S: Into<String>>(name: S, dim: Dim) -> Self {
        let width = dim.w;
        let height = (dim.h / 2) * 2;
        Self {
            name: name.into(),
            dim: Dim::new(width, height),
            rooms: PosSet::new(dim, false),
            openings: Vec::new(),
            start: None,
            exit: None,
            player: None,
            cuts: Vec::new(),
            monsters: Vec::new(),
            highlights: PosSet::new(dim, false),
            turn: 0,
            next_monster: 50.min((width + height) / 3),
            lives: 1,
            monsters_period: width + height,
            potions: PosSet::new(dim, false),
            max_monsters: 10,
            default_status: "",
            squared_radius: None,
        }
    }
    pub fn start(&self) -> Option<Pos> {
        self.start
    }
    pub fn set_start(&mut self, player: Pos) {
        self.start = Some(player);
        self.player = Some(player);
        self.open(player);
    }
    pub fn status(&self) -> &'static str {
        if self.is_won() {
            "You win. Hit any key for next level"
        } else if self.is_lost() {
            "You lost. Hit any key to try again"
        } else {
            self.default_status
        }
    }
    pub fn player(&self) -> Option<Pos> {
        self.player
    }
    pub fn is_won(&self) -> bool {
        self.player == self.exit
    }
    pub fn is_lost(&self) -> bool {
        self.lives < 1
    }
    pub fn is_wall(&self, p: Pos) -> bool {
        !self.rooms.get(p)
    }
    pub fn is_room(&self, p: Pos) -> bool {
        self.rooms.get(p)
    }
    pub fn give_up(&mut self) {
        self.lives = 0;
    }
    /// While a cell can contain several "things", one of them is
    /// more visible and determines how it looks
    pub fn visible_nature(&self, p: Pos) -> Nature {
        if self.is_wall(p) {
            Nature::Wall
        } else if self.monsters.contains(&p) {
            Nature::Monster
        } else if Some(p) == self.player {
            Nature::Player
        } else if self.potions.get(p) {
            Nature::Potion
        } else if self.highlights.get(p) {
            Nature::Highlight
        } else {
            Nature::Room
        }
    }
    fn center(&self) -> Pos {
        Pos::new(self.dim.w / 2, self.dim.h / 2)
    }
    fn open(&mut self, p: Pos) {
        self.rooms.set(p, true);
        let neighbours = self.inside_neighbours(p);
        for p in neighbours {
            if self.is_wall(p) {
                if let Some(squared_radius) = self.squared_radius {
                    if Pos::sq_euclidian_distance(p, self.center()) > squared_radius {
                        continue;
                    }
                }
                self.openings.push(p);
            }
        }
    }
    pub fn try_move_up(&mut self) {
        if let Some(p) = self.player {
            if p.y == 0 {
                return;
            }
            let p = Pos::new(p.x, p.y - 1);
            if self.is_room(p) {
                self.player = Some(p);
                self.player_moved();
            }
        }
    }
    pub fn try_move_right(&mut self) {
        if let Some(p) = self.player {
            if p.x == self.dim.w - 1 {
                return;
            }
            let p = Pos::new(p.x + 1, p.y);
            if self.is_room(p) {
                self.player = Some(p);
                self.player_moved();
            }
        }
    }
    pub fn try_move_down(&mut self) {
        if let Some(p) = self.player {
            if p.y == self.dim.h - 1 {
                return;
            }
            let p = Pos::new(p.x, p.y + 1);
            if self.is_room(p) {
                self.player = Some(p);
                self.player_moved();
            }
        }
    }
    pub fn try_move_left(&mut self) {
        if let Some(p) = self.player {
            if p.x == 0 {
                return;
            }
            let p = Pos::new(p.x - 1, p.y);
            if self.is_room(p) {
                self.player = Some(p);
                self.player_moved();
            }
        }
    }
    fn seek_open(&mut self) -> bool {
        let mut rng = thread_rng();
        loop {
            if self.openings.is_empty() {
                return false;
            }
            let len = self.openings.len();
            let tail = match len % 35 {
                0 => len,
                1 => len.min(15),
                _ => len.min(4),
            };
            let idx: usize = rng.gen_range(0..tail);
            let opening = self.openings.swap_remove(len - idx - 1);
            let neighbours = self.inside_neighbours(opening);
            let room_count = neighbours.iter().filter(|&p| self.is_room(*p)).count();
            if room_count != 1 {
                continue;
            }
            self.open(opening);
            return true;
        }
    }
    // border walls which, when open, make an exit
    fn possible_exits(&self) -> Vec<Pos> {
        let mut possible_exits = Vec::new();
        for x in 1..self.dim.w - 1 {
            if self.is_room(Pos::new(x, 1)) {
                possible_exits.push(Pos::new(x, 0));
            }
            if self.is_room(Pos::new(x, self.dim.h - 2)) {
                possible_exits.push(Pos::new(x, self.dim.h - 1));
            }
        }
        for y in 1..self.dim.h - 1 {
            if self.is_room(Pos::new(1, y)) {
                possible_exits.push(Pos::new(0, y));
            }
            if self.is_room(Pos::new(self.dim.w - 2, y)) {
                possible_exits.push(Pos::new(self.dim.w - 1, y));
            }
        }
        possible_exits
    }
    pub fn possible_jumps(&self, p: Pos) -> Vec<Pos> {
        let mut possible_jumps = Vec::new();
        let r = BLAST_RADIUS.min(self.dim.w / 2 - 3).min(self.dim.h / 2 - 3).max(1);
        let c = Pos::new(
            p.x.max(r + 1).min(self.dim.w - r - 1),
            p.y.max(r + 1).min(self.dim.h - r - 1),
        );
        for x in c.x - r..=c.x + r {
            for y in c.y - r..=c.y + r {
                let d = Pos::new(x, y);
                if self.is_wall(d) || self.monsters.contains(&d) {
                    continue;
                }
                if Pos::manhattan_distance(p, d) >= MIN_JUMP {
                    possible_jumps.push(d);
                }
            }
        }
        possible_jumps
    }
    fn len_to_player(&self, p: Pos) -> Option<usize> {
        self.player
            .and_then(|player| path::find_astar(self, player, p))
            .map(|path| path.len())
    }
    fn try_make_exit(&mut self) {
        if let Some(exit) = self.exit {
            self.rooms.set(exit, true);
        }
        self.exit = self
            .possible_exits()
            .drain(..)
            .max_by_key(|p| self.len_to_player(*p).unwrap_or(0));
        if let Some(exit) = self.exit {
            self.rooms.set(exit, true);
        }
    }
    fn empty_rooms(&self) -> Vec<Pos> {
        let mut empty_rooms = Vec::new();
        for x in 1..self.dim.w - 1 {
            for y in 1..self.dim.h - 1 {
                let p = Pos::new(x, y);
                if self.is_room(p)
                    && Some(p) != self.player
                    && !self.potions.get(p)
                    && !self.monsters.contains(&p)
                {
                    empty_rooms.push(p);
                }
            }
        }
        empty_rooms
    }
    fn possible_cuts(&self) -> Vec<Pos> {
        let mut possible_cuts = Vec::new();
        for x in 1..self.dim.w - 1 {
            for y in 1..self.dim.h - 1 {
                let p = Pos::new(x, y);
                if let Some(squared_radius) = self.squared_radius {
                    if Pos::sq_euclidian_distance(p, self.center()) > squared_radius {
                        continue;
                    }
                }
                if self.is_wall(p) && Some(p) != self.player {
                    possible_cuts.push(p);
                }
            }
        }
        possible_cuts
    }
    fn add_cuts(&mut self, n: usize) {
        debug!("adding {n} cuts");
        let mut possible_cuts = self.possible_cuts();
        let mut rng = thread_rng();
        let mut added = 0;
        while added < n && !possible_cuts.is_empty() {
            let idx: usize = rng.gen_range(0..possible_cuts.len());
            let cut = possible_cuts.swap_remove(idx);
            self.cuts.push(cut);
            self.rooms.set(cut, true);
            added += 1;
        }
    }
    fn add_potions(&mut self, n: usize) {
        debug!("adding {n} potions");
        let mut empty_rooms = self.empty_rooms();
        let mut rng = thread_rng();
        let mut added = 0;
        while added < n && !empty_rooms.is_empty() {
            let idx: usize = rng.gen_range(0..empty_rooms.len());
            let potion = empty_rooms.swap_remove(idx);
            self.potions.set(potion, true);
            added += 1;
        }
    }
    // (not counting the border)
    fn inside_neighbours(&self, p: Pos) -> SmallVec<[Pos; 4]> {
        let mut list = SmallVec::new();
        if p.y > 1 {
            list.push(Pos::new(p.x, p.y - 1));
        }
        if p.x < self.dim.w - 2 {
            list.push(Pos::new(p.x + 1, p.y));
        }
        if p.y < self.dim.h - 2 {
            list.push(Pos::new(p.x, p.y + 1));
        }
        if p.x > 1 {
            list.push(Pos::new(p.x - 1, p.y));
        }
        list
    }
    pub fn enterable_neighbours(&self, p: Pos) -> SmallVec<[Pos; 4]> {
        let mut list = SmallVec::new();
        if p.y > 0 {
            let e = Pos::new(p.x, p.y - 1);
            if self.is_room(e) {
                list.push(e);
            }
        }
        if p.x < self.dim.w - 1 {
            let e = Pos::new(p.x + 1, p.y);
            if self.is_room(e) {
                list.push(e);
            }
        }
        if p.y < self.dim.h - 1 {
            let e = Pos::new(p.x, p.y + 1);
            if self.is_room(e) {
                list.push(e);
            }
        }
        if p.x > 0 {
            let e = Pos::new(p.x - 1, p.y);
            if self.is_room(e) {
                list.push(e);
            }
        }
        list
    }
    fn grow(&mut self, max: usize) -> usize {
        for n in 0..max {
            let open = self.seek_open();
            if !open {
                return n;
            }
        }
        max
    }
    pub fn set_highlights(&mut self, arr: &[Pos]) {
        self.highlights.clear();
        for p in arr {
            self.highlights.set(*p, true);
        }
    }
    pub fn clear_highlight(&mut self) -> bool {
        if self.highlights.is_empty() {
            // slow
            false
        } else {
            self.highlights.clear();
            true
        }
    }
    pub fn highlight_start(&mut self) {
        if let Some(start) = self.start {
            self.highlights.set(start, true);
        }
    }
    pub fn highlight_path_to_exit(&mut self, from: Option<Pos>) {
        if let (Some(start), Some(exit)) = (from, self.exit) {
            let path = time!(path::find_astar(self, start, exit));
            if let Some(path) = path {
                for p in &path {
                    self.highlights.set(*p, true);
                }
            }
            self.highlights.set(exit, true);
        }
    }
    // remove a life and teleport the player
    pub fn kill_player(&mut self) {
        self.lives -= 1;
        if let Some(player) = self.player {
            if self.lives > 0 {
                // random jump on collision
                let possible_jumps = self.possible_jumps(player);
                if possible_jumps.is_empty() {
                    self.lives = 0;
                } else {
                    let mut rng = thread_rng();
                    let idx = rng.gen_range(0..possible_jumps.len());
                    self.player = Some(possible_jumps[idx]);
                    if self.potions.remove(possible_jumps[idx]) {
                        self.lives += 1;
                    }
                }
            }
        } else {
            self.lives = 0; // there's no player anyway...
        }
        debug!("Remaining lives: {}", self.lives);
    }
    pub fn player_moved(&mut self) {
        if let Some(player) = self.player {
            if self.monsters.contains(&player) {
                self.kill_player();
            } else if self.potions.remove(player) {
                self.lives += 1;
            }
        }
        self.end_player_turn();
    }
    pub fn move_player_auto(&mut self) {
        if let (Some(player), Some(exit)) = (self.player, self.exit) {
            if let Some(path) = path::find_astar(self, player, exit) {
                let dest = path[0];
                if !self.monsters.contains(&dest) {
                    self.player = Some(dest);
                }
            } else {
                // workaround for some invalid mazes I observed
                self.kill_player();
            }
        }
        self.player_moved();
    }
    pub fn end_player_turn(&mut self) {
        self.turn += 1;
        if let (Some(player), Some(exit)) = (self.player, self.exit) {
            for i in 0..self.monsters.len() {
                if Pos::sides(self.monsters[i], player) {
                    self.monsters[i] = player; // monster takes the player's place
                    self.kill_player();
                    break; // other monsters don't move
                }
                if let Some(path) = path::find_astar(self, self.monsters[i], player) {
                    let dest = path[0];
                    if self.monsters.contains(&dest) {
                        continue;
                    }
                    self.monsters[i] = dest;
                    self.potions.set(dest, false);
                    if dest == player {
                        self.kill_player();
                        break; // other monsters don't move
                    }
                }
            }
            if self.monsters.len() < self.max_monsters && self.turn == self.next_monster {
                let can_appear = exit != player && !self.monsters.contains(&exit);
                if can_appear {
                    self.monsters.push(exit);
                } else {
                    self.next_monster += 1;
                }
                if self.monsters.len() < 10 {
                    self.next_monster = self.turn + self.monsters_period;
                    self.monsters_period += match self.monsters.len() {
                        1 => 105,
                        2 => 60,
                        _ => 35,
                    };
                }
            }
        }
    }
}

impl From<Specs> for Maze {
    fn from(specs: Specs) -> Self {
        let width = specs.dim.w;
        let height = specs.dim.h;
        let mut maze = Self::new(&specs.name, specs.dim);
        if specs.disk {
            let d = width.min(height) / 2;
            if d > 10 {
                maze.squared_radius = Some((d + 1) * (d + 1));
            }
        }
        maze.lives = specs.lives;
        let mut rng = thread_rng();
        maze.set_start(Pos::new(
            rng.gen_range(width / 6..width * 5 / 6),
            rng.gen_range(height / 6..height * 5 / 6),
        ));
        while maze.grow(1) > 0 {}
        maze.add_cuts(specs.cuts);
        maze.add_potions(specs.potions);
        maze.max_monsters = specs.monsters;
        maze.try_make_exit();
        maze.default_status = specs.status;
        debug!("squared_radius: {:?}", maze.squared_radius);
        maze
    }
}
