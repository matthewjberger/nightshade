use crate::components::{Cell, Glow, Indoor, Prop};
use crate::data::look::{LAMP_COOL, LAMP_WARM};
use crate::resources::{Lamp, Neon};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::commands::light::spawn_light_entity;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::asset_id::MaterialId;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

pub fn spawn_lamp(
    commands: &mut Commands,
    lamp: &Lamp,
    strength: f32,
    reach: f32,
    lit: bool,
    cell: [i32; 2],
) -> Entity {
    let transform = LocalTransform {
        translation: lamp.position,
        rotation: nalgebra_glm::quat_identity(),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    commands.spawn((
        Name("Street lamp".to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        Light {
            light_type: LightType::Point,
            color: if lamp.warm { LAMP_WARM } else { LAMP_COOL },
            intensity: if lit { strength } else { 0.0 },
            range: reach,
            ..Default::default()
        },
        Glow(strength),
        Cell(cell),
        Prop(lamp.prop),
    ))
}

pub fn spawn_room_light(
    commands: &mut Commands,
    position: Vec3,
    strength: f32,
    reach: f32,
    serial: u32,
) -> Entity {
    let light = spawn_light_entity(commands, position, "Room light");
    commands.set(
        light,
        Light {
            light_type: LightType::Point,
            color: LAMP_WARM,
            intensity: strength,
            range: reach,
            ..Default::default()
        },
    );
    commands.set(light, Indoor(serial));
    light
}

pub fn spawn_neon(
    commands: &mut Commands,
    neon: &Neon,
    material: MaterialId,
    cell: [i32; 2],
) -> Entity {
    let strip = spawn_mesh_at(commands, "Cube", neon.position, neon.size);
    commands.set(strip, MaterialRef::new(material));
    commands.set(strip, Cell(cell));
    strip
}
