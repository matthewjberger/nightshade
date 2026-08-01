use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_render::contract::{
    CameraFrameInputs, CameraMatricesInputs, CameraProjectionParams, CameraTarget, RenderViews,
};

fn snapshot(
    storage: &Storage,
    viewport: &nightshade_render::resources::viewport::Viewport,
    window: &nightshade_platform::contract::window::Window,
    settings: &nightshade_render::data::config::RenderSettings,
    debug: &nightshade_render::data::config::DebugDraw,
    entity: nightshade::ecs::Entity,
) -> CameraFrameInputs {
    let row = crate::queries::camera_row(storage.query_ref(), entity);
    let fallbacks = crate::queries::aspect_fallbacks(viewport, window, entity);
    let matrices = row
        .as_ref()
        .and_then(|row| crate::queries::query_camera_matrices(row, fallbacks))
        .map(|matrices| CameraMatricesInputs {
            view: matrices.view,
            projection: matrices.projection,
            cull_projection: matrices.cull_projection,
            camera_position: matrices.camera_position,
        });
    let projection = match crate::queries::resolve_projection(row.as_ref(), fallbacks) {
        crate::queries::ResolvedProjection::Override(override_projection) => {
            CameraProjectionParams {
                z_near: override_projection.z_near,
                z_far: override_projection.z_far,
                y_fov_rad: override_projection.y_fov_rad,
                aspect: override_projection.aspect,
                orthographic: None,
            }
        }
        crate::queries::ResolvedProjection::Perspective(perspective, aspect) => {
            CameraProjectionParams {
                z_near: perspective.z_near,
                z_far: perspective.z_far.unwrap_or(1000.0),
                y_fov_rad: perspective.y_fov_rad,
                aspect,
                orthographic: None,
            }
        }
        crate::queries::ResolvedProjection::Orthographic(orthographic, aspect) => {
            CameraProjectionParams {
                z_near: orthographic.z_near,
                z_far: orthographic.z_far,
                y_fov_rad: std::f32::consts::FRAC_PI_4,
                aspect,
                orthographic: Some((orthographic.half_width(aspect), orthographic.y_mag)),
            }
        }
        crate::queries::ResolvedProjection::None => CameraProjectionParams::default(),
    };
    let shading = storage
        .get::<crate::components::ViewportShading>(entity)
        .copied()
        .unwrap_or_default();
    let post_process = storage
        .get::<crate::components::CameraPostProcess>(entity)
        .copied();
    let culling_mask = storage
        .get::<nightshade_render::components::culling::CameraCullingMask>(entity)
        .copied();
    let environment = storage
        .get::<crate::components::CameraEnvironment>(entity)
        .copied();
    let effective_shading = crate::queries::viewport_shading::effective_shading_for_camera(
        settings,
        debug,
        &shading,
        post_process.as_ref(),
        culling_mask.as_ref(),
        environment.as_ref(),
    );
    CameraFrameInputs {
        entity,
        matrices,
        projection,
        constrained_aspect: storage
            .get::<crate::components::ConstrainedAspect>(entity)
            .map(|constrained| constrained.0),
        effective_shading,
        update_mode: storage
            .get::<nightshade_render::data::config::ViewportUpdateMode>(entity)
            .copied()
            .unwrap_or_default(),
        world_transform: storage
            .get::<nightshade_scene::contract::GlobalTransform>(entity)
            .map(|global| global.0),
        target: storage
            .get::<crate::components::RenderTarget>(entity)
            .map(|target| CameraTarget {
                size: target.size(),
                host_owned: target.is_host_owned(),
            }),
    }
}

pub fn publish_render_views(
    viewport: Res<nightshade_render::resources::viewport::Viewport>,
    window: Res<nightshade_platform::contract::window::Window>,
    active_camera: Res<crate::resources::ActiveCamera>,
    settings: Res<nightshade_render::data::config::RenderSettings>,
    debug: Res<nightshade_render::data::config::DebugDraw>,
    mut views: ResMut<RenderViews>,
    storage: &mut Storage,
) {
    let mut dispatched = viewport.required_cameras.clone();
    for entity in storage.query_entities::<(crate::components::RenderTarget,)>() {
        if !dispatched.contains(&entity) {
            dispatched.push(entity);
        }
    }

    let active = active_camera
        .0
        .map(|entity| snapshot(storage, &viewport, &window, &settings, &debug, entity));
    let dispatched: Vec<CameraFrameInputs> = dispatched
        .into_iter()
        .map(|entity| snapshot(storage, &viewport, &window, &settings, &debug, entity))
        .collect();

    let fallback_shading =
        crate::queries::viewport_shading::effective_shading_from_graphics(&settings, &debug);

    views.active = active;
    views.dispatched = dispatched;
    views.fallback_shading = fallback_shading;
}
