use crate::components::{
    Angle, AngularSpeed, BobPhase, Health, Height, Hit, Label, OrbitRadius, Respawn,
};
use crate::queries::palm_axis;
use crate::resources::Jarvis;
use crate::theme::{
    BUTTON_FACE, DRONE_COLOR, DRONE_HEALTH, FLARE_COLOR, FLOOR_COLOR, GAUNTLET_COLOR,
    GAUNTLET_TRIM, GLOW, GLOW_CORE, MARGIN, REPULSOR_COLOR,
};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_openxr::contract::{XrHand, XrRigSlot};
use nightshade_render::commands::material_registry::queue_set_material;
use nightshade_render::data::material::Material;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::theme::RectEffect;
use std::f32::consts::FRAC_PI_2;

const DRONES: [(f32, f32, f32, f32); 5] = [
    (5.0, 1.6, 0.35, 0.0),
    (7.5, 2.4, -0.22, 1.3),
    (9.0, 1.2, 0.18, 2.7),
    (6.0, 3.1, -0.4, 4.1),
    (11.0, 2.0, 0.12, 5.4),
];
const DRONE_SCALE: f32 = 0.22;
const CONTROLS_WIDTH: f32 = 150.0;
const CONTROLS_HEIGHT: f32 = 84.0;
const RETICLE_GLOW_SIZE: f32 = 340.0;
const REACTOR_GLOW_SIZE: f32 = 170.0;
const REACTOR_CENTER: Vec2 = Vec2::new(12.5, 47.0);
const CLEAR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

pub fn spawn_hangar(
    mut settings: ResMut<RenderSettings>,
    mut debug_draw: ResMut<DebugDraw>,
    mut jarvis: ResMut<Jarvis>,
    rig: Res<XrRigSlot>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Nebula;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.12;
    debug_draw.show_grid = true;

    let sun = spawn_sun(&mut commands);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<Light>(sun) {
            light.cast_shadows = true;
        }
    });

    let floor = spawn_plane_at(&mut commands, Vec3::zeros());
    commands.set(
        floor,
        LocalTransform {
            translation: Vec3::zeros(),
            rotation: Quat::identity(),
            scale: Vec3::new(80.0, 1.0, 80.0),
        },
    );
    queue_set_material(
        &mut commands,
        floor,
        "hangar_floor",
        Material {
            base_color: FLOOR_COLOR,
            roughness: 0.35,
            metallic: 0.6,
            ..Default::default()
        },
    );

    for (label, (orbit_radius, height, angular_speed, angle)) in DRONES.iter().enumerate() {
        let drone = spawn_sphere_at(&mut commands, Vec3::zeros());
        commands.set(
            drone,
            LocalTransform {
                translation: Vec3::new(
                    orbit_radius * angle.cos(),
                    *height,
                    orbit_radius * angle.sin(),
                ),
                rotation: Quat::identity(),
                scale: Vec3::new(DRONE_SCALE, DRONE_SCALE, DRONE_SCALE),
            },
        );
        queue_set_material(
            &mut commands,
            drone,
            format!("drone_{label}"),
            Material {
                base_color: [DRONE_COLOR[0], DRONE_COLOR[1], DRONE_COLOR[2], 1.0],
                emissive_factor: DRONE_COLOR,
                roughness: 0.3,
                metallic: 0.5,
                ..Default::default()
            },
        );
        commands.set(drone, Label(label + 1));
        commands.set(drone, OrbitRadius(*orbit_radius));
        commands.set(drone, Height(*height));
        commands.set(drone, Angle(*angle));
        commands.set(drone, AngularSpeed(*angular_speed));
        commands.set(drone, BobPhase(label as f32 * 1.7));
        commands.set(drone, Health(DRONE_HEALTH));
        commands.set(drone, Hit(0.0));
        commands.set(drone, Respawn(0.0));
    }

    let Some(rig) = rig.0 else {
        return;
    };
    for eye in [rig.left_eye, rig.right_eye] {
        commands.set(
            eye,
            CameraPostProcess {
                color_grading: Some(ColorGrading {
                    vignette_intensity: 0.42,
                    vignette_radius: 0.78,
                    vignette_smoothness: 0.5,
                    chromatic_aberration: 0.006,
                    saturation: 1.1,
                    ..ColorGrading::default()
                }),
                bloom_intensity: Some(0.12),
                ..Default::default()
            },
        );
    }
    for hand in XrHand::ALL {
        let flare = spawn_gauntlet(&mut commands, rig.hand(hand), hand);
        jarvis.gauntlet_mut(hand).flare = flare;
    }
}

fn spawn_gauntlet(commands: &mut Commands, hand_entity: Entity, hand: XrHand) -> Entity {
    let axis = palm_axis(hand);
    let palm_rotation = match hand {
        XrHand::Left => nalgebra_glm::quat_angle_axis(-FRAC_PI_2, &Vec3::z()),
        XrHand::Right => nalgebra_glm::quat_angle_axis(FRAC_PI_2, &Vec3::z()),
    };

    let body = spawn_part(commands, hand_entity, |commands| {
        spawn_cube_at(commands, Vec3::zeros())
    });
    commands.set(
        body,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.02),
            rotation: Quat::identity(),
            scale: Vec3::new(0.075, 0.045, 0.15),
        },
    );
    queue_set_material(
        commands,
        body,
        format!("gauntlet_body_{hand:?}"),
        Material {
            base_color: GAUNTLET_COLOR,
            roughness: 0.35,
            metallic: 0.85,
            ..Default::default()
        },
    );

    let trim = spawn_part(commands, hand_entity, |commands| {
        spawn_cube_at(commands, Vec3::zeros())
    });
    commands.set(
        trim,
        LocalTransform {
            translation: Vec3::new(0.0, 0.03, -0.04),
            rotation: Quat::identity(),
            scale: Vec3::new(0.082, 0.018, 0.05),
        },
    );
    queue_set_material(
        commands,
        trim,
        format!("gauntlet_trim_{hand:?}"),
        Material {
            base_color: GAUNTLET_TRIM,
            roughness: 0.25,
            metallic: 1.0,
            ..Default::default()
        },
    );

    let disc = spawn_part(commands, hand_entity, |commands| {
        spawn_cylinder_at(commands, Vec3::zeros())
    });
    commands.set(
        disc,
        LocalTransform {
            translation: axis * 0.042,
            rotation: palm_rotation,
            scale: Vec3::new(0.03, 0.005, 0.03),
        },
    );
    queue_set_material(
        commands,
        disc,
        format!("repulsor_{hand:?}"),
        Material {
            base_color: [REPULSOR_COLOR[0], REPULSOR_COLOR[1], REPULSOR_COLOR[2], 1.0],
            emissive_factor: REPULSOR_COLOR,
            roughness: 0.2,
            ..Default::default()
        },
    );

    let flare = spawn_part(commands, hand_entity, |commands| {
        spawn_cone_at(commands, Vec3::zeros())
    });
    commands.set(
        flare,
        LocalTransform {
            translation: axis * 0.06,
            rotation: palm_rotation,
            scale: Vec3::new(0.02, 0.02, 0.02),
        },
    );
    queue_set_material(
        commands,
        flare,
        format!("flare_{hand:?}"),
        Material {
            base_color: [FLARE_COLOR[0], FLARE_COLOR[1], FLARE_COLOR[2], 0.9],
            emissive_factor: FLARE_COLOR,
            roughness: 0.1,
            ..Default::default()
        },
    );
    flare
}

fn spawn_part(
    commands: &mut Commands,
    parent: Entity,
    spawn: impl FnOnce(&mut Commands) -> Entity,
) -> Entity {
    let entity = spawn(commands);
    commands.set(entity, nightshade::ecs::ChildOf(parent));
    entity
}

pub fn build_visor(mut build: UiBuild, mut jarvis: ResMut<Jarvis>, mut commands: Commands) {
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = jarvis.handles;
    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(RETICLE_GLOW_SIZE, RETICLE_GLOW_SIZE)),
                Anchor::Center,
            )
            .with_rect(RETICLE_GLOW_SIZE * 0.5, 0.0, CLEAR)
            .color_raw::<UiBase>(GLOW)
            .with_effect(RectEffect::RadialGlow {
                glow_color: GLOW_CORE,
                intensity: 0.8,
            });
        tree.add_node()
            .window(
                Rl(REACTOR_CENTER),
                Ab(vec2(REACTOR_GLOW_SIZE, REACTOR_GLOW_SIZE)),
                Anchor::Center,
            )
            .with_rect(REACTOR_GLOW_SIZE * 0.5, 0.0, CLEAR)
            .color_raw::<UiBase>(GLOW)
            .with_effect(RectEffect::RadialGlow {
                glow_color: GLOW_CORE,
                intensity: 1.2,
            });

        handles.canvas = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();
        tree.commands().set(
            handles.canvas,
            nightshade_ui::components::UiCanvasData {
                commands: Vec::new(),
                hit_test_enabled: false,
                command_bounds: Vec::new(),
            },
        );

        let controls = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-MARGIN, -MARGIN)),
                Ab(vec2(CONTROLS_WIDTH, CONTROLS_HEIGHT)),
                Anchor::BottomRight,
            )
            .flow(FlowDirection::Vertical, 0.0, 8.0)
            .entity();
        tree.in_parent(controls, |tree| {
            handles.pin_button = tree.add_button("PIN VISOR");
            handles.flight_button = tree.add_button("FLIGHT MODE");
        });
    });
    handles.root = tree.root_entity();
    tree.finish();

    for button in [handles.pin_button, handles.flight_button] {
        ui_set_color::<UiBase>(&mut commands, button, BUTTON_FACE);
    }
    jarvis.handles = handles;
}
