use crate::components::{Angry, Beast, Flinch, Goal, Lair, Roam, Shove, Swing};
use crate::data::beasts::{BEASTS, Form};
use crate::queries::{beast_material, beast_snout};
use crate::theme;
use blocker_figure::commands::dress_the_body;
use blocker_figure::components::{Gait, Heading, Limb, Owner, Pace};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::Entity;
use nightshade::prelude::Commands;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::prelude::spawn_mesh_at;
use nightshade_health::commands::hurt;
use nightshade_health::{Armor, Bounty, Health, MaxHealth, PendingDamage, Shield};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::prelude::ChildOf;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

pub fn spawn_beast(
    commands: &mut Commands,
    position: Vec3,
    heading: f32,
    kind: usize,
    lair: Vec2,
) -> Entity {
    let beast = &BEASTS[kind];
    let size = beast.scale * theme::BEAST_SCALE;
    let coat = beast_material(kind);
    let snout = beast_snout(kind);
    let body = commands.spawn((
        Name(beast.name.to_string()),
        LocalTransform {
            translation: position,
            rotation: nalgebra_glm::quat_angle_axis(heading, &Vec3::y()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
    ));
    match beast.form {
        Form::Figure { look, fit } => dress_the_body(commands, body, look, fit),
        Form::Cube => {
            build_the_cubes(commands, body, size, &coat, &snout);
            commands.set(body, Gait(0.0));
            commands.set(body, Pace(0.0));
        }
    }
    commands.set(body, Beast(kind));
    commands.set(body, Health(beast.health));
    commands.set(body, MaxHealth(beast.health));
    commands.set(body, Shield(0.0));
    commands.set(body, Armor(0.0));
    commands.set(body, PendingDamage(0.0));
    commands.set(body, Bounty(beast.renown));
    commands.set(body, Heading(heading));
    commands.set(body, Roam(0.0));
    commands.set(body, Swing(0.0));
    commands.set(body, Angry(0.0));
    commands.set(body, Flinch(0.0));
    commands.set(body, Goal(lair));
    commands.set(body, Lair(lair));
    commands.set(body, Shove::default());
    body
}

fn build_the_cubes(commands: &mut Commands, body: Entity, size: f32, coat: &str, snout: &str) {
    beast_cube(
        commands,
        body,
        Vec3::new(0.0, 0.55, 0.0) * size,
        Vec3::new(0.8, 0.7, 1.3) * size,
        coat,
    );
    beast_cube(
        commands,
        body,
        Vec3::new(0.0, 0.65, -0.85) * size,
        Vec3::new(0.5, 0.45, 0.5) * size,
        coat,
    );
    beast_cube(
        commands,
        body,
        Vec3::new(0.0, 0.5, -1.15) * size,
        Vec3::new(0.22, 0.2, 0.2) * size,
        snout,
    );
    beast_cube(
        commands,
        body,
        Vec3::new(0.0, 0.62, 0.85) * size,
        Vec3::new(0.12, 0.12, 0.5) * size,
        coat,
    );
    for side in [-0.28, 0.28] {
        for depth in [-0.45, 0.45] {
            let sign = if (side < 0.0) == (depth < 0.0) {
                1.0
            } else {
                -1.0
            };
            let leg = beast_cube(
                commands,
                body,
                Vec3::new(side, 0.25, depth) * size,
                Vec3::new(0.18, 0.5, 0.18) * size,
                coat,
            );
            commands.set(leg, Limb(sign));
            commands.set(leg, Owner(body));
        }
    }
}

pub fn insert_beast_materials(registry: &mut MaterialRegistry) {
    for (index, beast) in BEASTS.iter().enumerate() {
        if !matches!(beast.form, Form::Cube) {
            continue;
        }
        insert_material(
            registry,
            beast_material(index),
            Material {
                base_color: beast.color,
                roughness: 0.8,
                metallic: 0.1,
                ..Default::default()
            },
        );
        insert_material(
            registry,
            beast_snout(index),
            Material {
                base_color: beast.snout,
                roughness: 0.8,
                metallic: 0.1,
                ..Default::default()
            },
        );
    }
}

fn beast_cube(
    commands: &mut Commands,
    body: Entity,
    position: Vec3,
    scale: Vec3,
    material: &str,
) -> Entity {
    let child = spawn_mesh_at(commands, "Cube", position, scale);
    set_mesh_material(commands, child, material);
    commands.set(child, ChildOf(body));
    child
}

pub fn hurt_the_body(commands: &mut Commands, body: Entity, amount: f32) {
    commands.queue(move |storage, _| {
        if let Some(pending) = storage.get_mut::<PendingDamage>(body) {
            hurt(pending, amount);
        }
    });
}
