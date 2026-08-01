use crate::data::SaveDue;

#[derive(Clone, Copy, Debug)]
pub struct Autosave {
    period: f32,
    wait: f32,
    asked: bool,
}

impl Autosave {
    pub fn every(period: f32) -> Self {
        Self {
            period,
            wait: period,
            asked: false,
        }
    }

    pub fn ask(&mut self) {
        self.asked = true;
    }

    pub fn due(&mut self, delta: f32, leaving: bool) -> Option<SaveDue> {
        self.wait -= delta;
        let due = if self.asked {
            Some(SaveDue::Asked)
        } else if leaving {
            Some(SaveDue::Leaving)
        } else if self.wait <= 0.0 {
            Some(SaveDue::Period)
        } else {
            None
        };
        if due.is_some() {
            self.wait = self.period;
            self.asked = false;
        }
        due
    }
}
