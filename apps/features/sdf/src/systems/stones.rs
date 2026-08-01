use crate::commands::spawn_stone;
use crate::data::Settings;
use crate::resources::SdfScene;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn throw(
    input: Res<Input>,
    config: Res<Config<Settings>>,
    mut scene: ResMut<SdfScene>,
    mut commands: Commands,
    transforms: Query<(&LocalTransform,)>,
) {
    if !input.keyboard.just_pressed(KeyCode::KeyG) {
        return;
    }
    let Some(camera) = scene.camera else {
        return;
    };
    let mut seen = None;
    transforms.for_each(|entity, (transform,)| {
        if entity == camera {
            seen = Some(*transform);
        }
    });
    let Some(view) = seen else {
        return;
    };
    let ahead = nalgebra_glm::quat_rotate_vec3(&view.rotation, &Vec3::new(0.0, 0.0, -1.0));
    let stone = spawn_stone(
        &mut commands,
        &config,
        view.translation + ahead * config.number(Settings::StoneAhead),
        ahead * config.number(Settings::StoneSpeed),
    );
    scene.stones.push_back(stone);
    while scene.stones.len() > theme::STONE_LIMIT {
        if let Some(oldest) = scene.stones.pop_front() {
            commands.despawn_recursive(oldest);
        }
    }
}
