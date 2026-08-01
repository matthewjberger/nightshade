use crate::data::{LofiLook, clean};
use crate::resources::LofiSettings;

pub fn wear(settings: &mut LofiSettings, look: LofiLook) {
    settings.look = look;
    settings.waiting = true;
}

pub fn restore(settings: &mut LofiSettings) {
    wear(settings, clean());
}
