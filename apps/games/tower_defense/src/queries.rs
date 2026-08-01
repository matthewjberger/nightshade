pub mod theme;

use crate::commands::Scene;
use nightshade::prelude::Entity;

pub fn read_or_default<T: Copy + Default + Send + Sync + 'static>(
    scene: &Scene<'_, '_>,
    entity: Entity,
) -> T {
    scene.storage.get::<T>(entity).copied().unwrap_or_default()
}
