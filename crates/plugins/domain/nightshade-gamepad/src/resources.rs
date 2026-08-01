#[cfg(not(target_os = "android"))]
#[derive(Default)]
pub struct GamepadBackend {
    pub gilrs: std::sync::Mutex<Option<gilrs::Gilrs>>,
    pub gamepad: Option<gilrs::GamepadId>,
}

#[cfg(target_os = "android")]
#[derive(Default)]
pub struct GamepadBackend {
    pub device: Option<i32>,
    pub axes: Vec<(crate::contract::GamepadAxis, f32)>,
    pub buttons: std::collections::HashSet<crate::contract::GamepadButton>,
    pub hat: std::collections::HashSet<crate::contract::GamepadButton>,
}
