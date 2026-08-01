use crate::components::{
    Age, Air, Boat, Count, Fall, Food, Grounded, Held, Motion, Phase, Pitch, Spawn, Submerged,
    Velocity, WatchSpeed, Yaw, Zoom, ZoomWanted,
};
use crate::queries::{boat_mesh, glider_mesh};
use crate::resources::{Fireworks, Resumed, Shell};
use crate::theme;
use blocker_blocks::data::items::Item;
use blocker_blocks::queries::{ground_height, item_scale};
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::commands::dress_the_body;
use blocker_figure::components::{Eye, Gaze, Soar};
use blocker_figure::resources::Hero;
use blocker_world::queries::dry_spot;
use blocker_world::resources::Generation;
use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::Entity;
use nightshade::prelude::Commands;
use nightshade_camera::prelude::{ActiveCamera, Camera, PerspectiveCamera, Projection};
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_health::{Armor, Bounty, Health, MaxHealth, PendingDamage, Shield};
use nightshade_particles::contract::ParticleEmitter;
use nightshade_random::Random;
use nightshade_random::commands::{between, index};
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::lines::{Line, Lines};
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_scene::prelude::{GlobalTransform, Lifetime, LocalTransform, Name, Visibility};

pub fn install_the_craft(cache: &mut MeshCache) {
    mesh_cache_insert(cache, theme::BOAT_MESH.to_string(), boat_mesh());
    mesh_cache_protect(cache, theme::BOAT_MESH.to_string());
    mesh_cache_insert(cache, theme::GLIDER_MESH.to_string(), glider_mesh());
    mesh_cache_protect(cache, theme::GLIDER_MESH.to_string());
}

pub fn spawn_the_camera(
    commands: &mut Commands,
    active: &mut ActiveCamera,
    fov_degrees: f32,
    near: f32,
    far: f32,
) -> Entity {
    let camera = commands.spawn((
        Name("Player Camera".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: fov_degrees.to_radians(),
                z_far: Some(far),
                z_near: near,
            }),
            smoothing: None,
        },
    ));
    active.0 = Some(camera);
    camera
}

pub fn spawn_the_hero(commands: &mut Commands, hero: &mut Hero, look: Option<usize>) -> Entity {
    let body = commands.spawn((
        Name("You".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
    ));
    if let Some(look) = look {
        dress_the_body(
            commands,
            body,
            look,
            blocker_figure::data::rig::FIGURE_SCALE,
        );
        commands.set(body, Soar(0.0));
        commands.set(body, Visibility { visible: false });
    }
    commands.set(body, Health(crate::data::body::FULL_HEALTH));
    commands.set(body, MaxHealth(crate::data::body::FULL_HEALTH));
    commands.set(body, Shield(0.0));
    commands.set(body, Armor(0.0));
    commands.set(body, PendingDamage(0.0));
    commands.set(body, Bounty(0));
    commands.set(body, Food(crate::data::body::FULL_FOOD));
    commands.set(body, Air::default());
    commands.set(body, Velocity::default());
    commands.set(body, Yaw::default());
    commands.set(body, Pitch::default());
    commands.set(body, Eye::default());
    commands.set(body, Gaze::default());
    commands.set(body, Zoom::default());
    commands.set(body, ZoomWanted::default());
    commands.set(body, Motion::Walk);
    commands.set(body, Grounded::default());
    commands.set(body, Submerged::default());
    commands.set(body, Fall::default());
    commands.set(body, Spawn::default());
    commands.set(body, WatchSpeed::default());
    hero.0 = Some(body);
    body
}

pub fn spawn_a_boat(commands: &mut Commands, place: Vec3, yaw: f32) -> Entity {
    let mut transform = LocalTransform::from_translation(place);
    transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    let extent = theme::BOAT_EXTENT;
    let entity = commands.spawn((
        Name("Boat".to_string()),
        Boat(yaw),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(theme::BOAT_MESH),
        MaterialRef::default(),
        Visibility { visible: true },
        BoundingVolume::new(
            OrientedBoundingBox::from_aabb(Vec3::zeros(), extent),
            extent.norm(),
        ),
    ));
    set_mesh_material(commands, entity, blocker_blocks::data::atlas::FLAT_MATERIAL);
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

pub fn set_watching(
    motion: &mut Motion,
    velocity: &mut Velocity,
    watch: &mut WatchSpeed,
    watching: bool,
) {
    *motion = if watching {
        Motion::Watch
    } else {
        Motion::Walk
    };
    velocity.0 = Vec3::zeros();
    watch.0 = theme::WATCH_SPEED;
}

pub fn spawn_glider_wing(commands: &mut Commands) -> Entity {
    let wing = commands.spawn((
        Name("Glider".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        RenderMesh::new(theme::GLIDER_MESH),
        MaterialRef::default(),
        CastsShadow,
        Visibility { visible: false },
        BoundingVolume::new(
            OrientedBoundingBox::from_aabb(theme::GLIDER_LOW, theme::GLIDER_HIGH),
            theme::GLIDER_LOW
                .abs()
                .sup(&theme::GLIDER_HIGH.abs())
                .norm(),
        ),
    ));
    set_mesh_material(commands, wing, blocker_blocks::data::atlas::FLAT_MATERIAL);
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), wing);
    });
    wing
}

pub fn launch_firework(
    commands: &mut Commands,
    fireworks: &mut Fireworks,
    random: &mut Random,
    launch: Vec3,
) {
    let velocity = Vec3::new(
        between(random, -1.25, 1.25),
        theme::FIREWORK_SPEED + between(random, 0.0, 4.0),
        between(random, -1.25, 1.25),
    );
    let color = theme::FIREWORK_COLORS[index(random, theme::FIREWORK_COLORS.len())];
    let entity = commands.spawn(());
    let mut trail = ParticleEmitter::firework_shell(launch, velocity);
    trail.size_start = 0.08;
    trail.size_end = 0.0;
    commands.set(entity, trail);
    commands.set(entity, Lifetime { remaining: 8.0 });
    fireworks.shells.push(Shell {
        entity,
        position: launch,
        velocity,
        fuse: theme::FIREWORK_FUSE + between(random, 0.0, 0.4),
        color,
    });
}

pub fn spawn_drop(
    commands: &mut Commands,
    item: Item,
    count: u32,
    position: Vec3,
    velocity: Vec3,
    phase: f32,
) -> Entity {
    let transform = LocalTransform {
        translation: position,
        scale: Vec3::repeat(item_scale(item)),
        ..Default::default()
    };
    let entity = commands.spawn((
        Name(format!("Dropped {}", item.name())),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(item.mesh_name()),
        MaterialRef::default(),
        CastsShadow,
        Visibility { visible: true },
        Held(item),
        Count(count),
        Velocity(velocity),
        Age(0.0),
        Phase(phase),
    ));
    set_mesh_material(commands, entity, item.material_name());
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

pub fn spawn_outline(commands: &mut Commands) -> Entity {
    let corners = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    let edges = [
        (corners(0.0, 0.0, 0.0), corners(1.0, 0.0, 0.0)),
        (corners(1.0, 0.0, 0.0), corners(1.0, 0.0, 1.0)),
        (corners(1.0, 0.0, 1.0), corners(0.0, 0.0, 1.0)),
        (corners(0.0, 0.0, 1.0), corners(0.0, 0.0, 0.0)),
        (corners(0.0, 1.0, 0.0), corners(1.0, 1.0, 0.0)),
        (corners(1.0, 1.0, 0.0), corners(1.0, 1.0, 1.0)),
        (corners(1.0, 1.0, 1.0), corners(0.0, 1.0, 1.0)),
        (corners(0.0, 1.0, 1.0), corners(0.0, 1.0, 0.0)),
        (corners(0.0, 0.0, 0.0), corners(0.0, 1.0, 0.0)),
        (corners(1.0, 0.0, 0.0), corners(1.0, 1.0, 0.0)),
        (corners(1.0, 0.0, 1.0), corners(1.0, 1.0, 1.0)),
        (corners(0.0, 0.0, 1.0), corners(0.0, 1.0, 1.0)),
    ];
    let inflate = 0.003;
    let lines = edges
        .iter()
        .map(|(start, end)| Line {
            start: start.map(|value| value * (1.0 + 2.0 * inflate) - inflate),
            end: end.map(|value| value * (1.0 + 2.0 * inflate) - inflate),
            color: theme::OUTLINE_COLOR,
        })
        .collect();
    commands.spawn((
        Name("Block Outline".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Lines::new(lines),
        Visibility { visible: false },
    ))
}

pub fn spawn_crack(commands: &mut Commands) -> Entity {
    let entity = commands.spawn((
        Name("Block Crack".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        RenderMesh::new(blocker_blocks::data::atlas::CRACK_MESH),
        MaterialRef::default(),
        Visibility { visible: false },
    ));
    set_mesh_material(
        commands,
        entity,
        blocker_blocks::data::atlas::CUTOUT_MATERIAL,
    );
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

pub type Standing<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut LocalTransform,
        &'item mut Yaw,
        &'item mut Pitch,
        &'item mut Velocity,
        &'item mut Spawn,
        &'item mut Fall,
    ),
>;

fn drop_the_body(bodies: &mut Standing<'_, '_>, hero: Entity, place: Vec3) {
    bodies.for_entity(hero, |(transform, _, pitch, velocity, spawn, fall)| {
        transform.translation = place;
        velocity.0 = Vec3::zeros();
        spawn.0 = place;
        fall.0 = place.y;
        pitch.0 = theme::START_PITCH;
    });
}

fn resume_the_body(
    bodies: &mut Standing<'_, '_>,
    hero: Entity,
    place: Vec3,
    home: Vec3,
    yaw_turn: f32,
    pitch_tilt: f32,
) {
    bodies.for_entity(hero, |(transform, yaw, pitch, velocity, spawn, fall)| {
        transform.translation = place;
        velocity.0 = Vec3::zeros();
        spawn.0 = home;
        yaw.0 = yaw_turn;
        pitch.0 = pitch_tilt;
        fall.0 = place.y;
    });
}

fn stand_the_body(bodies: &mut Standing<'_, '_>, hero: Entity, place: Vec3) {
    bodies.for_entity(hero, |(transform, yaw, pitch, velocity, spawn, fall)| {
        transform.translation = place;
        spawn.0 = place;
        velocity.0 = Vec3::zeros();
        fall.0 = place.y;
        yaw.0 = 0.0;
        pitch.0 = theme::START_PITCH;
    });
}

pub fn float_the_player(
    bodies: &mut Standing<'_, '_>,
    hero: Entity,
    generation: &mut Generation,
    resumed: Option<Resumed>,
    gate: [i32; 2],
) {
    match resumed {
        Some(found) => {
            generation.spawn = Some((
                found.position.x.floor() as i32,
                found.position.z.floor() as i32,
            ));
            resume_the_body(
                bodies,
                hero,
                found.position,
                found.home,
                found.yaw,
                found.pitch,
            );
        }
        None => {
            generation.spawn = Some((gate[0], gate[1]));
            drop_the_body(
                bodies,
                hero,
                vec3(
                    gate[0] as f32 + 0.5,
                    blocker_world::theme::SPAWN_SKY,
                    gate[1] as f32 + 0.5,
                ),
            );
        }
    }
}

pub(crate) fn set_the_player_down(
    bodies: &mut Standing<'_, '_>,
    hero: Entity,
    world: &VoxelWorld,
    generation: &mut Generation,
    resumed: Option<Resumed>,
) {
    if let Some(found) = resumed {
        let Some(top) = ground_height(
            world,
            found.position.x.floor() as i32,
            found.position.z.floor() as i32,
        ) else {
            return;
        };
        if found.position.y >= top as f32 {
            return;
        }
        let place = vec3(found.position.x, top as f32 + 1.0, found.position.z);
        stand_the_body(bodies, hero, place);
        return;
    }
    let (start_x, start_z) = generation.spawn.unwrap_or((0, 0));
    let (spawn_x, spawn_z) = dry_spot(world, start_x, start_z);
    let top =
        ground_height(world, spawn_x, spawn_z).unwrap_or(blocker_world::data::world::SEA_LEVEL) + 1;
    generation.spawn = Some((spawn_x, spawn_z));
    stand_the_body(
        bodies,
        hero,
        vec3(spawn_x as f32 + 0.5, top as f32, spawn_z as f32 + 0.5),
    );
}
