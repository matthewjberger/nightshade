use crate::components::Phase;
use crate::contract::PortalSide;
use crate::data;
use crate::queries;
use crate::resources::PortalsResources;
use crate::systems::{hud, menu};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::Window;
use nightshade_portal::contract::PortalPlane;
use nightshade_portal::prelude::{Link, Portal, PortalSettings, Rim};
use nightshade_render::components::culling::{CameraCullingMask, CullingMask};
use nightshade_text::prelude::*;

pub fn initialize(
    mut resources: ResMut<PortalsResources>,
    mut settings: ResMut<RenderSettings>,
    mut portals: ResMut<PortalSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut viewport: ResMut<nightshade_render::resources::viewport::Viewport>,
    mut window: ResMut<Window>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Sky;
    settings.color_grading.tonemap_algorithm =
        nightshade_render::data::config::TonemapAlgorithm::Reinhard;
    spawn_sun(&mut commands);

    let mut solids = Vec::new();
    for slab in data::rooms().into_iter().chain(data::props()) {
        let entity = spawn_mesh_at(
            &mut commands,
            "Cube",
            Vec3::new(slab.center[0], slab.center[1], slab.center[2]),
            Vec3::new(slab.size[0], slab.size[1], slab.size[2]),
        );
        set_material(
            &mut registry,
            &mut commands,
            entity,
            format!("portals_slab_{}", entity.id),
            Material {
                base_color: slab.color,
                roughness: slab.roughness,
                ..Default::default()
            },
        );
        solids.push(queries::solid_from(slab.center, slab.size));
    }
    resources.solids = solids;

    for (side, plane) in placements() {
        let index = side.index();
        let surface = commands.spawn((Portal(plane), side, Rim(rim_of(side))));
        resources.surfaces[index] = surface;
        spawn_sparks(side, &mut commands);
    }
    for side in PortalSide::BOTH {
        let other = resources.surfaces[side.index() ^ 1];
        commands.set(resources.surfaces[side.index()], Link(other));
    }

    spawn_avatar(
        &mut resources,
        &mut registry,
        &mut text_state,
        &mut commands,
    );

    let camera = spawn_camera(
        &mut commands,
        queries::eye_position(&resources.player),
        "Player Camera".to_string(),
    );
    commands.set(camera, CameraCullingMask::new(CullingMask::DEFAULT_LAYER));
    commands.set(
        camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: data::FIELD_OF_VIEW,
                z_far: Some(data::FAR_PLANE),
                z_near: data::NEAR_PLANE,
            }),
            smoothing: None,
        },
    );
    resources.player.camera = camera;
    active_camera.0 = Some(camera);

    viewport.required_cameras = vec![camera];
    portals.viewer = camera;
    portals.field_of_view = data::FIELD_OF_VIEW;
    portals.near_plane = data::NEAR_PLANE;
    portals.far_plane = data::FAR_PLANE;
    portals.plane_bias = data::PLANE_BIAS;
    portals.target_max = data::PORTAL_TARGET_MAX;
    portals.depth = data::PORTAL_DEPTH;
    portals.max_distance = data::PORTAL_MAX_DISTANCE;

    set_pointer_grab(&mut window, true);
}

pub fn build_overlay(
    mut resources: ResMut<PortalsResources>,
    style: Res<nightshade_ui_menu::prelude::MenuStyle>,
    mut menus: ResMut<nightshade_ui_menu::prelude::Menus<crate::resources::Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    resources.hud = hud::build(&mut tree);
    resources.menu = menu::build(&mut tree, &style, &mut menus);
    tree.finish();
}

fn rim_of(side: PortalSide) -> [f32; 4] {
    match side {
        PortalSide::Blue => data::BLUE_RIM,
        PortalSide::Orange => data::ORANGE_RIM,
    }
}

fn spark_gradient(side: PortalSide) -> ColorGradient {
    let rim = rim_of(side);
    let lifted = |channel: f32| channel + (1.0 - channel) * 0.45;
    ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
            (
                0.12,
                Vec4::new(lifted(rim[0]), lifted(rim[1]), lifted(rim[2]), 1.0),
            ),
            (0.5, Vec4::new(rim[0], rim[1], rim[2], 0.9)),
            (
                1.0,
                Vec4::new(rim[0] * 0.3, rim[1] * 0.3, rim[2] * 0.3, 0.0),
            ),
        ],
    }
}

fn spawn_sparks(side: PortalSide, commands: &mut Commands) {
    let gradient = spark_gradient(side);
    for index in 0..data::SPARKS_PER_RIM {
        let emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            spawn_rate: data::SPARK_RATE,
            particle_lifetime_min: data::SPARK_LIFE[0],
            particle_lifetime_max: data::SPARK_LIFE[1],
            initial_velocity_min: data::SPARK_SPEED[0],
            initial_velocity_max: data::SPARK_SPEED[1],
            velocity_spread: data::SPARK_SPREAD,
            gravity: Vec3::new(0.0, data::SPARK_GRAVITY, 0.0),
            drag: data::SPARK_DRAG,
            size_start: data::SPARK_SIZE[0],
            size_end: data::SPARK_SIZE[1],
            color_gradient: gradient.clone(),
            emissive_strength: data::SPARK_GLOW,
            ..Default::default()
        };
        commands.spawn((
            side,
            Phase(index as f32 / data::SPARKS_PER_RIM as f32),
            emitter,
        ));
    }
}

fn spawn_avatar(
    resources: &mut PortalsResources,
    registry: &mut MaterialRegistry,
    text_state: &mut TextState,
    commands: &mut Commands,
) {
    let standing = resources.player.position;
    let avatar = spawn_mesh_at(
        commands,
        "Cylinder",
        standing + Vec3::new(0.0, data::PLAYER_HEIGHT * 0.5, 0.0),
        Vec3::new(
            data::AVATAR_RADIUS * 2.0,
            data::PLAYER_HEIGHT,
            data::AVATAR_RADIUS * 2.0,
        ),
    );
    set_material(
        registry,
        commands,
        avatar,
        "portals_avatar".to_string(),
        Material {
            base_color: data::AVATAR_COLOR,
            roughness: 0.6,
            ..Default::default()
        },
    );
    commands.set(avatar, CullingMask::new(data::AVATAR_LAYER));
    resources.player.avatar = avatar;

    let label = spawn_3d_billboard_text_with_properties(
        text_state,
        commands,
        "Player",
        standing + Vec3::new(0.0, data::PLAYER_HEIGHT + data::LABEL_LIFT, 0.0),
        TextProperties {
            font_size: data::LABEL_SIZE,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: data::LABEL_OUTLINE,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );
    commands.set(label, CullingMask::new(data::AVATAR_LAYER));
    resources.player.label = label;
}

fn placements() -> [(PortalSide, PortalPlane); 2] {
    let wall = -data::HALF_DEPTH + data::PORTAL_SURFACE_OFFSET;
    [
        (
            PortalSide::Blue,
            PortalPlane {
                center: Vec3::new(-6.0, data::PORTAL_LIFT, wall),
                normal: Vec3::z(),
                up: Vec3::y(),
                width: data::PORTAL_WIDTH,
                height: data::PORTAL_HEIGHT,
            },
        ),
        (
            PortalSide::Orange,
            PortalPlane {
                center: Vec3::new(6.0, data::PORTAL_LIFT, wall),
                normal: Vec3::z(),
                up: Vec3::y(),
                width: data::PORTAL_WIDTH,
                height: data::PORTAL_HEIGHT,
            },
        ),
    ]
}
