use nightshade::prelude::*;

#[derive(Default, Clone)]
pub struct DemoMesh {
    pub label: &'static str,
}

#[derive(Default, Clone)]
pub struct DebugVolume {
    pub mesh: Entity,
}
