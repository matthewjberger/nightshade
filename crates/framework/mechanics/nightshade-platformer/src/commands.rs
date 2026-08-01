use crate::components::{Coyote, JumpAsked};
use crate::resources::PlatformerSettings;

pub fn ask_for_a_jump(asked: &mut JumpAsked, settings: &PlatformerSettings) {
    asked.0 = settings.buffer;
}

pub fn spend_the_jump(asked: &mut JumpAsked, coyote: &mut Coyote) {
    asked.0 = 0.0;
    coyote.0 = 0.0;
}
