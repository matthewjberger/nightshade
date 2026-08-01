use nightshade::prelude::nalgebra_glm;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_openxr::OpenXrPlugin;
use nightshade_openxr::contract::{XrFrame, XrHand, XrInput, XrRigSlot};
use nightshade_platform::prelude::*;
use nightshade_render::commands::material_registry::queue_set_material;
use nightshade_render::data::material::Material;

const MOVE_SPEED: f32 = 2.0;
const SNAP_TURN_RADIANS: f32 = std::f32::consts::FRAC_PI_6;
const STICK_DEAD_ZONE: f32 = 0.2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugin(nightshade_window::WindowPlugin::default())
        .add_plugin(nightshade_log::LogPlugin::default())
        .add_plugin(nightshade_wgpu::RenderPlugin::default())
        .add_plugin(nightshade_camera::CameraPlugin)
        .add_plugin(OpenXrPlugin::default())
        .add_plugin(MinimalPlugin)
        .run()
}

#[derive(Default)]
struct Minimal {
    turn_armed: bool,
}

struct MinimalPlugin;

impl Plugin for MinimalPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "minimal".to_string();
        app.insert_resource(Minimal::default());
        app.add_system(Stage::Startup, initialize);
        app.add_system(Stage::Update, locomotion);
    }
}

fn initialize(mut settings: ResMut<RenderSettings>, rig: Res<XrRigSlot>, mut commands: Commands) {
    settings.atmosphere = Atmosphere::Sky;

    let sun = spawn_sun(&mut commands);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<Light>(sun) {
            light.cast_shadows = true;
        }
    });

    let ground = spawn_plane_at(&mut commands, Vec3::zeros());
    commands.set(
        ground,
        LocalTransform {
            translation: Vec3::zeros(),
            rotation: Quat::identity(),
            scale: Vec3::new(120.0, 1.0, 120.0),
        },
    );
    queue_set_material(
        &mut commands,
        ground,
        "ground",
        Material {
            base_color: [0.22, 0.24, 0.20, 1.0],
            roughness: 0.9,
            ..Default::default()
        },
    );

    let Some(rig) = rig.0 else {
        return;
    };
    for hand in XrHand::ALL {
        spawn_wand(&mut commands, rig.hand(hand), hand);
    }
}

fn spawn_wand(commands: &mut Commands, hand_entity: Entity, hand: XrHand) {
    let tint = match hand {
        XrHand::Left => [0.15, 0.75, 0.95],
        XrHand::Right => [0.98, 0.62, 0.15],
    };

    let shaft = spawn_cylinder_at(commands, Vec3::zeros());
    commands.set(shaft, nightshade::ecs::ChildOf(hand_entity));
    commands.set(
        shaft,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, -0.06),
            rotation: nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x()),
            scale: Vec3::new(0.018, 0.075, 0.018),
        },
    );
    queue_set_material(
        commands,
        shaft,
        format!("wand_shaft_{hand:?}"),
        Material {
            base_color: [0.1, 0.1, 0.12, 1.0],
            roughness: 0.4,
            metallic: 0.8,
            ..Default::default()
        },
    );

    let tip = spawn_sphere_at(commands, Vec3::zeros());
    commands.set(tip, nightshade::ecs::ChildOf(hand_entity));
    commands.set(
        tip,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, -0.14),
            rotation: Quat::identity(),
            scale: Vec3::new(0.03, 0.03, 0.03),
        },
    );
    queue_set_material(
        commands,
        tip,
        format!("wand_tip_{hand:?}"),
        Material {
            base_color: [tint[0], tint[1], tint[2], 1.0],
            emissive_factor: tint,
            roughness: 0.3,
            ..Default::default()
        },
    );
}

fn locomotion(
    mut minimal: ResMut<Minimal>,
    input: Res<XrInput>,
    rig: Res<XrRigSlot>,
    xr_frame: Res<XrFrame>,
    time: Res<Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(rig) = rig.0 else {
        return;
    };
    let input = *input;
    let delta_time = time.delta_time;

    let head_yaw = yaw_of(&xr_frame.head_orientation);
    let origin_yaw = storage
        .get::<LocalTransform>(rig.origin)
        .map(|transform| yaw_of(&transform.rotation))
        .unwrap_or(0.0);

    let stick = input.left.thumbstick;
    let mut translation = Vec3::zeros();
    if stick.norm() > STICK_DEAD_ZONE {
        let yaw = origin_yaw + head_yaw;
        let forward = Vec3::new(yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, yaw.sin());
        translation = (forward * stick.y + right * stick.x) * MOVE_SPEED * delta_time;
    }

    let turn_axis = input.right.thumbstick.x;
    let mut turn = 0.0;
    if turn_axis.abs() > 0.6 {
        if !minimal.turn_armed {
            turn = -turn_axis.signum() * SNAP_TURN_RADIANS;
            minimal.turn_armed = true;
        }
    } else if turn_axis.abs() < 0.3 {
        minimal.turn_armed = false;
    }

    if translation == Vec3::zeros() && turn == 0.0 {
        return;
    }
    if let Some(transform) = storage.get_mut::<LocalTransform>(rig.origin) {
        transform.translation += translation;
        if turn != 0.0 {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(turn, &Vec3::y()) * transform.rotation;
        }
    }
}

fn yaw_of(rotation: &Quat) -> f32 {
    let forward = nalgebra_glm::quat_rotate_vec3(rotation, &Vec3::new(0.0, 0.0, -1.0));
    forward.x.atan2(-forward.z)
}
