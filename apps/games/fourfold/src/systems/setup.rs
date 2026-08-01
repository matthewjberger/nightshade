use crate::commands as game;
use crate::components::{Cell, Dial, Extent, Pip};
use crate::contract::{Cue, Dimension, Glyph};
use crate::data;
use crate::queries;
use crate::resources::FourfoldResources;
use crate::systems::{hud, menu, palette};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::commands::builtins::{spawn_mesh_at, spawn_sun};
use nightshade_graphics::prelude::*;
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::Window;
use nightshade_portal::contract::PortalPlane;
use nightshade_portal::prelude::{Link, Portal, PortalSettings, Rim};
use nightshade_render::data::config::{Fog, TonemapAlgorithm};
use nightshade_scene::prelude::Visibility;

pub fn build_overlay(
    mut resources: ResMut<FourfoldResources>,
    style: Res<nightshade_ui_menu::prelude::MenuStyle>,
    mut menus: ResMut<nightshade_ui_menu::prelude::Menus<crate::resources::Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    resources.hud = hud::build(&mut tree);
    resources.menu = menu::build(&mut tree, &style, &mut menus);
    tree.finish();
}

pub fn initialize(
    mut resources: ResMut<FourfoldResources>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut portals: ResMut<PortalSettings>,
    mut viewport: ResMut<nightshade_render::resources::viewport::Viewport>,
    mut window: ResMut<Window>,
    mut commands: Commands,
) {
    dress_the_scene(&mut settings);
    spawn_sun(&mut commands);
    palette::insert_all(&mut registry);

    build_solids(&mut commands);
    build_arches(&mut resources, &mut commands);
    build_dials(&mut commands);
    build_posts(&mut resources, &mut commands);
    build_lamps(&mut commands);
    build_loom(&mut resources, &mut commands);
    build_cairn(&mut commands);
    build_hearth(&mut resources, &mut commands);
    open_the_portal(&mut commands, &mut portals);

    resources.avatar = spawn_avatar(&mut commands);
    resources.hub_camera = spawn_hub_camera(&mut commands, &resources);
    resources.glass_camera = spawn_glass_camera(&mut commands, &resources);
    resources.loom_camera = spawn_flat_camera(
        &mut commands,
        "Loom Eye",
        data::LOOM_ORIGIN + Vec3::new(0.0, data::LOOM_CAMERA_LIFT, 0.0),
        nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
        data::LOOM_Y_MAG,
    );
    resources.cairn_camera =
        spawn_camera(&mut commands, data::CAIRN_START, "Cairn Eye".to_string());
    commands.set(resources.cairn_camera, hub_lens());
    resources.hearth_camera =
        spawn_camera(&mut commands, data::HEARTH_START, "Hearth Eye".to_string());
    commands.set(
        resources.hearth_camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: data::HEARTH_FIELD_OF_VIEW,
                z_far: Some(data::FAR_PLANE),
                z_near: data::NEAR_PLANE,
            }),
            smoothing: None,
        },
    );
    active_camera.0 = Some(resources.hub_camera);
    game::dispatch_last(
        &mut viewport,
        [
            resources.hub_camera,
            resources.glass_camera,
            resources.loom_camera,
            resources.cairn_camera,
            resources.hearth_camera,
        ],
        resources.hub_camera,
    );
    portals.viewer = resources.hub_camera;
    portals.field_of_view = data::HUB_FIELD_OF_VIEW;
    portals.near_plane = data::NEAR_PLANE;
    portals.far_plane = data::FAR_PLANE;
    portals.plane_bias = data::PORTAL_PLANE_BIAS;
    portals.target_max = data::PORTAL_TARGET_MAX;
    portals.depth = 1;
    set_pointer_grab(&mut window, false);
    game::queue_cue(&mut resources.story, Cue::Opening);
}

fn open_the_portal(commands: &mut Commands, portals: &mut PortalSettings) {
    for gate in data::gates() {
        let arch = commands.spawn((
            Portal(PortalPlane {
                center: gate.center,
                normal: gate.normal,
                up: Vec3::y(),
                width: data::ARCH_WIDTH,
                height: data::ARCH_HEIGHT,
            }),
            Rim(data::PORTAL_RIM),
        ));
        let window = commands.spawn((
            Portal(PortalPlane {
                center: gate.window,
                normal: gate.window_normal,
                up: Vec3::y(),
                width: data::ARCH_WIDTH,
                height: data::ARCH_HEIGHT,
            }),
            Rim(data::PORTAL_RIM),
        ));
        commands.set(arch, Link(window));
        commands.set(window, Link(arch));
    }
    portals.depth = 1;
}

pub fn hub_lens() -> Camera {
    Camera {
        projection: Projection::Perspective(PerspectiveCamera {
            aspect_ratio: None,
            y_fov_rad: data::HUB_FIELD_OF_VIEW,
            z_far: Some(data::FAR_PLANE),
            z_near: data::NEAR_PLANE,
        }),
        smoothing: None,
    }
}

pub fn glass_lens() -> Camera {
    Camera {
        projection: Projection::Orthographic(OrthographicCamera {
            x_mag: None,
            y_mag: data::GLASS_Y_MAG,
            z_far: data::FAR_PLANE,
            z_near: data::NEAR_PLANE,
        }),
        smoothing: None,
    }
}

fn dress_the_scene(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::Space;
    settings.clear_color = [0.02, 0.02, 0.03, 1.0];
    settings.fog = Some(Fog {
        color: data::HUB_FOG,
        start: data::FOG_START,
        end: data::FOG_END,
        ..Default::default()
    });
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.11;
    settings.bloom_threshold = 0.8;
    settings.ssao_enabled = true;
    settings.ssao_intensity = 0.8;
    settings.color_grading.tonemap_algorithm = TonemapAlgorithm::Aces2;
    settings.color_grading.vignette_intensity = 0.42;
    settings.color_grading.vignette_radius = 0.74;
    settings.color_grading.contrast = 1.08;
}

fn build_solids(commands: &mut Commands) {
    for slab in data::hub_solids()
        .into_iter()
        .chain(data::glass_solids())
        .chain(data::loom_solids())
        .chain(data::loom_tiles())
        .chain(data::cairn_solids())
        .chain(data::hearth_solids())
    {
        let entity = spawn_mesh_at(commands, "Cube", slab.center, slab.size);
        set_mesh_material(commands, entity, slab.material);
        commands.set(entity, slab.dimension);
        commands.set(entity, Extent(slab.size));
    }
}

fn build_arches(resources: &mut FourfoldResources, commands: &mut Commands) {
    let seal = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(0.0, -0.4, 0.0),
        Vec3::new(data::SHAFT_WIDTH, 0.8, data::SHAFT_WIDTH),
    );
    set_mesh_material(commands, seal, data::SEAL);
    commands.set(seal, Dimension::Hub);
    commands.set(
        seal,
        Extent(Vec3::new(data::SHAFT_WIDTH, 0.8, data::SHAFT_WIDTH)),
    );
    resources.seal = seal;

    let below = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(0.0, -9.0, 0.0),
        Vec3::new(data::SHAFT_WIDTH, 0.4, data::SHAFT_WIDTH),
    );
    set_mesh_material(commands, below, data::SHUT_ARCH);
}

fn build_dials(commands: &mut Commands) {
    for dial in 0..data::DIAL_COUNT {
        let center = data::dial_center(dial);
        let plate = spawn_mesh_at(commands, "Cube", center, data::DIAL_PLATE);
        set_mesh_material(commands, plate, data::PLATE);
        commands.set(plate, Dial(dial));

        for index in 0..data::GLYPH_ROWS * data::GLYPH_ROWS {
            let (across, up) = data::pip_offset(index, data::PIP_SPACING);
            let pip = spawn_mesh_at(
                commands,
                "Cube",
                center + Vec3::new(across, up, -data::PIP_STANDOFF),
                data::PIP_SIZE,
            );
            set_mesh_material(commands, pip, data::PIP_LIT);
            commands.set(pip, Dial(dial));
            commands.set(pip, Pip(index));
            commands.set(
                pip,
                Visibility {
                    visible: data::pip_lit(Glyph::Bar, index),
                },
            );
        }
    }
}

fn build_posts(resources: &mut FourfoldResources, commands: &mut Commands) {
    for (index, tone) in data::TONES.iter().enumerate() {
        let center = tone.post;
        let frame = spawn_mesh_at(
            commands,
            "Cube",
            center + Vec3::new(0.0, 0.0, -0.07),
            data::WALL_FRAME,
        );
        set_mesh_material(commands, frame, data::PLATE);
        resources.frames[index] = frame;

        let plate = spawn_mesh_at(commands, "Cube", center, data::WALL_PLATE);
        set_mesh_material(commands, plate, data::PLATE);

        for pip in 0..data::GLYPH_ROWS * data::GLYPH_ROWS {
            if !data::pip_lit(tone.glyph, pip) {
                continue;
            }
            let (across, up) = data::pip_offset(pip, data::WALL_PIP_SPACING);
            let mark = spawn_mesh_at(
                commands,
                "Cube",
                center + Vec3::new(across, up, data::PIP_STANDOFF),
                data::WALL_PIP_SIZE,
            );
            set_mesh_material(commands, mark, data::PIP_LIT);
        }
    }

    let door = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(data::GLASS_GATE.x - 0.6, 1.8, 0.0),
        Vec3::new(0.3, 3.6, 2.6),
    );
    set_mesh_material(commands, door, data::BONE);
}

fn build_loom(resources: &mut FourfoldResources, commands: &mut Commands) {
    for plate in data::LOOM_PLATES {
        let mark = spawn_mesh_at(
            commands,
            "Cube",
            data::loom_cell(plate) + Vec3::new(0.0, 0.06, 0.0),
            Vec3::new(data::LOOM_TILE * 0.8, 0.12, data::LOOM_TILE * 0.8),
        );
        set_mesh_material(commands, mark, data::SHUT_ARCH);
    }
    for block in data::LOOM_BLOCKS {
        let cube = spawn_mesh_at(
            commands,
            "Cube",
            data::loom_cell(block) + Vec3::new(0.0, 0.7, 0.0),
            Vec3::new(1.4, 1.4, 1.4),
        );
        set_mesh_material(commands, cube, data::PILLAR);
        commands.set(cube, Cell(block));
    }
    let gate = spawn_mesh_at(
        commands,
        "Cube",
        data::loom_cell(data::LOOM_GATE) + Vec3::new(0.0, 0.06, 0.0),
        Vec3::new(data::LOOM_TILE * 0.9, 0.12, data::LOOM_TILE * 0.9),
    );
    set_mesh_material(commands, gate, data::BONE);

    let pawn = spawn_mesh_at(
        commands,
        "Cube",
        data::loom_cell(data::LOOM_START) + Vec3::new(0.0, 0.8, 0.0),
        Vec3::new(0.9, 1.6, 0.9),
    );
    set_mesh_material(commands, pawn, data::RUNNER);
    resources.pawn = pawn;
}

fn build_cairn(commands: &mut Commands) {
    let peak = spawn_mesh_at(
        commands,
        "Cube",
        data::CAIRN_PEAK + Vec3::new(0.0, 5.4, 0.0),
        Vec3::new(1.2, 1.2, 1.2),
    );
    set_mesh_material(commands, peak, data::BONE);
    let gate = spawn_mesh_at(
        commands,
        "Cube",
        data::CAIRN_GATE + Vec3::new(0.0, 2.6, 0.0),
        Vec3::new(1.6, 3.0, 0.4),
    );
    set_mesh_material(commands, gate, data::PORTAL_MARK);
}

fn spawn_art(commands: &mut Commands, art: &[&str], base: Vec3, cell: f32) {
    let tilt = nalgebra_glm::quat_angle_axis(data::HEARTH_TILT, &Vec3::x());
    let rows = art.len() as f32;
    for (row, line) in art.iter().enumerate() {
        for (column, mark) in line.chars().enumerate() {
            if data::art_color(mark).is_none() {
                continue;
            }
            let across = (column as f32 - 7.5) * cell;
            let up = (rows - 1.0 - row as f32) * cell;
            let offset = nalgebra_glm::quat_rotate_vec3(&tilt, &Vec3::new(across, up, 0.0));
            let pip = spawn_mesh_at(commands, "Cube", base + offset, Vec3::new(cell, cell, cell));
            set_mesh_material(commands, pip, data::art_material(mark));
        }
    }
}

fn build_hearth(resources: &mut FourfoldResources, commands: &mut Commands) {
    for spot in data::hearth_trees() {
        spawn_art(commands, &data::TREE_ART, spot, data::ART_CELL * 2.0);
    }
    for spot in data::hearth_folk() {
        spawn_art(commands, &data::FOLK_ART, spot, data::ART_CELL);
    }
    let gate = spawn_mesh_at(
        commands,
        "Cube",
        data::HEARTH_GATE + Vec3::new(0.0, 1.6, 0.0),
        Vec3::new(1.8, 3.2, 0.4),
    );
    set_mesh_material(commands, gate, data::PORTAL_MARK);

    let figure = spawn_mesh_at(
        commands,
        "Cube",
        data::HEARTH_START,
        Vec3::new(0.7, 1.8, 0.7),
    );
    set_mesh_material(commands, figure, data::RUNNER);
    resources.figure = figure;
}

fn build_lamps(commands: &mut Commands) {
    for tone in &data::TONES {
        let post = data::pillar_at(tone.pillar);
        let inward = Vec3::new(-post.x, 0.0, -post.z).normalize();
        let face = Vec3::new(post.x, 0.0, post.z) + inward * data::LAMP_INSET;

        let lamp = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(face.x, data::LAMP_LIFT, face.z),
            data::LAMP_SIZE,
        );
        set_mesh_material(commands, lamp, tone.material);

        for step in 0..tone.marks {
            let height = data::MARK_BASE - step as f32 * data::MARK_STEP;
            let mark = spawn_mesh_at(
                commands,
                "Cube",
                Vec3::new(face.x, height, face.z),
                data::MARK_SIZE,
            );
            set_mesh_material(commands, mark, data::PIP_LIT);
        }
    }
}

fn spawn_avatar(commands: &mut Commands) -> Entity {
    let entity = spawn_mesh_at(
        commands,
        "Cube",
        data::GLASS_START + Vec3::new(0.0, data::RUNNER_SIZE.y * 0.5, 0.0),
        data::RUNNER_SIZE,
    );
    set_mesh_material(commands, entity, data::RUNNER);
    entity
}

fn spawn_hub_camera(commands: &mut Commands, resources: &FourfoldResources) -> Entity {
    let eye = queries::eye_position(&resources.walker);
    let camera = spawn_camera(commands, eye, "Hub Eye".to_string());
    commands.set(
        camera,
        LocalTransform {
            translation: eye,
            rotation: queries::look_rotation(resources.walker.yaw, resources.walker.pitch),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(camera, hub_lens());
    camera
}

fn spawn_flat_camera(
    commands: &mut Commands,
    name: &str,
    eye: Vec3,
    rotation: Quat,
    y_mag: f32,
) -> Entity {
    let camera = spawn_camera(commands, eye, name.to_string());
    commands.set(
        camera,
        LocalTransform {
            translation: eye,
            rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(
        camera,
        Camera {
            projection: Projection::Orthographic(OrthographicCamera {
                x_mag: None,
                y_mag,
                z_far: data::FAR_PLANE,
                z_near: data::NEAR_PLANE,
            }),
            smoothing: None,
        },
    );
    camera
}

fn spawn_glass_camera(commands: &mut Commands, resources: &FourfoldResources) -> Entity {
    let eye = queries::glass_eye(&resources.runner);
    let camera = spawn_camera(commands, eye, "Glass Eye".to_string());
    commands.set(
        camera,
        LocalTransform {
            translation: eye,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(camera, glass_lens());
    camera
}
