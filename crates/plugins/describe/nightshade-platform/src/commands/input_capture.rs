use std::any::TypeId;

use crate::contract::input_capture::InputCapture;

pub fn set_pointer_claim<Source: 'static>(capture: &mut InputCapture, claimed: bool) {
    let source = TypeId::of::<Source>();
    if claimed {
        capture.pointer_claims.insert(source);
    } else {
        capture.pointer_claims.remove(&source);
    }
}

pub fn set_keyboard_claim<Source: 'static>(capture: &mut InputCapture, claimed: bool) {
    let source = TypeId::of::<Source>();
    if claimed {
        capture.keyboard_claims.insert(source);
    } else {
        capture.keyboard_claims.remove(&source);
    }
}
