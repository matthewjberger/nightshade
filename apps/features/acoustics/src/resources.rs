use nightshade::prelude::{Entity, Vec3};

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub panel: Entity,
    pub door_row: Entity,
    pub path_row: Entity,
    pub room_row: Entity,
    pub opening_row: Entity,
}

#[derive(Default)]
pub struct AcousticsDemo {
    pub source: Entity,
    pub door: Entity,
    pub markers: Entity,
    pub paths: Entity,
    pub rain: Entity,
    pub camera: Entity,
    pub head: Entity,
    pub chest: Entity,
    pub stand: Vec3,
    pub eye: Vec3,
    pub chasing: bool,
    pub shut: bool,
    pub showing: bool,
    pub flowing: bool,
    pub front: f32,
    pub drawn: u64,
    #[cfg(not(target_arch = "wasm32"))]
    pub playing: bool,
    pub built: bool,
    pub handles: UiHandles,
}
