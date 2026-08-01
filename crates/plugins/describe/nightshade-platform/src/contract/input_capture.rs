use std::any::TypeId;
use std::collections::HashSet;

#[derive(Default)]
pub struct InputCapture {
    pub pointer_claims: HashSet<TypeId>,
    pub keyboard_claims: HashSet<TypeId>,
    pub event_consumed: bool,
}
