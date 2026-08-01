use crate::components::Home;
use crate::data::{Ground, Prop};
use crate::resources::CoinDashResources;
use crate::systems::materials;
use crate::systems::textures::{self, GROUND_DARK, GROUND_GREEN, GROUND_ORANGE};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_physics::contract::components::{ColliderComponent, RigidBodyComponent};
use nightshade_render::resources::mesh_state::MeshRenderState;

pub const ARENA: f32 = 55.0;
pub const ROUND_SECONDS: f32 = 50.0;
pub const PROP_COUNT: usize = 64;
pub const COIN_COUNT: usize = 14;
pub const PICKUP_RADIUS: f32 = 2.5;
pub const COIN_HEIGHT: f32 = 1.4;
pub const PLAYER_SPEED: f32 = 9.0;
const GROUND_TILING: f32 = 14.0;

pub struct Theme {
    pub ground: &'static str,
    pub hill: [f32; 4],
    pub foliage: [f32; 4],
}

pub const THEMES: [Theme; 3] = [
    Theme {
        ground: GROUND_GREEN,
        hill: rgb(0.45, 0.35, 0.22),
        foliage: GREEN,
    },
    Theme {
        ground: GROUND_ORANGE,
        hill: rgb(0.72, 0.56, 0.32),
        foliage: rgb(0.5, 0.6, 0.25),
    },
    Theme {
        ground: GROUND_DARK,
        hill: GRAY,
        foliage: rgb(0.62, 0.72, 0.82),
    },
];

pub fn scatter(rng: &mut fastrand::Rng, min_radius: f32, max_radius: f32) -> (f32, f32) {
    let angle = rng.f32() * std::f32::consts::TAU;
    let radius = min_radius + rng.f32() * (max_radius - min_radius);
    (angle.cos() * radius, angle.sin() * radius)
}

#[derive(Clone, Copy)]
enum PropKind {
    Hill,
    Post,
    Tree,
    Rock,
}

impl PropKind {
    fn mesh(self) -> &'static str {
        match self {
            PropKind::Hill => "Cube",
            PropKind::Post => "Cylinder",
            PropKind::Tree => "Cone",
            PropKind::Rock => "Sphere",
        }
    }

    fn collider(self, scale: Vec3) -> ColliderComponent {
        match self {
            PropKind::Hill => {
                ColliderComponent::new_cuboid(scale.x * 0.5, scale.y * 0.5, scale.z * 0.5)
            }
            PropKind::Post | PropKind::Tree => {
                ColliderComponent::new_cylinder(scale.y * 0.5, scale.x * 0.5)
            }
            PropKind::Rock => ColliderComponent::new_ball(scale.x),
        }
    }
}

pub fn generate_world(resources: &mut CoinDashResources, commands: &mut Commands) {
    let round = resources.round.round;
    let theme = &THEMES[((round - 1) % 3) as usize];

    let ground_name = format!("coin_dash_{}", theme.ground);
    let ground_material = textures::tiled_material(theme.ground, GROUND_TILING);
    commands.queue(move |storage, resources| {
        let grounds: Vec<Entity> = storage.query_tag_type::<Ground>().collect();
        for ground in grounds {
            nightshade::ecs::with_commands_over(storage, resources, |_, _, commands| {
                materials::attach(
                    commands,
                    ground,
                    ground_name.clone(),
                    ground_material.clone(),
                );
            });
        }

        let props: Vec<Entity> = storage.query_tag_type::<Prop>().collect();
        for prop in props {
            let mut targets = resources
                .res::<nightshade::ecs::HierarchyIndex>()
                .descendants(prop);
            targets.insert(0, prop);
            storage.despawn_entities(&targets);
        }
    });

    let rng = &mut resources.round.rng;
    for _ in 0..PROP_COUNT {
        spawn_prop(commands, rng, theme);
    }

    let homes: Vec<Vec3> = (0..COIN_COUNT)
        .map(|_| {
            let (x, z) = scatter(rng, 5.0, ARENA * 0.85);
            Vec3::new(x, COIN_HEIGHT, z)
        })
        .collect();

    commands.queue(move |storage, resources| {
        let coins: Vec<Entity> = storage.query_entities::<(Home,)>().collect();
        for (coin, home) in coins.into_iter().zip(homes) {
            storage.set(coin, Home(home));
        }
        resources.res_mut::<MeshRenderState>().full_rebuild_needed = true;
    });
}

fn spawn_prop(commands: &mut Commands, rng: &mut fastrand::Rng, theme: &Theme) {
    let (x, z) = scatter(rng, 8.0, ARENA);
    let (kind, scale, color) = match rng.u32(..4) {
        0 => {
            let width = 2.0 + rng.f32() * 5.0;
            let height = 1.0 + rng.f32() * 3.0;
            (PropKind::Hill, Vec3::new(width, height, width), theme.hill)
        }
        1 => {
            let height = 3.0 + rng.f32() * 6.0;
            (PropKind::Post, Vec3::new(1.2, height, 1.2), STEEL)
        }
        2 => {
            let width = 1.5 + rng.f32() * 2.0;
            let height = 3.0 + rng.f32() * 4.0;
            (
                PropKind::Tree,
                Vec3::new(width, height, width),
                theme.foliage,
            )
        }
        _ => {
            let radius = 1.0 + rng.f32() * 2.5;
            (PropKind::Rock, Vec3::new(radius, radius, radius), GRAY)
        }
    };

    let position = Vec3::new(x, scale.y * 0.5, z);
    let entity = spawn_mesh_at(commands, kind.mesh(), position, scale);
    materials::attach(
        commands,
        entity,
        format!("coin_dash_prop_{}", entity.id),
        textures::solid_material(color, 0.85, 0.0),
    );
    commands.add_tag_type::<Prop>(entity);
    commands.set(
        entity,
        RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
    );
    commands.set(
        entity,
        kind.collider(scale)
            .with_friction(0.8)
            .with_restitution(0.1),
    );
}
