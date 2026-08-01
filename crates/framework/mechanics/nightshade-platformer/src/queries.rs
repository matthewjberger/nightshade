use crate::components::{Coyote, JumpAsked};

pub fn may_jump(coyote: &Coyote, asked: &JumpAsked) -> bool {
    coyote.0 > 0.0 && asked.0 > 0.0
}
