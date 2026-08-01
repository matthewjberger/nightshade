use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.world
            .resource_or_insert_default::<crate::contract::render_data::SpriteFrame>();
        app.world
            .resource_or_insert_default::<crate::resources::SpriteAtlases>();
        app.add_system(
            Stage::FrameUpdate,
            crate::systems::animate::advance_sprite_animators,
        );
        app.add_system(Stage::Render, crate::systems::gather::gather_sprites_system);
    }
}
