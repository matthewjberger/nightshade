use crate::commands::spawn_drop;
use crate::data::{BlockKind, Item};
use crate::resources::VoxelWorld;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;

pub fn drop_the_fallen(time: Res<Time>, mut world: ResMut<VoxelWorld>, mut commands: Commands) {
    if world.fallen.is_empty() {
        return;
    }
    let phase = time.uptime_milliseconds as f32 * 0.001;
    for (position, block) in std::mem::take(&mut world.fallen) {
        let Some(drop) = BlockKind::from_index(block).drop() else {
            continue;
        };
        let center = Vec3::new(
            position.x as f32 + 0.5,
            position.y as f32 + 0.5,
            position.z as f32 + 0.5,
        );
        spawn_drop(
            &mut commands,
            Item::Block(drop),
            1,
            center,
            Vec3::new(0.0, 1.5, 0.0),
            phase,
        );
    }
}
