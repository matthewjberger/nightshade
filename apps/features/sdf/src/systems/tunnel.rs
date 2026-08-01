use crate::data::Settings;
use crate::resources::SdfScene;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn aim(
    mut scene: ResMut<SdfScene>,
    input: Res<Input>,
    config: Res<Config<Settings>>,
    mut commands: Commands,
    transforms: Query<(&LocalTransform,)>,
) {
    let (Some(camera), Some(tunnel)) = (scene.camera, scene.tunnel) else {
        return;
    };
    if input.keyboard.just_pressed(KeyCode::KeyF) {
        scene.digging = !scene.digging;
        commands.set(
            tunnel,
            Visibility {
                visible: scene.digging,
            },
        );
    }
    if !scene.digging {
        return;
    }
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
    commands.set(
        tunnel,
        LocalTransform {
            translation: view.translation + ahead * config.number(Settings::TunnelReach),
            rotation: nalgebra_glm::quat_rotation(&Vec3::new(0.0, 1.0, 0.0), &ahead),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
}
