use crate::components::{Eyes, Layers, Link, Portal, Rim};
use crate::contract::{PortalFrame, PortalOrder, PortalPlane};
use crate::data;
use crate::queries;
use crate::resources::PortalSettings;
use nightshade::prelude::*;
use nightshade_camera::prelude::{
    Camera, PerspectiveCamera, Projection, ProjectionOverride, RenderTarget,
};
use nightshade_platform::contract::time::Time;
use nightshade_render::components::culling::CameraCullingMask;
use nightshade_render::data::config::ViewportUpdateMode;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::contract::primitives::Name;
use nightshade_scene::contract::{ChildOf, GlobalTransform, LocalTransform};

struct Opening {
    entity: Entity,
    plane: PortalPlane,
    link: Entity,
    chain: Vec<Entity>,
    rim: [f32; 4],
}

pub fn open(
    settings: Res<PortalSettings>,
    mut viewport: ResMut<Viewport>,
    mut commands: Commands,
    portals: Query<(&Portal, Option<&Eyes>, Option<&Layers>)>,
) {
    let mut wanting = Vec::new();
    portals.for_each(|entity, (_portal, eyes, layers)| {
        if eyes.is_none() {
            wanting.push((entity, layers.copied().unwrap_or_default()));
        }
    });
    if wanting.is_empty() {
        return;
    }

    let depth = settings.depth.clamp(1, data::MAX_DEPTH);
    let target = queries::portal_target(
        (data::FALLBACK_WIDTH, data::FALLBACK_HEIGHT),
        settings.target_max,
    );
    for (portal, layers) in wanting {
        let chain: Vec<Entity> = (0..depth)
            .map(|level| {
                commands.spawn((
                    Name(format!("{} {level}", data::EYE_NAME)),
                    LocalTransform::default(),
                    GlobalTransform::default(),
                    Camera {
                        projection: Projection::Perspective(PerspectiveCamera {
                            aspect_ratio: None,
                            y_fov_rad: settings.field_of_view,
                            z_far: Some(settings.far_plane),
                            z_near: settings.near_plane,
                        }),
                        smoothing: None,
                    },
                    RenderTarget::renderer_owned(target.0, target.1),
                    ProjectionOverride::default(),
                    CameraCullingMask(layers.0),
                    ViewportUpdateMode::Always,
                ))
            })
            .collect();

        for eye in &chain {
            viewport.required_cameras.insert(0, *eye);
        }
        commands.set(portal, Eyes(chain));
    }
}

pub fn aim(
    time: Res<Time>,
    settings: Res<PortalSettings>,
    viewport: Res<Viewport>,
    mut frame: ResMut<PortalFrame>,
    mut commands: Commands,
    portals: Query<(&Portal, &Link, &Eyes, &Rim)>,
    mut eyes: Query<(&RenderTarget, &ViewportUpdateMode)>,
    mut transforms: Query<(&LocalTransform, Option<&GlobalTransform>, Option<&ChildOf>)>,
) {
    frame.orders.clear();
    frame.elapsed = time.uptime_seconds as f32;
    if settings.viewer == Entity::default() {
        return;
    }

    let mut viewer = Mat4::identity();
    let mut standing = false;
    transforms.for_entity(settings.viewer, |(local, global, parent)| {
        viewer = match (parent, global) {
            (Some(_), Some(global)) => global.0,
            _ => local.as_matrix(),
        };
        standing = true;
    });
    if !standing {
        return;
    }

    let mut gathered: Vec<Opening> = Vec::new();
    portals.for_each(|entity, (portal, link, chain, rim)| {
        gathered.push(Opening {
            entity,
            plane: portal.0,
            link: link.0,
            chain: chain.0.clone(),
            rim: rim.0,
        });
    });

    let size = queries::viewport_size(&viewport);
    let aspect = size.0 as f32 / size.1.max(1) as f32;
    let target = queries::portal_target(size, settings.target_max);
    let standpoint = queries::position_of(&viewer);

    for opening in &gathered {
        let source = &opening.plane;
        let chain = &opening.chain;
        let Some(destination) = gathered
            .iter()
            .find(|held| held.entity == opening.link)
            .map(|held| held.plane)
        else {
            continue;
        };

        let seen = queries::portal_is_seen(source, standpoint, settings.max_distance);
        let wanted = if seen {
            ViewportUpdateMode::Always
        } else {
            ViewportUpdateMode::Disabled
        };
        for camera in chain {
            let mut held = ((0, 0), ViewportUpdateMode::Always);
            eyes.for_entity(*camera, |(held_target, mode)| {
                held = (held_target.size(), *mode)
            });
            if held.1 != wanted {
                commands.set(*camera, wanted);
            }
            if seen && held.0 != target {
                commands.set(*camera, RenderTarget::renderer_owned(target.0, target.1));
            }
        }
        if !seen {
            continue;
        }

        let carry = queries::carry_matrix(source, &destination);
        let model = queries::portal_model(source);
        let mut stepped = Mat4::identity();
        for (level, camera) in chain.iter().enumerate() {
            let world = carry * stepped * viewer;
            let gate = queries::carried_portal(&destination, &stepped);
            commands.set(*camera, LocalTransform::from_matrix(&world));
            commands.set(
                *camera,
                queries::portal_projection(
                    &world,
                    &gate,
                    aspect,
                    settings.field_of_view,
                    settings.near_plane,
                    settings.far_plane,
                    settings.plane_bias,
                ),
            );
            stepped = carry * stepped;

            frame.orders.push(PortalOrder {
                camera: if level == 0 {
                    settings.viewer
                } else {
                    chain[level - 1]
                },
                source: *camera,
                model,
                rim: opening.rim,
            });
        }
    }
}
