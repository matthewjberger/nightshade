use crate::contract::{
    XrFrame, XrInput, XrPose, XrRig, XrRigSlot, XrUiAnchor, XrUiPanel, XrUiPointer,
};
use nightshade::prelude::*;
use nightshade_platform::contract::input::Mouse;
use nightshade_platform::prelude::*;
use nightshade_render::components::lines::{Line, Lines};

const PARALLEL_LIMIT: f32 = 1.0e-4;
const BEAM_REACH: f32 = 2.0;
const BEAM_SEGMENTS: usize = 16;
const BEAM_COLOR: [f32; 3] = [0.92, 0.94, 1.0];
const BEAM_ALPHA_AT_HAND: f32 = 0.85;
const BEAM_ALPHA_ON_HIT: f32 = 0.95;

pub fn ui_pointer_system(
    panel: Res<XrUiPanel>,
    frame: Res<XrFrame>,
    xr_input: Res<XrInput>,
    window: Res<Window>,
    rig: Res<XrRigSlot>,
    mut input: ResMut<Input>,
    mut pointer: ResMut<XrUiPointer>,
    storage: &mut nightshade::ecs::Storage,
) {
    let previous_pressed = pointer.pressed;
    pointer.hit = None;
    pointer.hit_world = None;
    pointer.pressed = false;

    let hand = XrRig::hand_state(&xr_input, panel.pointer_hand);
    let hit = panel
        .enabled
        .then(|| panel_hit(&panel, &frame, &xr_input, &window))
        .flatten();

    if let Some(rig) = rig.0.as_ref() {
        draw_beam(
            storage,
            rig.pointer,
            hand.aim,
            hit.as_ref().map(|hit| hit.world),
        );
    }

    let Some(hit) = hit else {
        if previous_pressed {
            apply_button(&mut input.mouse, false, previous_pressed);
        }
        return;
    };

    let pressed = hand.trigger_pressed();
    pointer.hit = Some(hit.pixel);
    pointer.hit_world = Some(hit.world);
    pointer.pressed = pressed;

    let mouse = &mut input.mouse;
    if mouse.position_initialized {
        mouse.position_delta = hit.pixel - mouse.position;
    }
    mouse.position = hit.pixel;
    mouse.position_initialized = true;
    mouse.state.insert(MouseState::MOVED);
    apply_button(mouse, pressed, previous_pressed);
}

fn apply_button(mouse: &mut Mouse, pressed: bool, previous_pressed: bool) {
    mouse.state.set(MouseState::LEFT_CLICKED, pressed);
    if pressed && !previous_pressed {
        mouse.state.insert(MouseState::LEFT_JUST_PRESSED);
    }
    if !pressed && previous_pressed {
        mouse.state.insert(MouseState::LEFT_JUST_RELEASED);
    }
}

struct PanelHit {
    pixel: Vec2,
    world: Vec3,
}

fn panel_hit(
    panel: &XrUiPanel,
    frame: &XrFrame,
    xr_input: &XrInput,
    window: &Window,
) -> Option<PanelHit> {
    let (pixel_width, pixel_height) = window.cached_viewport_size?;
    if pixel_width == 0 || pixel_height == 0 {
        return None;
    }
    let aim = XrRig::hand_state(xr_input, panel.pointer_hand).aim?;
    let panel_pose = panel_pose(panel, frame)?;

    let normal = nalgebra_glm::quat_rotate_vec3(&panel_pose.orientation, &Vec3::z());
    let direction = nalgebra_glm::quat_rotate_vec3(&aim.orientation, &-Vec3::z());
    let denominator = direction.dot(&normal);
    if denominator.abs() < PARALLEL_LIMIT {
        return None;
    }
    let distance = (panel_pose.position - aim.position).dot(&normal) / denominator;
    if distance <= 0.0 {
        return None;
    }

    let world = aim.position + direction * distance;
    let local = nalgebra_glm::quat_rotate_vec3(
        &nalgebra_glm::quat_inverse(&panel_pose.orientation),
        &(world - panel_pose.position),
    );
    let width = panel.width;
    let height = width * pixel_height as f32 / pixel_width as f32;
    let u = (local.x / width + 0.5) * pixel_width as f32;
    let v = (0.5 - local.y / height) * pixel_height as f32;
    let inside = u >= 0.0 && u <= pixel_width as f32 && v >= 0.0 && v <= pixel_height as f32;
    inside.then(|| PanelHit {
        pixel: Vec2::new(u, v),
        world,
    })
}

fn panel_pose(panel: &XrUiPanel, frame: &XrFrame) -> Option<XrPose> {
    match panel.anchor {
        XrUiAnchor::Head => {
            if !frame.tracking_valid {
                return None;
            }
            let forward = nalgebra_glm::quat_rotate_vec3(&frame.head_orientation, &-Vec3::z());
            Some(XrPose {
                position: frame.head_position + forward * panel.distance,
                orientation: frame.head_orientation,
            })
        }
        XrUiAnchor::Stage(pose) => Some(pose),
    }
}

fn draw_beam(
    storage: &mut nightshade::ecs::Storage,
    pointer: Entity,
    aim: Option<XrPose>,
    hit: Option<Vec3>,
) {
    let Some(lines) = storage.get_mut::<Lines>(pointer) else {
        return;
    };
    lines.lines.clear();
    let Some(aim) = aim else {
        lines.mark_dirty();
        return;
    };
    let direction = nalgebra_glm::quat_rotate_vec3(&aim.orientation, &-Vec3::z());
    let (end, alpha_at_end) = match hit {
        Some(hit) => (hit, BEAM_ALPHA_ON_HIT),
        None => (aim.position + direction * BEAM_REACH, 0.0),
    };
    let alpha_at_start = if hit.is_some() {
        BEAM_ALPHA_ON_HIT
    } else {
        BEAM_ALPHA_AT_HAND
    };
    for segment in 0..BEAM_SEGMENTS {
        let from = segment as f32 / BEAM_SEGMENTS as f32;
        let to = (segment + 1) as f32 / BEAM_SEGMENTS as f32;
        let alpha = alpha_at_start + (alpha_at_end - alpha_at_start) * from;
        lines.lines.push(Line {
            start: aim.position.lerp(&end, from),
            end: aim.position.lerp(&end, to),
            color: Vec4::new(BEAM_COLOR[0], BEAM_COLOR[1], BEAM_COLOR[2], alpha),
        });
    }
    lines.mark_dirty();
}
