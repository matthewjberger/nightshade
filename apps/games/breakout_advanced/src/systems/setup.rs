use crate::components::{HalfExtent, Lean, LeanRate, Radius, Speed, Squash, Velocity};
use crate::data;
use crate::queries;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::commands::builtins::{spawn_mesh_at, spawn_sun_without_shadows};
use nightshade_graphics::prelude::*;
use nightshade_particles::contract::{ParticleTextureUpload, ParticleTextures};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::data::material::Material;

pub const WALL_MATERIAL: &str = "breakout_advanced_wall";
pub const PADDLE_MATERIAL: &str = "breakout_advanced_paddle";
pub const BALL_MATERIAL: &str = "breakout_advanced_ball";

pub fn brick_material(row: u32, armor: u32) -> String {
    format!("breakout_advanced_brick_{row}_{armor}")
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
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.22;
    settings.bloom_threshold = 0.85;
    settings.bloom_knee = 0.4;

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

pub fn upload_debris_sprite(mut textures: ResMut<ParticleTextures>) {
    let middle = data::SPRITE_TEXELS as f32 * 0.5;
    let mut pixels = Vec::with_capacity((data::SPRITE_TEXELS * data::SPRITE_TEXELS * 4) as usize);
    for row in 0..data::SPRITE_TEXELS {
        for column in 0..data::SPRITE_TEXELS {
            let reach = (column as f32 - middle).hypot(row as f32 - middle) / middle;
            let alpha = (1.0 - reach).clamp(0.0, 1.0).powf(0.9);
            pixels.extend_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }
    textures.pending.push(ParticleTextureUpload {
        slot: 0,
        rgba_data: pixels,
        width: data::SPRITE_TEXELS,
        height: data::SPRITE_TEXELS,
    });
}

fn register_materials(registry: &mut MaterialRegistry) {
    insert_material(registry, WALL_MATERIAL, flat(data::WALL_COLOR, 0.0));
    insert_material(registry, PADDLE_MATERIAL, flat(data::PADDLE_COLOR, 0.6));
    insert_material(registry, BALL_MATERIAL, flat(data::BALL_COLOR, 1.4));
    for row in 0..data::BRICK_ROWS {
        for armor in 0..data::TIER_SHADE.len() as u32 {
            let color = queries::tier_color(row, armor);
            insert_material(registry, brick_material(row, armor), flat(color, 0.5));
        }
    }
}

fn flat(color: [f32; 4], glow: f32) -> Material {
    Material {
        base_color: color,
        emissive_factor: [color[0] * glow, color[1] * glow, color[2] * glow],
        roughness: 0.55,
        metallic: 0.0,
        ..Default::default()
    }
}

fn spawn_walls(commands: &mut Commands) {
    let side = Vec2::new(
        data::WALL_HALF_THICKNESS,
        data::FIELD_HALF_HEIGHT + data::WALL_HALF_THICKNESS,
    );
    spawn_box(
        commands,
        Vec2::new(-data::FIELD_HALF_WIDTH, 0.0),
        side,
        WALL_MATERIAL,
    );
    spawn_box(
        commands,
        Vec2::new(data::FIELD_HALF_WIDTH, 0.0),
        side,
        WALL_MATERIAL,
    );
    spawn_box(
        commands,
        Vec2::new(0.0, data::FIELD_HALF_HEIGHT),
        Vec2::new(
            data::FIELD_HALF_WIDTH + data::WALL_HALF_THICKNESS,
            data::WALL_HALF_THICKNESS,
        ),
        WALL_MATERIAL,
    );
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
    commands.set(entity, Lean(0.0));
    commands.set(entity, LeanRate(0.0));
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
    commands.set(entity, Squash(0.0));
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
