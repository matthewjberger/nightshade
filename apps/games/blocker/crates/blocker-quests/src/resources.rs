use std::collections::HashSet;

pub struct Journal {
    pub active: Vec<usize>,
    pub finished: Vec<usize>,
    pub renown: u32,
    pub xp: u32,
    pub coin: u32,
    pub slain: Vec<u32>,
    pub met: Vec<usize>,
    pub seen: Vec<usize>,
    pub discovered: HashSet<[i32; 3]>,
    pub found_secret: bool,
}

impl Default for Journal {
    fn default() -> Self {
        Self {
            active: vec![crate::data::quests::OPENING],
            finished: Vec::new(),
            renown: 0,
            xp: 0,
            coin: 0,
            slain: Vec::new(),
            met: Vec::new(),
            seen: Vec::new(),
            discovered: HashSet::new(),
            found_secret: false,
        }
    }
}

impl Journal {
    pub fn slain_count(&self, kind: usize) -> u32 {
        self.slain.get(kind).copied().unwrap_or(0)
    }

    pub fn record_slain(&mut self, kind: usize) {
        if self.slain.len() <= kind {
            self.slain.resize(kind + 1, 0);
        }
        self.slain[kind] += 1;
    }

    pub fn forget_slain(&mut self, kind: usize) {
        if kind < self.slain.len() {
            self.slain[kind] = 0;
        }
    }

    pub fn record_met(&mut self, giver: usize) {
        if !self.met.contains(&giver) {
            self.met.push(giver);
        }
    }

    pub fn record_seen(&mut self, place: usize) {
        if !self.seen.contains(&place) {
            self.seen.push(place);
        }
    }
}

#[derive(Default)]
pub struct Waypoint {
    pub at: Option<nalgebra_glm::Vec3>,
    pub name: String,
}
