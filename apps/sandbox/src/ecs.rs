mod components;
mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::freecs;

freecs::dynamic_schema! {
    pub fn register_sandbox_components {
        enemy: Enemy => ENEMY,
    }
}

#[derive(Default)]
pub struct SandboxResources {
    pub screen: ScreenState,
    pub player: PlayerState,
    pub character: CharacterState,
    pub pose: PoseState,
    pub ragdoll: RagdollState,
    pub ui_handles: UiHandles,
    pub infinite_world: InfiniteWorldState,
    pub rocks: RocksState,
}
