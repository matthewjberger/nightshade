use crate::contract::input_capture::InputCapture;

pub fn pointer_captured(capture: &InputCapture) -> bool {
    !capture.pointer_claims.is_empty()
}

pub fn keyboard_captured(capture: &InputCapture) -> bool {
    !capture.keyboard_claims.is_empty()
}
