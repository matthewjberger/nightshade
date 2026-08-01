pub mod hud;
use crate::resources::TowerDefenseResources;
use nightshade::prelude::*;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::*;

pub struct Scene<'a, 'c> {
    pub commands: &'a mut Commands<'c>,
    pub storage: &'a mut nightshade::ecs::Storage,
    pub registry: &'a mut nightshade_render::resources::material_registry::MaterialRegistry,
    pub text_state: &'a mut TextState,
    pub entity_registry: &'a mut nightshade_scene::resources::entity_registry::EntityRegistry,
}

pub struct SceneParts<'a> {
    commands: Commands<'a>,
    registry: &'a mut nightshade_render::resources::material_registry::MaterialRegistry,
    text_state: &'a mut TextState,
    entity_registry: &'a mut nightshade_scene::resources::entity_registry::EntityRegistry,
}

impl<'a> SceneParts<'a> {
    pub fn scene<'s>(&'s mut self, storage: &'s mut nightshade::ecs::Storage) -> Scene<'s, 'a> {
        Scene {
            commands: &mut self.commands,
            storage,
            registry: self.registry,
            text_state: self.text_state,
            entity_registry: self.entity_registry,
        }
    }
}

impl<W> nightshade::ecs::system_param::ExtractParam<W> for SceneParts<'_>
where
    W: nightshade::ecs::system_param::EcsHost + nightshade::ecs::ResourceHost,
{
    type State = ();
    type Owned = nightshade::ecs::system_param::CommandBuffer;
    type Item<'item> = SceneParts<'item>;

    fn init() -> Self::State {}

    fn resources() -> Vec<nightshade::ecs::system_param::ResourceNeed> {
        use nightshade::ecs::system_param::ResourceNeed;
        vec![
            ResourceNeed::write::<nightshade_render::resources::material_registry::MaterialRegistry>(
            ),
            ResourceNeed::write::<TextState>(),
            ResourceNeed::write::<nightshade_scene::resources::entity_registry::EntityRegistry>(),
        ]
    }

    fn extract(_state: &mut (), host: &W) -> nightshade::ecs::system_param::CommandBuffer {
        <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::extract(&mut (), host)
    }

    fn build<'a, I: Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>>(
        owned: &'a mut nightshade::ecs::system_param::CommandBuffer,
        slots: &mut I,
        resources: &'a nightshade::ecs::ResourceMap,
    ) -> SceneParts<'a> {
        SceneParts {
            commands: <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::build(
                owned,
                &mut std::iter::empty(),
                resources,
            ),
            registry: next_slot(slots),
            text_state: next_slot(slots),
            entity_registry: next_slot(slots),
        }
    }

    fn apply(_state: &mut (), owned: nightshade::ecs::system_param::CommandBuffer, host: &W) {
        <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::apply(
            &mut (),
            owned,
            host,
        );
    }

    fn flush(host: &mut W) {
        <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::flush(host);
    }

    fn access() -> nightshade::ecs::SystemAccess {
        use nightshade::ecs::{Access, SystemAccess};
        SystemAccess::empty()
            .with_resource(
                std::any::TypeId::of::<
                    nightshade_render::resources::material_registry::MaterialRegistry,
                >(),
                Access::Write,
            )
            .with_resource(std::any::TypeId::of::<TextState>(), Access::Write)
            .with_resource(
                std::any::TypeId::of::<nightshade_scene::resources::entity_registry::EntityRegistry>(),
                Access::Write,
            )
            .merge(<Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::access())
    }
}

fn next_slot<'a, T: Send + Sync + 'static>(
    slots: &mut impl Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>,
) -> &'a mut T {
    slots
        .next()
        .expect("the runner borrows one slot per resource a scene names")
        .expect("a scene names no resource it can do without")
        .into_mut::<T>()
}

pub fn build(
    game_world: &mut TowerDefenseResources,
    build: &mut nightshade_ui::commands::builder::UiBuild<'_>,
) {
    let selected = game_world.selected_tower_type;
    game_world.ui_handles = {
        let mut tree = UiTreeBuilder::new(build.context());
        let handles = hud::build(&mut tree, selected);
        tree.finish();
        handles
    };
}

pub fn poll(
    game_world: &mut TowerDefenseResources,
    frame: &nightshade_ui::resources::RetainedUiFrame,
) {
    hud::poll(game_world, frame);
}

pub fn update(
    game_world: &TowerDefenseResources,
    build: &mut nightshade_ui::commands::builder::UiBuild<'_>,
) {
    hud::update(game_world, build);
}
