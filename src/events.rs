use crate::*;

#[derive(Debug, Clone, Default)]
pub struct EventList {
    pub events: Vec<Event>,
}

/// An event that can happen in the game and that we want to animate for clarity.
#[derive(Debug, Clone)]
pub enum Event {
    Move(PosMove),
    Teleport(Teleport),
}

#[derive(Debug, Clone, Copy)]
pub struct PosMove {
    pub start: Pos,
    pub dir: Dir,
    pub moving_nature: Nature,
    pub start_background_nature: Nature,
    pub dest_background_nature: Nature,
}

#[derive(Debug, Clone)]
pub struct Teleport {
    pub start: Pos,
    pub possible_jumps: Vec<Pos>,
    pub arrival: Pos,
}

impl EventList {
    pub fn add_teleport(
        &mut self,
        start: Pos,
        possible_jumps: Vec<Pos>,
        arrival: Pos,
    ) {
        self.events.push(Event::Teleport(Teleport {
            start,
            possible_jumps,
            arrival,
        }));
    }
    pub fn add_player_move(
        &mut self,
        start: Pos,
        dir: Dir,
        dest_background_nature: Nature,
    ) {
        let moving_nature = Nature::Player;
        let start_background_nature = Nature::Room;
        self.events.push(Event::Move(PosMove {
            start,
            dir,
            moving_nature,
            start_background_nature,
            dest_background_nature,
        }));
    }
    pub fn add_monster_move(
        &mut self,
        start: Pos,
        dir: Dir,
        mut dest_background_nature: Nature,
    ) {
        let Some(dest) = start.in_dir(dir) else {
            return;
        };
        // for a better rendering, we set the dest_background_nature to the
        //  one of a player/move leaving the destination
        for event in &self.events {
            if let Event::Move(pos_move) = event {
                if pos_move.start == dest {
                    dest_background_nature = pos_move.moving_nature;
                    break;
                }
            }
        }
        let moving_nature = Nature::Monster;
        let start_background_nature = Nature::Room;
        self.events.push(Event::Move(PosMove {
            start,
            dir,
            moving_nature,
            start_background_nature,
            dest_background_nature,
        }));
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
