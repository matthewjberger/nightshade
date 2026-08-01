use crate::commands::creatures::{
    spawn_animals, spawn_floating_pool, spawn_givers, spawn_guards, spawn_landmarks, spawn_mobs,
    spawn_people, spawn_pickups, spawn_ring, spawn_townsfolk,
};
use crate::commands::plant;
use crate::commands::sky::{spawn_birds, spawn_clouds};
use crate::components::{
    Age, Aura, Beat, Block, Drift, Fade, GlowBlock, Health, Held, Orbit, Pickup, Plate, Progress,
    Projectile, Rooted, SignLabel, Span, Stage, Watered,
};
use crate::data::{CRYPT, CropKind, DEEP, FARM, TOWN};
use crate::queries::zone_center;
use crate::resources::{
    Chests, Digging, Draft, Effects, Fireworks, Flashes, Generation, Hero, HudText, Inventory,
    Journal, PlaceSpot, Places, Player, Save, Screen, Sheet, VoxelWorld, Weather,
};
use crate::theme;
use nalgebra_glm::{IVec3, Vec3, quat_angle_axis, quat_rotate_vec3};
use nightshade::ecs::{ChildOf, Entity, ResourceMap, Storage};
use nightshade::prelude::*;
use nightshade_camera::prelude::{ActiveCamera, Camera};
use nightshade_camera_controller::prelude::ThirdPersonCamera;
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::Time;
use nightshade_random::prelude::*;
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
    next.pending = Some(Screen::Creation);
}

pub fn people_the_world(
    generation: Res<Generation>,
    world: Res<VoxelWorld>,
    sheet: Res<Sheet>,
    save: Res<Save>,
    mut places: ResMut<Places>,
    mut hero: ResMut<Hero>,
    mut flashes: ResMut<Flashes>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
) {
    if generation.spawn.is_none() || generation.zones.is_empty() {
        return;
    }
    if !places.laid {
        places.town = generation.zones[TOWN].center;
        places.spots = generation
            .zones
            .iter()
            .enumerate()
            .map(|(index, zone)| PlaceSpot {
                center: zone_center(zone),
                radius: zone.radius,
                floor: matches!(index, CRYPT | DEEP).then_some(zone.level as f32 + 1.0),
            })
            .collect();
        places.laid = true;
    }
    if places.peopled {
        return;
    }
    places.peopled = true;
    let hero = &mut *hero;
    let town = generation.zones[TOWN];
    let sky_center = Vec3::new(
        town.center.0 as f32,
        town.level as f32,
        town.center.1 as f32,
    );
    spawn_clouds(&mut commands, sky_center, &mut hero.random);
    spawn_birds(&mut commands, sky_center, &mut hero.random);
    if save.found.is_none() {
        sow_the_farm(&generation, &world, &sheet, &mut hero.random, &mut commands);
    }
    spawn_landmarks(&generation, &places, &mut commands);
    spawn_mobs(
        &mut hero.random,
        &generation,
        &places,
        &mut text_state,
        &mut commands,
    );
    spawn_people(
        &mut hero.random,
        &generation,
        &places,
        &mut text_state,
        &mut commands,
    );
    spawn_guards(&generation, &places, &mut text_state, &mut commands);
    spawn_townsfolk(&generation, &places, &mut text_state, &mut commands);
    spawn_animals(&mut hero.random, &generation, &places, &mut commands);
    spawn_givers(&generation, &places, &mut text_state, &mut commands);
    spawn_pickups(&mut hero.random, &generation, &places, &mut commands);
    if flashes.pool.is_empty() {
        flashes.pool = spawn_floating_pool(&mut text_state, &mut commands);
    }
    if hero.ring.is_none() {
        hero.ring = Some(spawn_ring(&mut commands));
    }
}

fn sow_the_farm(
    generation: &Generation,
    world: &VoxelWorld,
    sheet: &Sheet,
    random: &mut Random,
    commands: &mut Commands,
) {
    let farm = generation.zones[FARM];
    for u in (-12i32..=-3).chain(3..=12) {
        for v in -7i32..=7 {
            if v.rem_euclid(3) == 1 || (u + v).rem_euclid(2) != 0 {
                continue;
            }
            let kind = if u < 0 {
                CropKind::Wheat
            } else {
                CropKind::Carrot
            };
            let block = IVec3::new(farm.center.0 + u, farm.level, farm.center.1 + v);
            let entity = plant(commands, world, sheet, kind, block);
            commands.set(entity, kind);
            commands.set(entity, Block(block));
            commands.set(entity, Progress(0.2 + random.f32() * 0.7));
            commands.set(entity, Stage(0));
            commands.set(entity, Watered(true));
        }
    }
}

pub fn look_over_the_world(
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut active_camera: ResMut<ActiveCamera>,
    mut cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    let Some(camera) = player.camera else {
        return;
    };
    if active_camera.0 != Some(camera) {
        active_camera.0 = Some(camera);
    }
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
    gone.extend(storage.query_entities::<(CropKind,)>());
    gone.extend(storage.query_entities::<(GlowBlock,)>());
    gone.extend(storage.query_entities::<(SignLabel,)>());
    gone.extend(storage.query_entities::<(Health,)>());
    gone.extend(storage.query_entities::<(Rooted,)>());
    gone.extend(storage.query_entities::<(Pickup,)>());
    gone.extend(storage.query_entities::<(Plate,)>());
    gone.extend(storage.query_entities::<(Fade,)>());
    gone.extend(storage.query_entities::<(Span,)>());
    gone.extend(storage.query_entities::<(Projectile,)>());
    gone.extend(storage.query_entities::<(Aura,)>());
    gone.extend(storage.query_entities::<(Beat,)>());
    gone.extend(storage.query_entities::<(Age,)>());
    gone.extend(storage.query_entities::<(Drift,)>());
    gone.extend(storage.query_entities::<(Orbit,)>());
    gone.extend(storage.query_entities::<(ThirdPersonCamera,)>());
    gone.extend(storage.query_entities::<(ParticleEmitter,)>());
    gone.extend(resources.res::<Flashes>().pool.iter().copied());
    if let Some(ring) = resources.res::<Hero>().ring {
        gone.push(ring);
    }
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
    resources.res_mut::<ActiveCamera>().0 = camera;
    *resources.res_mut::<Generation>() = Generation::default();
    *resources.res_mut::<Chests>() = Chests::default();
    *resources.res_mut::<Effects>() = Effects::default();
    *resources.res_mut::<Fireworks>() = Fireworks::default();
    *resources.res_mut::<Weather>() = Weather::default();
    *resources.res_mut::<Inventory>() = Inventory::default();
    *resources.res_mut::<Player>() = Player::default();
    resources.res_mut::<Player>().camera = camera;
    *resources.res_mut::<Journal>() = Journal::default();
    *resources.res_mut::<Places>() = Places::default();
    *resources.res_mut::<Flashes>() = Flashes::default();
    *resources.res_mut::<HudText>() = HudText::default();
    let hero = resources.res_mut::<Hero>();
    hero.player = None;
    hero.camera = None;
    hero.ring = None;
    hero.named = false;
    let save = resources.res_mut::<Save>();
    save.found = None;
    save.hero = None;
    save.autosave.ask();
}
