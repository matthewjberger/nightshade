use nightshade::prelude::nalgebra_glm;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_openxr::OpenXrPlugin;
use nightshade_openxr::contract::{
    XrFrame, XrHand, XrPose, XrRigSlot, XrUiAnchor, XrUiPanel, XrUiPointer,
};
use nightshade_platform::prelude::*;
use nightshade_render::commands::material_registry::queue_set_material;
use nightshade_render::data::material::Material;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;

const PANEL_BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.78);
const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.12);
const TEXT_COLOR: Vec4 = Vec4::new(0.45, 0.92, 0.50, 1.0);
const PANEL_WIDTH: f32 = 360.0;
const PANEL_HEIGHT: f32 = 200.0;
const SPIN_SPEED: f32 = 0.8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugin(nightshade_window::WindowPlugin::default())
        .add_plugin(nightshade_log::LogPlugin::default())
        .add_plugin(nightshade_wgpu::RenderPlugin::default())
        .add_plugin(nightshade_camera::CameraPlugin)
        .add_plugin(OpenXrPlugin::default())
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(UiExamplePlugin)
        .run()
}

#[derive(Default)]
struct UiExample {
    cube: Option<Entity>,
    spinning: bool,
    rate_readout: Entity,
    pointer_readout: Entity,
    spin_button: Entity,
    pin_button: Entity,
    last_rate: u32,
    last_pointer: Option<(i32, i32)>,
}

struct UiExamplePlugin;

impl Plugin for UiExamplePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "openxr ui".to_string();
        app.insert_resource(UiExample {
            spinning: true,
            ..Default::default()
        });
        app.add_system(Stage::Startup, (spawn_scene, build_panel));
        app.add_system(
            Stage::Update,
            (spin_cube, show_rate, show_pointer, drive_button),
        );
    }
}

fn spawn_scene(
    mut settings: ResMut<RenderSettings>,
    mut example: ResMut<UiExample>,
    rig: Res<XrRigSlot>,
    mut commands: Commands,
) {
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

    let cube = spawn_cube_at(&mut commands, Vec3::new(0.0, 1.2, -2.5));
    commands.set(
        cube,
        LocalTransform {
            translation: Vec3::new(0.0, 1.2, -2.5),
            rotation: Quat::identity(),
            scale: Vec3::new(0.4, 0.4, 0.4),
        },
    );
    queue_set_material(
        &mut commands,
        cube,
        "cube",
        Material {
            base_color: [0.98, 0.62, 0.15, 1.0],
            roughness: 0.4,
            ..Default::default()
        },
    );
    example.cube = Some(cube);

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

fn build_panel(mut build: UiBuild, mut example: ResMut<UiExample>) {
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    tree.in_parent(root, |tree| {
        let panel = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(PANEL_WIDTH, PANEL_HEIGHT)),
                Anchor::Center,
            )
            .with_rect(12.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .flow(FlowDirection::Vertical, 14.0, 10.0)
            .entity();

        tree.in_parent(panel, |tree| {
            example.rate_readout = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0)))
                .with_text("", 22.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_COLOR)
                .entity();
            example.pointer_readout = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 22.0)))
                .with_text("pointer off the panel", 16.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_COLOR)
                .entity();
            example.spin_button = tree.add_button("Stop the cube");
            example.pin_button = tree.add_button("Pin the panel");
        });
    });
    tree.finish();
}

fn spin_cube(example: Res<UiExample>, time: Res<Time>, storage: &mut nightshade::ecs::Storage) {
    if !example.spinning {
        return;
    }
    let Some(cube) = example.cube else {
        return;
    };
    if let Some(transform) = storage.get_mut::<LocalTransform>(cube) {
        transform.rotation =
            nalgebra_glm::quat_angle_axis(SPIN_SPEED * time.delta_time, &Vec3::y())
                * transform.rotation;
    }
}

fn show_rate(mut example: ResMut<UiExample>, time: Res<Time>, mut commands: Commands) {
    if example.rate_readout == Entity::default() {
        return;
    }
    let rate = time.frames_per_second.round().max(0.0) as u32;
    if rate == example.last_rate {
        return;
    }
    example.last_rate = rate;
    ui_set_text(&mut commands, example.rate_readout, &format!("{rate} fps"));
}

fn show_pointer(mut example: ResMut<UiExample>, pointer: Res<XrUiPointer>, mut commands: Commands) {
    if example.pointer_readout == Entity::default() {
        return;
    }
    let current = pointer
        .hit
        .map(|hit| (hit.x.round() as i32, hit.y.round() as i32));
    if current == example.last_pointer {
        return;
    }
    example.last_pointer = current;
    let text = match current {
        Some((x, y)) => format!("pointer at {x} {y}"),
        None => "pointer off the panel".to_string(),
    };
    ui_set_text(&mut commands, example.pointer_readout, &text);
}

fn drive_button(
    mut example: ResMut<UiExample>,
    frame: Res<RetainedUiFrame>,
    xr_frame: Res<XrFrame>,
    mut panel: ResMut<XrUiPanel>,
    mut commands: Commands,
) {
    if example.spin_button == Entity::default() {
        return;
    }
    let mut spin_clicked = false;
    let mut pin_clicked = false;
    for entity in ui_button_clicks(&frame) {
        if entity == example.spin_button {
            spin_clicked = true;
        } else if entity == example.pin_button {
            pin_clicked = true;
        }
    }

    if spin_clicked {
        example.spinning = !example.spinning;
        let label = if example.spinning {
            "Stop the cube"
        } else {
            "Spin the cube"
        };
        ui_set_text(&mut commands, example.spin_button, label);
    }

    if pin_clicked {
        let label = match panel.anchor {
            XrUiAnchor::Head => {
                let forward =
                    nalgebra_glm::quat_rotate_vec3(&xr_frame.head_orientation, &-Vec3::z());
                panel.anchor = XrUiAnchor::Stage(XrPose {
                    position: xr_frame.head_position + forward * panel.distance,
                    orientation: xr_frame.head_orientation,
                });
                "Follow the head"
            }
            XrUiAnchor::Stage(_) => {
                panel.anchor = XrUiAnchor::Head;
                "Pin the panel"
            }
        };
        ui_set_text(&mut commands, example.pin_button, label);
    }
}
