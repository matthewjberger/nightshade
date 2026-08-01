use crate::resources::SandboxResources;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::contract::ThirdPersonCamera;

const FIRST_PERSON_LOOK_SPEED: f32 = 0.75;
const GAMEPAD_LOOK_SPEED: f32 = 0.75;
const THIRD_PERSON_ORBIT_SPEED: f32 = 0.5;

pub const LOOK_SENSITIVITY_MIN: f32 = 0.1;
pub const LOOK_SENSITIVITY_MAX: f32 = 3.0;

pub fn set_look_sensitivity(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    sensitivity: f32,
) {
    sandbox_world.camera.look_sensitivity =
        sensitivity.clamp(LOOK_SENSITIVITY_MIN, LOOK_SENSITIVITY_MAX);
    apply(sandbox_world, scene);
}

pub fn apply(sandbox_world: &SandboxResources, scene: &mut crate::commands::Scene<'_, '_>) {
    let sensitivity = sandbox_world.camera.look_sensitivity;

    scene.commands.queue(move |storage, _| {
        let cameras: Vec<Entity> = storage
            .query_ref::<&Camera>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();
        for camera in cameras {
            if let Some(component) = storage.get_mut::<Camera>(camera)
                && let Some(smoothing) = component.smoothing.as_mut()
            {
                smoothing.mouse_sensitivity = FIRST_PERSON_LOOK_SPEED * sensitivity;
                smoothing.gamepad_sensitivity = GAMEPAD_LOOK_SPEED * sensitivity;
            }
            if let Some(third_person) = storage.get_mut::<ThirdPersonCamera>(camera) {
                third_person.orbit_sensitivity = THIRD_PERSON_ORBIT_SPEED * sensitivity;
            }
        }
    });
}
