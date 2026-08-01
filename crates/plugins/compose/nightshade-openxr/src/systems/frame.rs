use crate::contract::{XrEye, XrHand};
use crate::contract::{XrFrame, XrRig, XrRigSlot, XrSettings};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_render::prelude::*;

pub fn apply_headset_settings(
    mut settings: nightshade::ecs::system_param::ResMut<RenderSettings>,
    mut pacing: nightshade::ecs::system_param::ResMut<nightshade_render::data::config::FramePacing>,
) {
    settings.anti_aliasing = AntiAliasing::Off;
    pacing.vsync_enabled = false;
    pacing.frame_rate_limit = None;
}

pub fn spawn_xr_rig(
    settings: Option<Res<XrSettings>>,
    mut active_camera: ResMut<ActiveCamera>,
    mut rig: ResMut<XrRigSlot>,
    storage: &mut nightshade::ecs::Storage,
) {
    let origin = spawn_tracked(storage, "XR Origin", None);
    let head = spawn_tracked(storage, "XR Head", Some(origin));
    let left_hand = spawn_tracked(storage, "XR Hand Left", Some(origin));
    let right_hand = spawn_tracked(storage, "XR Hand Right", Some(origin));
    let pointer = spawn_tracked(storage, "XR Pointer", Some(origin));
    storage.set(pointer, Visibility::default());
    storage.set(
        pointer,
        nightshade_render::components::lines::Lines::new_always_on_top(Vec::new()),
    );

    let z_near = settings
        .as_ref()
        .map(|settings| settings.z_near)
        .unwrap_or(0.05);
    let left_eye = spawn_eye(z_near, storage, XrEye::Left, origin);
    let right_eye = spawn_eye(z_near, storage, XrEye::Right, origin);

    let Some((width, height)) = settings
        .as_ref()
        .and_then(|settings| settings.eye_resolution)
    else {
        tracing::error!("no eye resolution: the OpenXR runner did not publish one");
        return;
    };
    for eye in [left_eye, right_eye] {
        storage.set(eye, RenderTarget::host_owned(width, height));
        storage.set(eye, ViewportUpdateMode::Always);
    }

    active_camera.0 = Some(left_eye);

    rig.0 = Some(XrRig {
        origin,
        left_eye,
        right_eye,
        head,
        left_hand,
        right_hand,
        pointer,
    });
}

fn spawn_tracked(
    storage: &mut nightshade::ecs::Storage,
    name: &str,
    parent: Option<Entity>,
) -> Entity {
    let entity = storage.spawn((
        Name(name.to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
    ));
    if let Some(parent) = parent {
        storage.set(entity, nightshade::ecs::ChildOf(parent));
    }
    entity
}

fn spawn_eye(
    z_near: f32,
    storage: &mut nightshade::ecs::Storage,
    eye: XrEye,
    origin: Entity,
) -> Entity {
    let name = match eye {
        XrEye::Left => "XR Eye Left",
        XrEye::Right => "XR Eye Right",
    };
    let entity = storage.spawn((
        Name(name.to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Camera::default(),
    ));
    storage.set(entity, nightshade::ecs::ChildOf(origin));

    if let Some(camera) = storage.get_mut::<Camera>(entity)
        && let Projection::Perspective(perspective) = &mut camera.projection
    {
        perspective.z_near = z_near;
    }

    storage.set(
        entity,
        ProjectionOverride::from_tangents(
            -1.0,
            1.0,
            1.0,
            -1.0,
            z_near,
            ProjectionOverride::DEFAULT_FAR,
        ),
    );
    entity
}

pub fn xr_pose_system(
    frame: Res<XrFrame>,
    rig: Res<XrRigSlot>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(rig) = rig.0 else {
        return;
    };
    if !frame.tracking_valid {
        return;
    }

    set_pose(
        storage,
        rig.head,
        frame.head_position,
        frame.head_orientation,
    );

    for eye in XrEye::ALL {
        let entity = rig.eye(eye);
        let Some(view) = frame.views.get(eye.layer() as usize) else {
            continue;
        };
        set_pose(storage, entity, view.position, view.orientation);

        let [tan_left, tan_right, tan_up, tan_down] = view.tangents;
        let z_near = storage
            .get::<Camera>(entity)
            .and_then(|camera| match &camera.projection {
                Projection::Perspective(perspective) => Some(perspective.z_near),
                Projection::Orthographic(_) => None,
            })
            .unwrap_or(0.01);
        storage.set(
            entity,
            ProjectionOverride::from_tangents(
                tan_left,
                tan_right,
                tan_up,
                tan_down,
                z_near,
                ProjectionOverride::DEFAULT_FAR,
            ),
        );
    }
}

fn set_pose(
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
    translation: Vec3,
    rotation: Quat,
) {
    if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
        transform.translation = translation;
        transform.rotation = rotation;
    }
}

pub fn apply_hand_pose(
    rig: &XrRigSlot,
    storage: &mut nightshade::ecs::Storage,
    hand: XrHand,
    translation: Vec3,
    rotation: Quat,
) {
    let Some(rig) = rig.0 else {
        return;
    };
    set_pose(storage, rig.hand(hand), translation, rotation);
}
