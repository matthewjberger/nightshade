mod components;
mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::freecs;

freecs::ecs! {
    SandboxWorld {
        engine_entity: EngineEntity => ENGINE_ENTITY,
        enemy: Enemy => ENEMY,
    }
    Tags {}
    Events {}
    Resources {
        screen: ScreenState,
        player: PlayerState,
        character: CharacterState,
        pose: PoseState,
        ui_handles: UiHandles,
        infinite_world: InfiniteWorldState,
        rocks: RocksState,
    }
}
