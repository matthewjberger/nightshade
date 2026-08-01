use crate::contract::PortalSide;
use crate::data;
use crate::queries;
use crate::resources::PortalsResources;
use crate::systems::{hud, menu};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::Window;
use nightshade_portal::contract::PortalPlane;
use nightshade_portal::prelude::{Layers, Link, Portal, PortalSettings, Rim};
use nightshade_render::components::culling::{CameraCullingMask, CullingMask};

pub fn initialize(
    mut resources: ResMut<PortalsResources>,
    mut settings: ResMut<RenderSettings>,
    mut portals: ResMut<PortalSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut viewport: ResMut<nightshade_render::resources::viewport::Viewport>,
    mut window: ResMut<Window>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Sky;
    settings.color_grading.tonemap_algorithm =
        nightshade_render::data::config::TonemapAlgorithm::Reinhard;
    spawn_sun(&mut commands);

    let gates = placements();
    let mut solids = Vec::new();
    let mut blocks: Vec<(data::Slab, Option<u32>)> =
        data::hall().into_iter().map(|slab| (slab, None)).collect();
    for (side, plane) in &gates {
        let layer = arch_layer(*side);
        blocks.extend(
            data::arch(plane.center)
                .into_iter()
                .map(|slab| (slab, Some(layer))),
        );
    }
    for (slab, layer) in blocks {
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
                emissive_factor: [
                    slab.color[0] * slab.emissive,
                    slab.color[1] * slab.emissive,
                    slab.color[2] * slab.emissive,
                ],
                roughness: 0.7,
                ..Default::default()
            },
        );
        if let Some(layer) = layer {
            commands.set(entity, CullingMask::new(layer));
        }
        if slab.solid {
            solids.push(queries::solid_from(slab.center, slab.size));
        }
    }
    resources.solids = solids;

    for (side, plane) in gates {
        let index = side.index();
        let surface = commands.spawn((
            Portal(plane),
            side,
            Rim(rim_of(side)),
            Layers(!arch_layer(other_side(side))),
        ));
        resources.surfaces[index] = surface;
    }
    for side in PortalSide::BOTH {
        let other = resources.surfaces[side.index() ^ 1];
        commands.set(resources.surfaces[side.index()], Link(other));
    }

    spawn_avatar(&mut resources, &mut registry, &mut commands);

    let camera = spawn_camera(
        &mut commands,
        queries::eye_position(&resources.player),
        "Player Camera".to_string(),
    );
    commands.set(
        camera,
        CameraCullingMask::new(
            CullingMask::DEFAULT_LAYER | data::BLUE_ARCH_LAYER | data::ORANGE_ARCH_LAYER,
        ),
    );
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
    portals.target_max = data::PORTAL_TARGET_MAX;
    portals.depth = data::LEVELS as u32;

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

fn spawn_avatar(
    resources: &mut PortalsResources,
    registry: &mut MaterialRegistry,
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
}

fn rim_of(side: PortalSide) -> [f32; 4] {
    match side {
        PortalSide::Blue => data::NEAR_RIM,
        PortalSide::Orange => data::FAR_RIM,
    }
}

fn arch_layer(side: PortalSide) -> u32 {
    match side {
        PortalSide::Blue => data::BLUE_ARCH_LAYER,
        PortalSide::Orange => data::ORANGE_ARCH_LAYER,
    }
}

fn other_side(side: PortalSide) -> PortalSide {
    match side {
        PortalSide::Blue => PortalSide::Orange,
        PortalSide::Orange => PortalSide::Blue,
    }
}

fn placements() -> [(PortalSide, PortalPlane); 2] {
    let wall = data::HALL_HALF_LENGTH - data::PORTAL_SURFACE_OFFSET;
    [
        (
            PortalSide::Blue,
            PortalPlane {
                center: Vec3::new(0.0, data::PORTAL_LIFT, -wall),
                normal: Vec3::z(),
                up: Vec3::y(),
                width: data::PORTAL_WIDTH,
                height: data::PORTAL_HEIGHT,
            },
        ),
        (
            PortalSide::Orange,
            PortalPlane {
                center: Vec3::new(0.0, data::PORTAL_LIFT, wall),
                normal: -Vec3::z(),
                up: Vec3::y(),
                width: data::PORTAL_WIDTH,
                height: data::PORTAL_HEIGHT,
            },
        ),
    ]
}
