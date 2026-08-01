use crate::commands::{populate_now, toast_show};
use crate::components::{Boat, GlowBlock, Held, NpcName, SignLabel};
use crate::data::{CropKind, MonsterKind};
use crate::resources::{
    Chests, Digging, Draft, Effects, Fireworks, Generation, Hand, Horde, Inventory, Player, Save,
    Screen, Sheet, Toast, VoxelWorld, Weather,
};
use crate::systems::generation::stand_the_player_up;
use crate::tags::HeldItem;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis, quat_rotate_vec3};
use nightshade::ecs::{ChildOf, Entity, ResourceMap, Storage};
use nightshade::prelude::*;
use nightshade_camera::prelude::Camera;
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::Time;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::LocalTransform;
use nightshade_state::prelude::*;
use nightshade_text::prelude::TextState;
use nightshade_voxels::prelude::reset_the_world;
use std::collections::HashSet;

pub fn start_the_world(
    mut draft: ResMut<Draft>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
) {
    if !draft.asked {
        return;
    }
    draft.asked = false;
    draft.building = true;
    commands.queue(|storage: &mut Storage, resources: &mut ResourceMap| {
        clear_the_world(storage, resources);
    });
    next.pending = Some(Screen::Playing);
}

pub fn raise_the_new_world(
    mut draft: ResMut<Draft>,
    mut player: ResMut<Player>,
    mut world: ResMut<VoxelWorld>,
    mut generation: ResMut<Generation>,
    mut renderer_state: ResMut<RendererState>,
    mut text_state: ResMut<TextState>,
    mut toast: ResMut<Toast>,
    sheet: Res<Sheet>,
    mut commands: Commands,
) {
    if !draft.building {
        return;
    }
    draft.building = false;
    generation.seed = draft.seed;
    generation.plan = draft.plan;
    stand_the_player_up(&mut player, &mut world, &mut generation, None);
    populate_now(&world, &generation, &sheet, &mut text_state, &mut commands);
    renderer_state.day_night.hour = generation.plan.start.hour();
    toast_show(&mut toast, "A new world");
}

pub fn look_over_the_world(
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    let Some(camera) = player.camera else {
        return;
    };
    player.yaw += time.delta_time * theme::TITLE_TURN;
    let rotation =
        quat_angle_axis(player.yaw, &Vec3::y()) * quat_angle_axis(theme::TITLE_PITCH, &Vec3::x());
    let eye = player.position + Vec3::new(0.0, theme::EYE_HEIGHT, 0.0);
    player.eye = eye;
    player.forward = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
    cameras.for_entity(camera, |(_, transform)| {
        transform.translation = eye;
        transform.rotation = rotation;
    });
}

fn clear_the_world(storage: &mut Storage, resources: &mut ResourceMap) {
    let mut gone: Vec<Entity> = Vec::new();
    gone.extend(storage.query_entities::<(Held,)>());
    gone.extend(storage.query_entities::<(NpcName,)>());
    gone.extend(storage.query_entities::<(CropKind,)>());
    gone.extend(storage.query_entities::<(GlowBlock,)>());
    gone.extend(storage.query_entities::<(SignLabel,)>());
    gone.extend(storage.query_entities::<(Boat,)>());
    gone.extend(storage.query_entities::<(MonsterKind,)>());
    gone.extend(
        storage
            .query_tag_type::<HeldItem>()
            .collect::<Vec<Entity>>(),
    );
    gone.extend(storage.query_entities::<(ParticleEmitter,)>());
    let held: HashSet<Entity> = gone.iter().copied().collect();
    let children: Vec<Entity> = storage
        .query_entities::<(ChildOf,)>()
        .collect::<Vec<Entity>>()
        .into_iter()
        .filter(|entity| {
            storage
                .get::<ChildOf>(*entity)
                .is_some_and(|parent| held.contains(&parent.0))
        })
        .collect();
    gone.extend(children);
    let mut once: HashSet<Entity> = HashSet::new();
    gone.retain(|entity| once.insert(*entity));
    despawn_entities_with_cache_cleanup(storage, resources, &gone);

    reset_the_world::<Digging>(storage, resources);
    let camera = resources.res::<Player>().camera;
    *resources.res_mut::<Generation>() = Generation::default();
    *resources.res_mut::<Horde>() = Horde::default();
    *resources.res_mut::<Chests>() = Chests::default();
    *resources.res_mut::<Effects>() = Effects::default();
    *resources.res_mut::<Fireworks>() = Fireworks::default();
    *resources.res_mut::<Weather>() = Weather::default();
    *resources.res_mut::<Hand>() = Hand::default();
    *resources.res_mut::<Inventory>() = Inventory::default();
    *resources.res_mut::<Player>() = Player::default();
    resources.res_mut::<Player>().camera = camera;
    let save = resources.res_mut::<Save>();
    save.found = None;
    save.autosave.ask();
}
