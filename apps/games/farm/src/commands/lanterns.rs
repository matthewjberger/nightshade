use crate::components::{NightOnly, Phase, Strength};
use crate::queries::terrain::{height_at, lantern_spots};
use crate::resources::Terrain;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_with_material;
use nightshade_graphics::commands::light::spawn_light_entity;
use nightshade_render::components::light::{Light, LightType};

pub fn spawn(commands: &mut Commands, terrain: &Terrain) {
    for (index, (x, z)) in lantern_spots(terrain).into_iter().enumerate() {
        let Some(height) = height_at(terrain, x, z) else {
            continue;
        };
        let foot = height as f32;
        spawn_mesh_with_material(
            commands,
            "Cube",
            Vec3::new(x, foot + theme::LANTERN_POST_HEIGHT / 2.0, z),
            Vec3::new(0.12, theme::LANTERN_POST_HEIGHT, 0.12),
            theme::POST_MATERIAL,
        );

        let lamp_center = Vec3::new(
            x,
            foot + theme::LANTERN_POST_HEIGHT + theme::LANTERN_LAMP_SIZE / 2.0,
            z,
        );
        spawn_mesh_with_material(
            commands,
            "Cube",
            lamp_center,
            Vec3::new(
                theme::LANTERN_LAMP_SIZE,
                theme::LANTERN_LAMP_SIZE,
                theme::LANTERN_LAMP_SIZE,
            ),
            theme::LAMP_MATERIAL,
        );

        let light = spawn_light_entity(commands, lamp_center + Vec3::new(0.0, 0.3, 0.0), "Lantern");
        commands.set(
            light,
            Light {
                light_type: LightType::Point,
                color: theme::LANTERN_COLOR,
                intensity: theme::LANTERN_STRENGTH,
                range: theme::LANTERN_RANGE,
                ..Default::default()
            },
        );
        commands.set(light, Phase(index as f32 * 1.7));
        commands.set(light, Strength(theme::LANTERN_STRENGTH));
        commands.set(light, NightOnly(true));
    }
}
