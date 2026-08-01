use crate::components::{HalfExtent, Radius, Speed, Velocity};
use crate::data;
use crate::queries;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::commands::builtins::{spawn_mesh_at, spawn_sun_without_shadows};
use nightshade_graphics::prelude::*;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::data::material::Material;

const WALL_MATERIAL: &str = "breakout_wall";
const PADDLE_MATERIAL: &str = "breakout_paddle";
const BALL_MATERIAL: &str = "breakout_ball";

pub fn brick_material(row: u32) -> String {
    format!("breakout_brick_{row}")
}

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::None;
    settings.show_sky = false;
    settings.clear_color = data::CLEAR_COLOR;
    settings.ibl_blend_factor = 0.0;

    spawn_sun_without_shadows(&mut commands);
    register_materials(&mut registry);

    active_camera.0 = Some(nightshade_camera::commands::scene::spawn_flat_camera(
        &mut commands,
        "Field Camera".to_string(),
        Vec3::new(0.0, 0.0, 30.0),
        data::FIELD_HALF_HEIGHT + 1.5,
        0.1,
        100.0,
    ));
    spawn_walls(&mut commands);
    spawn_paddle(&mut commands);
    spawn_ball(&mut commands);
}

fn register_materials(registry: &mut MaterialRegistry) {
    insert_material(registry, WALL_MATERIAL, flat(data::WALL_COLOR));
    insert_material(registry, PADDLE_MATERIAL, flat(data::PADDLE_COLOR));
    insert_material(registry, BALL_MATERIAL, flat(data::BALL_COLOR));
    for row in 0..data::BRICK_ROWS {
        insert_material(registry, brick_material(row), flat(queries::row_color(row)));
    }
}

fn flat(color: [f32; 4]) -> Material {
    Material {
        base_color: color,
        roughness: 0.55,
        metallic: 0.0,
        ..Default::default()
    }
}

fn spawn_walls(commands: &mut Commands) {
    for (center, half_extent) in data::WALLS {
        spawn_box(
            commands,
            Vec2::new(center[0], center[1]),
            Vec2::new(half_extent[0], half_extent[1]),
            WALL_MATERIAL,
        );
    }
}

fn spawn_paddle(commands: &mut Commands) {
    let half_extent = Vec2::new(data::PADDLE_HALF_WIDTH, data::PADDLE_HALF_HEIGHT);
    let entity = spawn_box(
        commands,
        Vec2::new(0.0, data::PADDLE_Y),
        half_extent,
        PADDLE_MATERIAL,
    );
    commands.set(entity, HalfExtent(half_extent));
    commands.set(entity, Speed(data::PADDLE_SPEED));
}

fn spawn_ball(commands: &mut Commands) {
    let position = queries::serve_position(0.0);
    let entity = spawn_mesh_at(
        commands,
        "Sphere",
        Vec3::new(position.x, position.y, 0.0),
        Vec3::new(data::BALL_RADIUS, data::BALL_RADIUS, data::BALL_RADIUS),
    );
    set_mesh_material(commands, entity, BALL_MATERIAL);
    commands.set(entity, Radius(data::BALL_RADIUS));
    commands.set(entity, Velocity(Vec2::zeros()));
}

pub fn spawn_box(
    commands: &mut Commands,
    center: Vec2,
    half_extent: Vec2,
    material: &str,
) -> Entity {
    let entity = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(center.x, center.y, 0.0),
        Vec3::new(half_extent.x * 2.0, half_extent.y * 2.0, 1.0),
    );
    set_mesh_material(commands, entity, material);
    entity
}
