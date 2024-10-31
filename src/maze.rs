use {
    crate::*,
    rand::{
        Rng,
        thread_rng,
    },
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
    invisible_walls: PosSet, // look like rooms, but can't teleport to them
    openings: Vec<Pos>,      // used in growth: where it's possible to dig a new cell
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
    pub fn new<S: Into<String>>(
        name: S,
        dim: Dim,
    ) -> Self {
        let width = dim.w;
        let height = (dim.h / 2) * 2;
        Self {
            name: name.into(),
            dim: Dim::new(width, height),
            rooms: PosSet::new(dim, false),
            invisible_walls: PosSet::new(dim, false),
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
            monsters_period: width + height - 3,
            potions: PosSet::new(dim, false),
            max_monsters: 10,
            default_status: "",
            squared_radius: None,
        }
    }
    pub fn start(&self) -> Option<Pos> {
        self.start
    }
    pub fn set_start(
        &mut self,
        player: Pos,
    ) {
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
    pub fn is_wall(
        &self,
        p: Pos,
    ) -> bool {
        !self.rooms.get(p)
    }
    pub fn is_room(
        &self,
        p: Pos,
    ) -> bool {
        self.rooms.get(p)
    }
    pub fn give_up(&mut self) {
        self.lives = 0;
    }
    /// While a cell can contain several "things", one of them is
    /// more visible and determines how it looks
    pub fn visible_nature(
        &self,
        p: Pos,
    ) -> Nature {
        if !self.rooms.get(p) {
            if self.invisible_walls.get(p) {
                Nature::InvisibleWall
            } else {
                Nature::Wall
            }
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
    fn open(
        &mut self,
        p: Pos,
    ) {
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
    pub fn pos_in_dir(
        &self,
        pos: Pos,
        dir: Dir,
    ) -> Option<Pos> {
        match dir {
            Dir::Up => {
                if pos.y == 0 {
                    None
                } else {
                    Some(Pos::new(pos.x, pos.y - 1))
                }
            }
            Dir::Right => {
                if pos.x == self.dim.w - 1 {
                    None
                } else {
                    Some(Pos::new(pos.x + 1, pos.y))
                }
            }
            Dir::Down => {
                if pos.y == self.dim.h - 1 {
                    None
                } else {
                    Some(Pos::new(pos.x, pos.y + 1))
                }
            }
            Dir::Left => {
                if pos.x == 0 {
                    None
                } else {
                    Some(Pos::new(pos.x - 1, pos.y))
                }
            }
        }
    }
    /// Try moving the player in the given direction, adding both this move
    /// and the monster move to the provided moves vector.
    pub fn try_move(
        &mut self,
        dir: Dir,
        events: &mut EventList,
    ) {
        let Some(p) = self.player else {
            return;
        };
        let Some(dest) = self.pos_in_dir(p, dir) else {
            return;
        };
        if !self.is_room(dest) {
            return;
        }
        events.add_player_move(p, dir, self.visible_nature(dest));
        self.player = Some(dest);
        self.player_moved(events);
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
    fn can_place_exit(&self) -> bool {
        for x in 1..self.dim.w - 1 {
            if self.is_room(Pos::new(x, 1)) {
                return true;
            }
            if self.is_room(Pos::new(x, self.dim.h - 2)) {
                return true;
            }
        }
        for y in 1..self.dim.h - 1 {
            if self.is_room(Pos::new(1, y)) {
                return true;
            }
            if self.is_room(Pos::new(self.dim.w - 2, y)) {
                return true;
            }
        }
        false
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
    pub fn possible_jumps(
        &self,
        p: Pos,
    ) -> Vec<Pos> {
        let mut possible_jumps = Vec::new();
        let r = BLAST_RADIUS
            .min(self.dim.w / 2 - 3)
            .min(self.dim.h / 2 - 3)
            .max(1);
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
    fn len_to_player(
        &self,
        p: Pos,
    ) -> Option<usize> {
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
    fn add_cuts(
        &mut self,
        n: usize,
    ) {
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
    fn add_potions(
        &mut self,
        n: usize,
    ) {
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
    #[allow(dead_code)]
    fn neighbours(
        &self,
        p: Pos,
    ) -> SmallVec<[Pos; 4]> {
        let mut list = SmallVec::new();
        if p.y > 0 {
            list.push(Pos::new(p.x, p.y - 1));
        }
        if p.x < self.dim.w - 1 {
            list.push(Pos::new(p.x + 1, p.y));
        }
        if p.y < self.dim.h - 1 {
            list.push(Pos::new(p.x, p.y + 1));
        }
        if p.x > 0 {
            list.push(Pos::new(p.x - 1, p.y));
        }
        list
    }
    // neighbour cells, including diagonals
    fn neighbours_8(
        &self,
        p: Pos,
    ) -> SmallVec<[Pos; 8]> {
        let mut list = SmallVec::new();
        if p.x > 0 && p.y > 0 {
            list.push(Pos::new(p.x - 1, p.y - 1));
        }
        if p.y > 0 {
            list.push(Pos::new(p.x, p.y - 1));
        }
        if p.x < self.dim.w - 1 && p.y > 0 {
            list.push(Pos::new(p.x + 1, p.y - 1));
        }
        if p.x < self.dim.w - 1 {
            list.push(Pos::new(p.x + 1, p.y));
        }
        if p.x < self.dim.w - 1 && p.y < self.dim.h - 1 {
            list.push(Pos::new(p.x + 1, p.y + 1));
        }
        if p.y < self.dim.h - 1 {
            list.push(Pos::new(p.x, p.y + 1));
        }
        if p.x > 0 && p.y < self.dim.h - 1 {
            list.push(Pos::new(p.x - 1, p.y + 1));
        }
        if p.x > 0 {
            list.push(Pos::new(p.x - 1, p.y));
        }
        list
    }
    // (not counting the border)
    fn inside_neighbours(
        &self,
        p: Pos,
    ) -> SmallVec<[Pos; 4]> {
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
    pub fn enterable_neighbours(
        &self,
        p: Pos,
    ) -> SmallVec<[Pos; 4]> {
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
    fn grow(
        &mut self,
        max: usize,
    ) -> usize {
        for n in 0..max {
            let open = self.seek_open();
            if !open {
                return n;
            }
        }
        max
    }
    /// Due to cuts added after growing, some rooms may be unreachable
    /// in case of interrupted growing. This function makes them
    /// invisble walls to ensure we can't teleport to them.
    fn change_unreachable_rooms_into_invisible_walls(&mut self) {
        let Some(exit) = self.exit else {
            return;
        };
        for x in 0..self.dim.w {
            for y in 0..self.dim.h {
                let pos = Pos::new(x, y);
                if !self.rooms.get(pos) {
                    continue;
                }
                let path = path::find_astar(self, pos, exit);
                if path.is_none() {
                    self.rooms.set(pos, false);
                    self.invisible_walls.set(pos, true);
                }
            }
        }
    }
    /// Make some walls invisible, for cosmetic reasons
    ///
    /// Warning: don't call this before the maze is fully grown and
    /// the exit has been set.
    fn grow_invisible_walls(&mut self) {
        let mut candidates = Vec::new();
        let mut seen = PosSet::new(self.dim, false);
        for x in 0..self.dim.w {
            for y in 0..self.dim.h {
                let pos = Pos::new(x, y);
                if !self.rooms.get(pos) {
                    candidates.push(pos);
                    seen.set(pos, true);
                }
            }
        }
        while let Some(candidate) = candidates.pop() {
            let mut all_walls = true;
            for neighbour in self.neighbours_8(candidate) {
                if self.rooms.get(neighbour) {
                    all_walls = false;
                } else if !seen.get(neighbour) {
                    candidates.push(neighbour);
                    seen.set(neighbour, true);
                }
            }
            if all_walls {
                self.invisible_walls.set(candidate, true);
            }
        }
    }
    pub fn set_highlights(
        &mut self,
        arr: &[Pos],
    ) {
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
    pub fn highlight_path_to_exit(
        &mut self,
        from: Option<Pos>,
    ) {
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
    pub fn kill_player(
        &mut self,
        events: &mut EventList,
    ) {
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
                    let dest = possible_jumps[idx];
                    events.add_teleport(player, possible_jumps, dest);
                    self.player = Some(dest);
                    if self.potions.remove(dest) {
                        self.lives += 1;
                    }
                }
            }
        } else {
            self.lives = 0; // there's no player anyway...
        }
        debug!("Remaining lives: {}", self.lives);
    }
    pub fn player_moved(
        &mut self,
        events: &mut EventList,
    ) {
        if let Some(player) = self.player {
            if self.monsters.contains(&player) {
                self.kill_player(events);
            } else if self.potions.remove(player) {
                self.lives += 1;
            }
        }
        self.end_player_turn(events);
    }
    pub fn move_player_auto(
        &mut self,
        events: &mut EventList,
    ) {
        if let (Some(player), Some(exit)) = (self.player, self.exit) {
            if let Some(path) = path::find_astar(self, player, exit) {
                let dest = path[0];
                if self.monsters.contains(&dest) {
                    self.end_player_turn(events);
                } else {
                    self.try_move(player.dir_to(dest), events);
                }
            } else {
                // workaround for some invalid mazes I observed
                self.kill_player(events);
                self.end_player_turn(events);
            }
        }
    }
    /// move the world
    pub fn end_player_turn(
        &mut self,
        events: &mut EventList,
    ) {
        self.turn += 1;
        if let (Some(player), Some(exit)) = (self.player, self.exit) {
            for i in 0..self.monsters.len() {
                if let Some(dir) = self.monsters[i].step_dir_to(player) {
                    events.add_monster_move(self.monsters[i], dir, Nature::Player);
                    self.monsters[i] = player; // monster takes the player's place
                    self.kill_player(events);
                    break; // other monsters don't move
                }
                if let Some(path) = path::find_astar(self, self.monsters[i], player) {
                    let dest = path[0];
                    if self.monsters.contains(&dest) {
                        continue;
                    }
                    events.add_monster_move(
                        self.monsters[i],
                        self.monsters[i].dir_to(dest),
                        self.visible_nature(dest),
                    );
                    self.monsters[i] = dest;
                    self.potions.set(dest, false);
                    if dest == player {
                        self.kill_player(events);
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
        loop {
            let start = Pos::new(
                rng.gen_range(width / 6..width * 5 / 6),
                rng.gen_range(height / 6..height * 5 / 6),
            );
            if let Some(squared_radius) = maze.squared_radius {
                if Pos::sq_euclidian_distance(start, maze.center()) + 2 > squared_radius {
                    // this isn't a valid starting position: we might be unable to grow
                    // from there
                    continue;
                }
            }
            maze.set_start(start);
            break;
        }
        if specs.fill {
            while maze.grow(10) > 0 {}
        } else {
            let n = (width * height) / 3;
            loop {
                info!("growing 1");
                if maze.grow(n) == 0 {
                    break;
                }
                if maze.can_place_exit() {
                    break;
                }
            }
        }
        maze.add_cuts(specs.cuts);
        maze.add_potions(specs.potions);
        maze.max_monsters = specs.monsters;
        maze.try_make_exit();
        maze.grow_invisible_walls();
        maze.change_unreachable_rooms_into_invisible_walls();
        maze.default_status = specs.status;
        debug!("squared_radius: {:?}", maze.squared_radius);
        maze
    }
}
