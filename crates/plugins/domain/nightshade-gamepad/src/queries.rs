use crate::contract::Gamepad;

pub fn query_active_gamepad<R>(pad: &Gamepad, read: impl FnOnce(&Gamepad) -> R) -> Option<R> {
    pad.connected.then(|| read(pad))
}
