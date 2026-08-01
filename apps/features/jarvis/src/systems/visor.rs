use crate::components::{Health, Hit, Label};
use crate::queries::{ease, heading_degrees, origin_pose, to_stage, to_world};
use crate::resources::Jarvis;
use crate::theme::{
    AMBER, AMBER_DIM, BOOT_LINES, BOOT_SECONDS, BRACKET_ARM, CORNER_ARM, CYAN, CYAN_DIM,
    CYAN_FAINT, HEAVY, LINE, MARGIN, MAX_SPEED, PROXIMITY_METERS, RADAR_RADIUS, RADAR_RANGE,
    REACTOR_RADIUS, RED, RED_DIM, RETICLE_INNER, RETICLE_OUTER, RETICLE_TICKS, TEXT_BODY,
    TEXT_LARGE, TEXT_SMALL, THIN, WHITE,
};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_openxr::contract::{
    XrFrame, XrHand, XrInput, XrPose, XrRig, XrUiAnchor, XrUiPanel, XrUiPointer,
};
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::*;
use std::f32::consts::{PI, TAU};

const DEFAULT_PIXELS: (u32, u32) = (800, 600);
const LOCK_RADIUS: f32 = 90.0;
const COMPASS_HALF_SPAN: f32 = 70.0;
const COMPASS_WIDTH: f32 = 360.0;
const TAPE_HALF_SPAN: f32 = 2.0;
const TAPE_HEIGHT: f32 = 220.0;
const REACTOR_BASE: f32 = 0.74;
const INTEGRITY: f32 = 0.98;
const GAUGE_WIDTH: f32 = 120.0;
const GAUGE_HEIGHT: f32 = 6.0;
const LADDER_PIXELS_PER_DEGREE: f32 = 5.5;
const LADDER_HALF_WIDTH: f32 = 60.0;
const LADDER_GAP: f32 = 150.0;
const MARKER_DISTANCE: f32 = 6.0;
const STREAK_COUNT: usize = 14;
const GRAVITY: f32 = 9.81;

pub fn draw_visor(
    mut jarvis: ResMut<Jarvis>,
    frame: Res<XrFrame>,
    xr_input: Res<XrInput>,
    pointer: Res<XrUiPointer>,
    panel: Res<XrUiPanel>,
    rig: Option<Res<XrRig>>,
    window: Res<Window>,
    time: Res<Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    let canvas = jarvis.handles.canvas;
    if canvas == Entity::default() {
        return;
    }
    let seconds = time.uptime_milliseconds as f32 / 1000.0;
    let boot_start = *jarvis.boot_start.get_or_insert(seconds);
    let booted = seconds - boot_start;
    let boot = ease(booted / BOOT_SECONDS);

    let origin = rig
        .as_deref()
        .map(|rig| origin_pose(storage, rig))
        .unwrap_or_default();
    let (pixel_width, pixel_height) = window.cached_viewport_size.unwrap_or(DEFAULT_PIXELS);
    let size = vec2(pixel_width as f32, pixel_height as f32);
    let center = size * 0.5;

    let drones: Vec<DroneMark> = storage
        .query_entities::<(Label, LocalTransform)>()
        .filter_map(|entity| {
            let label = storage.get::<Label>(entity)?.0;
            let position = storage.get::<LocalTransform>(entity)?.translation;
            Some(DroneMark {
                entity,
                label,
                alive: crate::queries::drone_alive(storage, entity),
                hit: storage
                    .get::<Hit>(entity)
                    .map(|hit| hit.0)
                    .unwrap_or_default(),
                health: storage
                    .get::<Health>(entity)
                    .map(|health| health.0)
                    .unwrap_or_default(),
                stage: to_stage(&origin, position),
            })
        })
        .collect();

    ui_canvas_clear(storage, canvas);
    let mut visor = Visor {
        storage,
        canvas,
        size,
        center,
        seconds,
        boot,
    };

    let world_heading = heading_degrees(&(origin.orientation * frame.head_orientation));
    let altitude = to_world(&origin, frame.head_position).y;
    let speed = jarvis.velocity.norm();
    let locked_center = nearest_to_center(&visor, &frame, &panel, &drones);
    let arming = jarvis.targeting.arming_hand.is_some();

    frame_corners(&mut visor);
    speed_streaks(&mut visor, speed);
    pitch_ladder(&mut visor, &frame);
    reticle(&mut visor, locked_center.is_some() || arming);
    flight_marker(&mut visor, &frame, &panel, &origin, &jarvis.velocity);
    compass(&mut visor, world_heading);
    reactor(&mut visor, &xr_input);
    telemetry(
        &mut visor,
        &frame,
        altitude,
        speed,
        jarvis.acceleration,
        world_heading,
    );
    altitude_tape(&mut visor, altitude);
    hands(&mut visor, &xr_input);
    boot_log(&mut visor, booted, jarvis.flight_mode);
    pointer_readout(&mut visor, &pointer);
    radar(&mut visor, &frame, &drones, &jarvis.targeting.locks);
    counters(&mut visor, &drones, &jarvis);
    targets(
        &mut visor,
        &frame,
        &panel,
        &drones,
        &jarvis.targeting.locks,
        locked_center,
    );
    warnings(&mut visor, &frame, &drones, &jarvis);
}

struct DroneMark {
    entity: Entity,
    label: usize,
    alive: bool,
    hit: f32,
    health: i32,
    stage: Vec3,
}

struct Visor<'a> {
    storage: &'a mut nightshade::ecs::Storage,
    canvas: Entity,
    size: Vec2,
    center: Vec2,
    seconds: f32,
    boot: f32,
}

fn faded(color: Vec4, alpha_scale: f32) -> Vec4 {
    Vec4::new(
        color.x,
        color.y,
        color.z,
        color.w * alpha_scale.clamp(0.0, 1.0),
    )
}

fn blink(seconds: f32, rate: f32) -> f32 {
    ((seconds * rate).sin() * 0.5 + 0.5) * 0.6 + 0.4
}

fn frame_corners(visor: &mut Visor<'_>) {
    let arm = CORNER_ARM * visor.boot;
    let color = faded(CYAN_DIM, visor.boot);
    let width = visor.size.x;
    let height = visor.size.y;
    let corners = [
        (vec2(MARGIN, MARGIN), vec2(1.0, 1.0)),
        (vec2(width - MARGIN, MARGIN), vec2(-1.0, 1.0)),
        (vec2(MARGIN, height - MARGIN), vec2(1.0, -1.0)),
        (vec2(width - MARGIN, height - MARGIN), vec2(-1.0, -1.0)),
    ];
    for (corner, direction) in corners {
        ui_canvas_line(
            visor.storage,
            visor.canvas,
            corner,
            corner + vec2(arm * direction.x, 0.0),
            LINE,
            color,
        );
        ui_canvas_line(
            visor.storage,
            visor.canvas,
            corner,
            corner + vec2(0.0, arm * direction.y),
            LINE,
            color,
        );
    }
    let scan = (visor.seconds * 0.35).fract();
    let scan_y = MARGIN + (height - 2.0 * MARGIN) * scan;
    let scan_color = faded(CYAN_FAINT, visor.boot);
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        vec2(MARGIN, scan_y),
        vec2(width - MARGIN, scan_y),
        THIN,
        scan_color,
    );
}

fn speed_streaks(visor: &mut Visor<'_>, speed: f32) {
    let strength = ((speed - 5.0) / (MAX_SPEED - 5.0)).clamp(0.0, 1.0);
    if strength <= 0.0 {
        return;
    }
    let color = faded(CYAN_DIM, strength * 0.9);
    let inner = visor.size.y * 0.42;
    let outer = visor.size.y * (0.46 + 0.16 * strength);
    for index in 0..STREAK_COUNT {
        let angle = index as f32 * (TAU / STREAK_COUNT as f32) + visor.seconds * 0.9;
        let direction = vec2(angle.cos(), angle.sin());
        ui_canvas_line(
            visor.storage,
            visor.canvas,
            visor.center + direction * inner,
            visor.center + direction * outer,
            THIN,
            color,
        );
    }
}

fn pitch_ladder(visor: &mut Visor<'_>, frame: &XrFrame) {
    let forward = nalgebra_glm::quat_rotate_vec3(&frame.head_orientation, &-Vec3::z());
    let right = nalgebra_glm::quat_rotate_vec3(&frame.head_orientation, &Vec3::x());
    let pitch = forward.y.clamp(-1.0, 1.0).asin().to_degrees();
    let roll = right.y.clamp(-1.0, 1.0).asin();
    let color = faded(CYAN_DIM, visor.boot);
    let rotate = |point: Vec2, center: Vec2| -> Vec2 {
        let offset = point - center;
        vec2(
            center.x + offset.x * roll.cos() - offset.y * roll.sin(),
            center.y + offset.x * roll.sin() + offset.y * roll.cos(),
        )
    };
    let mut degrees = -30;
    while degrees <= 30 {
        let offset_y = (degrees as f32 - pitch) * LADDER_PIXELS_PER_DEGREE;
        if offset_y.abs() < 170.0 {
            let horizon = degrees == 0;
            let half_width = if horizon {
                LADDER_HALF_WIDTH * 2.2
            } else {
                LADDER_HALF_WIDTH
            };
            let y = visor.center.y - offset_y;
            for side in [-1.0, 1.0] {
                let inner = vec2(visor.center.x + side * LADDER_GAP, y);
                let outer = vec2(visor.center.x + side * (LADDER_GAP + half_width), y);
                ui_canvas_line(
                    visor.storage,
                    visor.canvas,
                    rotate(inner, visor.center),
                    rotate(outer, visor.center),
                    if horizon { LINE } else { THIN },
                    color,
                );
                if !horizon {
                    let label = format!("{degrees:>3}");
                    let at = vec2(
                        visor.center.x + side * (LADDER_GAP + half_width + 8.0) - 12.0,
                        y - 6.0,
                    );
                    ui_canvas_text(
                        visor.storage,
                        visor.canvas,
                        &label,
                        rotate(at, visor.center),
                        TEXT_SMALL,
                        color,
                    );
                }
            }
        }
        degrees += 10;
    }
}

fn reticle(visor: &mut Visor<'_>, hot: bool) {
    let center = visor.center;
    let boot = visor.boot;
    let outer = RETICLE_OUTER * boot;
    let inner = RETICLE_INNER * boot;
    let ticks = RETICLE_TICKS * boot;
    let (color, dim) = if hot {
        (faded(AMBER, boot), faded(AMBER_DIM, boot))
    } else {
        (faded(CYAN, boot), faded(CYAN_DIM, boot))
    };

    let spin = visor.seconds * 0.4;
    for index in 0..4 {
        let start = spin + index as f32 * (TAU / 4.0);
        ui_canvas_arc(
            visor.storage,
            visor.canvas,
            center,
            outer,
            (start, start + TAU / 4.0 * 0.62),
            LINE,
            color,
        );
    }
    let counter_spin = -visor.seconds * 0.7;
    for index in 0..3 {
        let start = counter_spin + index as f32 * (TAU / 3.0);
        ui_canvas_arc(
            visor.storage,
            visor.canvas,
            center,
            inner,
            (start, start + TAU / 3.0 * 0.5),
            HEAVY,
            dim,
        );
    }
    for index in 0..24 {
        let angle = index as f32 * (TAU / 24.0);
        let long = index % 6 == 0;
        let length = if long { 12.0 } else { 6.0 };
        let direction = vec2(angle.cos(), angle.sin());
        ui_canvas_line(
            visor.storage,
            visor.canvas,
            center + direction * ticks,
            center + direction * (ticks + length),
            if long { LINE } else { THIN },
            if long { color } else { dim },
        );
    }
    let gap = 10.0;
    let reach = 34.0;
    for direction in [
        vec2(1.0, 0.0),
        vec2(-1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(0.0, -1.0),
    ] {
        ui_canvas_line(
            visor.storage,
            visor.canvas,
            center + direction * gap,
            center + direction * (gap + reach) * boot,
            LINE,
            color,
        );
    }
    ui_canvas_circle(visor.storage, visor.canvas, center, 2.5 * boot, WHITE);
}

fn flight_marker(
    visor: &mut Visor<'_>,
    frame: &XrFrame,
    panel: &XrUiPanel,
    origin: &XrPose,
    velocity: &Vec3,
) {
    let speed = velocity.norm();
    if speed < 0.6 {
        return;
    }
    let Some(panel_pose) = panel_pose(panel, frame) else {
        return;
    };
    let direction_stage = crate::queries::direction_to_stage(origin, velocity / speed);
    let ahead = frame.head_position + direction_stage * MARKER_DISTANCE;
    let Some(marker) = project_through_head(ahead, frame, panel, &panel_pose, visor.size) else {
        return;
    };
    let color = faded(CYAN, visor.boot);
    ui_canvas_arc(
        visor.storage,
        visor.canvas,
        marker,
        7.0,
        (0.0, TAU),
        LINE,
        color,
    );
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        marker + vec2(7.0, 0.0),
        marker + vec2(19.0, 0.0),
        LINE,
        color,
    );
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        marker - vec2(7.0, 0.0),
        marker - vec2(19.0, 0.0),
        LINE,
        color,
    );
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        marker - vec2(0.0, 7.0),
        marker - vec2(0.0, 15.0),
        LINE,
        color,
    );
    let readout = format!("{speed:.1}");
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        &readout,
        marker + vec2(22.0, 6.0),
        TEXT_SMALL,
        color,
    );
}

fn compass(visor: &mut Visor<'_>, heading: f32) {
    let top = MARGIN + 18.0;
    let center_x = visor.center.x;
    let color = faded(CYAN, visor.boot);
    let dim = faded(CYAN_DIM, visor.boot);
    let half_width = COMPASS_WIDTH * 0.5 * visor.boot;
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        vec2(center_x - half_width, top),
        vec2(center_x + half_width, top),
        THIN,
        dim,
    );
    let scale = half_width / COMPASS_HALF_SPAN;
    let first = ((heading - COMPASS_HALF_SPAN) / 10.0).floor() as i32 * 10;
    let last = ((heading + COMPASS_HALF_SPAN) / 10.0).ceil() as i32 * 10;
    let mut degrees = first;
    while degrees <= last {
        let offset = (degrees as f32 - heading) * scale;
        if offset.abs() <= half_width {
            let x = center_x + offset;
            let major = degrees.rem_euclid(30) == 0;
            let height = if major { 10.0 } else { 5.0 };
            ui_canvas_line(
                visor.storage,
                visor.canvas,
                vec2(x, top),
                vec2(x, top + height),
                if major { LINE } else { THIN },
                if major { color } else { dim },
            );
            if major {
                let label = match degrees.rem_euclid(360) {
                    0 => "N".to_string(),
                    90 => "E".to_string(),
                    180 => "S".to_string(),
                    270 => "W".to_string(),
                    other => other.to_string(),
                };
                ui_canvas_text(
                    visor.storage,
                    visor.canvas,
                    &label,
                    vec2(x - 6.0, top + 13.0),
                    TEXT_SMALL,
                    color,
                );
            }
        }
        degrees += 10;
    }
    ui_canvas_polyline(
        visor.storage,
        visor.canvas,
        &[
            vec2(center_x - 6.0, top - 12.0),
            vec2(center_x, top - 3.0),
            vec2(center_x + 6.0, top - 12.0),
        ],
        LINE,
        AMBER,
        false,
    );
    let readout = format!("{:03}", heading.round() as i32 % 360);
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        &readout,
        vec2(center_x - 13.0, top - 32.0),
        TEXT_BODY,
        WHITE,
    );
}

fn reactor(visor: &mut Visor<'_>, xr_input: &XrInput) {
    let center = vec2(visor.size.x * 0.125, visor.size.y * 0.47);
    let pulse = (visor.seconds * 2.2).sin() * 0.5 + 0.5;
    let power = (REACTOR_BASE + pulse * 0.06) * visor.boot;
    let radius = REACTOR_RADIUS * visor.boot;
    let color = faded(CYAN, visor.boot);
    let dim = faded(CYAN_DIM, visor.boot);
    let glow = faded(CYAN_FAINT, visor.boot * (0.6 + pulse * 0.4));

    ui_canvas_circle(
        visor.storage,
        visor.canvas,
        center,
        radius + 8.0 + pulse * 4.0,
        glow,
    );
    ui_canvas_arc(
        visor.storage,
        visor.canvas,
        center,
        radius,
        (0.0, TAU),
        THIN,
        dim,
    );
    let start = -PI / 2.0;
    ui_canvas_arc(
        visor.storage,
        visor.canvas,
        center,
        radius,
        (start, start + TAU * power),
        HEAVY,
        color,
    );
    ui_canvas_arc(
        visor.storage,
        visor.canvas,
        center,
        radius - 10.0,
        (visor.seconds, visor.seconds + PI * 0.6),
        LINE,
        dim,
    );
    let percent = format!("{:.0}%", power * 100.0);
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        &percent,
        center - vec2(16.0, 9.0),
        TEXT_LARGE,
        WHITE,
    );
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        "ARC REACTOR",
        vec2(center.x - 40.0, center.y - radius - 30.0),
        TEXT_SMALL,
        color,
    );

    let gauge_origin = vec2(MARGIN + 14.0, center.y + radius + 28.0);
    gauge(visor, gauge_origin, "INTEGRITY", INTEGRITY, color, dim);
    let left = XrRig::hand_state(xr_input, XrHand::Left);
    let right = XrRig::hand_state(xr_input, XrHand::Right);
    gauge(
        visor,
        gauge_origin + vec2(0.0, 30.0),
        "THRUST L",
        left.trigger,
        AMBER,
        dim,
    );
    gauge(
        visor,
        gauge_origin + vec2(0.0, 60.0),
        "THRUST R",
        right.trigger,
        AMBER,
        dim,
    );
}

fn gauge(visor: &mut Visor<'_>, origin: Vec2, label: &str, value: f32, color: Vec4, dim: Vec4) {
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        label,
        origin,
        TEXT_SMALL,
        color,
    );
    let bar = origin + vec2(0.0, 16.0);
    let width = GAUGE_WIDTH * visor.boot;
    ui_canvas_rect(
        visor.storage,
        visor.canvas,
        bar,
        vec2(width, GAUGE_HEIGHT),
        faded(dim, 0.5),
        0.0,
    );
    ui_canvas_rect(
        visor.storage,
        visor.canvas,
        bar,
        vec2(width * value.clamp(0.0, 1.0), GAUGE_HEIGHT),
        color,
        0.0,
    );
    let readout = format!("{:.0}", value.clamp(0.0, 1.0) * 100.0);
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        &readout,
        bar + vec2(width + 8.0, -5.0),
        TEXT_SMALL,
        WHITE,
    );
}

fn telemetry(
    visor: &mut Visor<'_>,
    frame: &XrFrame,
    altitude: f32,
    speed: f32,
    acceleration: f32,
    heading: f32,
) {
    let color = faded(CYAN, visor.boot);
    let forward = nalgebra_glm::quat_rotate_vec3(&frame.head_orientation, &-Vec3::z());
    let pitch = forward.y.clamp(-1.0, 1.0).asin().to_degrees();
    let lines = [
        format!("ALT  {altitude:>6.2} M"),
        format!("VEL  {speed:>6.2} M/S"),
        format!("ACC  {:>6.2} G", acceleration / GRAVITY),
        format!("PIT  {pitch:>6.1} DEG"),
        format!("HDG  {heading:>6.0} DEG"),
    ];
    let origin = vec2(visor.size.x - MARGIN - 150.0, MARGIN + 60.0);
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        "TELEMETRY",
        origin,
        TEXT_SMALL,
        color,
    );
    for (index, line) in lines.iter().enumerate() {
        let reveal = ease(visor.boot * 5.0 - index as f32);
        if reveal <= 0.0 {
            continue;
        }
        let faded = faded(WHITE, reveal);
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            line,
            origin + vec2(0.0, 20.0 + index as f32 * 18.0),
            TEXT_BODY,
            faded,
        );
    }
    let tracking = if frame.tracking_valid {
        ("TRACKING LOCKED", color)
    } else {
        ("TRACKING LOST", RED)
    };
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        tracking.0,
        origin + vec2(0.0, 118.0),
        TEXT_SMALL,
        tracking.1,
    );
}

fn altitude_tape(visor: &mut Visor<'_>, altitude: f32) {
    let x = visor.size.x - MARGIN - 18.0;
    let center_y = visor.center.y;
    let half_height = TAPE_HEIGHT * 0.5 * visor.boot;
    let color = faded(CYAN, visor.boot);
    let dim = faded(CYAN_DIM, visor.boot);
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        vec2(x, center_y - half_height),
        vec2(x, center_y + half_height),
        THIN,
        dim,
    );
    let scale = half_height / TAPE_HALF_SPAN;
    let first = ((altitude - TAPE_HALF_SPAN) / 0.5).floor() as i32;
    let last = ((altitude + TAPE_HALF_SPAN) / 0.5).ceil() as i32;
    let mut step = first;
    while step <= last {
        let value = step as f32 * 0.5;
        let offset = (altitude - value) * scale;
        if offset.abs() <= half_height {
            let y = center_y + offset;
            let major = step.rem_euclid(2) == 0;
            let length = if major { 10.0 } else { 5.0 };
            ui_canvas_line(
                visor.storage,
                visor.canvas,
                vec2(x - length, y),
                vec2(x, y),
                if major { LINE } else { THIN },
                if major { color } else { dim },
            );
            if major {
                let label = format!("{value:.0}");
                ui_canvas_text(
                    visor.storage,
                    visor.canvas,
                    &label,
                    vec2(x - 26.0, y - 6.0),
                    TEXT_SMALL,
                    color,
                );
            }
        }
        step += 1;
    }
    ui_canvas_polyline(
        visor.storage,
        visor.canvas,
        &[
            vec2(x + 12.0, center_y - 6.0),
            vec2(x + 3.0, center_y),
            vec2(x + 12.0, center_y + 6.0),
        ],
        LINE,
        AMBER,
        false,
    );
}

fn hands(visor: &mut Visor<'_>, xr_input: &XrInput) {
    let color = faded(CYAN, visor.boot);
    let dim = faded(CYAN_DIM, visor.boot);
    let base = vec2(visor.center.x, visor.size.y - MARGIN - 26.0);
    for (hand, offset, label) in [(XrHand::Left, -70.0, "L"), (XrHand::Right, 70.0, "R")] {
        let state = XrRig::hand_state(xr_input, hand);
        let origin = base + vec2(offset - 20.0, 0.0);
        let outline = if state.tracked { color } else { dim };
        ui_canvas_rect_stroke(
            visor.storage,
            visor.canvas,
            origin,
            vec2(40.0, 18.0),
            outline,
            THIN,
            3.0,
        );
        let fill = if state.grip_pressed() { AMBER } else { color };
        ui_canvas_rect(
            visor.storage,
            visor.canvas,
            origin + vec2(2.0, 2.0),
            vec2(36.0 * state.trigger, 14.0),
            fill,
            0.0,
        );
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            label,
            origin + vec2(16.0, 22.0),
            TEXT_SMALL,
            outline,
        );
        let arc_center = origin + vec2(20.0, 9.0);
        let sweep = TAU * state.trigger;
        let arc_color = faded(AMBER, state.trigger);
        ui_canvas_arc(
            visor.storage,
            visor.canvas,
            arc_center,
            28.0,
            (-PI / 2.0, -PI / 2.0 + sweep),
            LINE,
            arc_color,
        );
    }
}

fn boot_log(visor: &mut Visor<'_>, booted: f32, flight_mode: bool) {
    let origin = vec2(MARGIN + 14.0, visor.size.y - MARGIN - 20.0);
    let color = faded(CYAN, visor.boot);
    let mut shown = 0;
    for (index, (line, at)) in BOOT_LINES.iter().enumerate() {
        if booted < *at {
            break;
        }
        let reveal = ease((booted - at) * 3.0);
        let visible = reveal.min(1.0);
        let text_color = if index + 1 == BOOT_LINES.len() {
            faded(WHITE, visible)
        } else {
            faded(color, visible)
        };
        let row = origin - vec2(0.0, (BOOT_LINES.len() - index) as f32 * 16.0);
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            line,
            row,
            TEXT_SMALL,
            text_color,
        );
        shown += 1;
    }
    if shown == BOOT_LINES.len() {
        let blink = blink(visor.seconds, 1.5);
        let mode = if flight_mode {
            "MODE  FLIGHT"
        } else {
            "MODE  HOVER"
        };
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            mode,
            origin,
            TEXT_SMALL,
            faded(AMBER, blink),
        );
    }
}

fn pointer_readout(visor: &mut Visor<'_>, pointer: &XrUiPointer) {
    let origin = vec2(visor.size.x - MARGIN - 150.0, visor.size.y - MARGIN - 118.0);
    let color = faded(CYAN_DIM, visor.boot);
    match pointer.hit {
        Some(hit) => {
            let state = if pointer.pressed { "ENGAGE" } else { "AIM" };
            let text = format!(
                "PTR  {:>4} {:>4}  {state}",
                hit.x.round() as i32,
                hit.y.round() as i32
            );
            ui_canvas_text(
                visor.storage,
                visor.canvas,
                &text,
                origin,
                TEXT_SMALL,
                faded(CYAN, visor.boot),
            );
        }
        None => ui_canvas_text(
            visor.storage,
            visor.canvas,
            "PTR ---- ----",
            origin,
            TEXT_SMALL,
            color,
        ),
    }
}

fn radar(visor: &mut Visor<'_>, frame: &XrFrame, drones: &[DroneMark], locks: &[Entity]) {
    let radius = RADAR_RADIUS * visor.boot;
    let center = vec2(MARGIN + 14.0 + RADAR_RADIUS, MARGIN + 44.0 + RADAR_RADIUS);
    let color = faded(CYAN, visor.boot);
    let dim = faded(CYAN_DIM, visor.boot);
    let faint = faded(CYAN_FAINT, visor.boot);
    ui_canvas_circle(visor.storage, visor.canvas, center, radius, faint);
    ui_canvas_arc(
        visor.storage,
        visor.canvas,
        center,
        radius,
        (0.0, TAU),
        THIN,
        dim,
    );
    ui_canvas_arc(
        visor.storage,
        visor.canvas,
        center,
        radius * 0.5,
        (0.0, TAU),
        THIN,
        faint,
    );
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        center - vec2(0.0, radius),
        center + vec2(0.0, radius),
        THIN,
        faint,
    );
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        center - vec2(radius, 0.0),
        center + vec2(radius, 0.0),
        THIN,
        faint,
    );
    let sweep = visor.seconds * 1.6;
    ui_canvas_line(
        visor.storage,
        visor.canvas,
        center,
        center + vec2(sweep.cos(), sweep.sin()) * radius,
        LINE,
        faded(CYAN, 0.5 * visor.boot),
    );

    let forward = nalgebra_glm::quat_rotate_vec3(&frame.head_orientation, &-Vec3::z());
    let flat_forward = Vec3::new(forward.x, 0.0, forward.z);
    let flat_forward = if flat_forward.norm() > 1.0e-3 {
        flat_forward.normalize()
    } else {
        -Vec3::z()
    };
    let right = Vec3::new(-flat_forward.z, 0.0, flat_forward.x);
    let scale = radius / RADAR_RANGE;
    for drone in drones.iter().filter(|drone| drone.alive) {
        let relative = drone.stage - frame.head_position;
        let mut dot = vec2(relative.dot(&right), -relative.dot(&flat_forward)) * scale;
        let length = dot.norm();
        if length > radius - 3.0 {
            dot *= (radius - 3.0) / length;
        }
        let locked = locks.contains(&drone.entity);
        let mark = if locked { RED } else { AMBER };
        ui_canvas_circle(
            visor.storage,
            visor.canvas,
            center + dot,
            if locked { 3.5 } else { 2.5 },
            mark,
        );
    }
    ui_canvas_polyline(
        visor.storage,
        visor.canvas,
        &[
            center + vec2(-4.0, 4.0),
            center + vec2(0.0, -5.0),
            center + vec2(4.0, 4.0),
        ],
        LINE,
        color,
        true,
    );
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        "RADAR",
        vec2(center.x - 18.0, center.y - radius - 16.0),
        TEXT_SMALL,
        color,
    );
}

fn counters(visor: &mut Visor<'_>, drones: &[DroneMark], jarvis: &Jarvis) {
    let alive = drones.iter().filter(|drone| drone.alive).count();
    let color = faded(CYAN, visor.boot);
    let origin = vec2(MARGIN + 14.0, MARGIN + 6.0);
    let text = format!(
        "TARGETS {alive:>2}   DOWN {:>2}   SHOTS {:>3}",
        jarvis.kills, jarvis.shots
    );
    ui_canvas_text(
        visor.storage,
        visor.canvas,
        &text,
        origin,
        TEXT_SMALL,
        color,
    );
    if jarvis.kill_flash > 0.0 {
        let flash = faded(AMBER, jarvis.kill_flash);
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            "TARGET DESTROYED",
            vec2(visor.center.x - 62.0, visor.center.y + RETICLE_TICKS + 30.0),
            TEXT_BODY,
            flash,
        );
    }
    if jarvis.targeting.launched > 0.0 {
        let flash = faded(RED, jarvis.targeting.launched);
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            "MISSILES AWAY",
            vec2(visor.center.x - 52.0, visor.center.y + RETICLE_TICKS + 52.0),
            TEXT_BODY,
            flash,
        );
    }
    if let Some(hand) = jarvis.targeting.arming_hand {
        let blink = blink(visor.seconds, 6.0);
        let count = jarvis.targeting.locks.len();
        let text = format!("LOCKING  {count}/{alive}   {hand:?}");
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            &text,
            vec2(visor.center.x - 70.0, visor.center.y + RETICLE_TICKS + 30.0),
            TEXT_BODY,
            faded(RED, blink),
        );
    }
}

fn nearest_to_center(
    visor: &Visor<'_>,
    frame: &XrFrame,
    panel: &XrUiPanel,
    drones: &[DroneMark],
) -> Option<Entity> {
    let panel_pose = panel_pose(panel, frame)?;
    let mut nearest: Option<(f32, Entity)> = None;
    for drone in drones.iter().filter(|drone| drone.alive) {
        let Some(pixel) = project_through_head(drone.stage, frame, panel, &panel_pose, visor.size)
        else {
            continue;
        };
        let from_center = (pixel - visor.center).norm();
        if from_center < LOCK_RADIUS && nearest.is_none_or(|(best, _)| from_center < best) {
            nearest = Some((from_center, drone.entity));
        }
    }
    nearest.map(|(_, entity)| entity)
}

fn targets(
    visor: &mut Visor<'_>,
    frame: &XrFrame,
    panel: &XrUiPanel,
    drones: &[DroneMark],
    locks: &[Entity],
    locked_center: Option<Entity>,
) {
    if !frame.tracking_valid {
        return;
    }
    let Some(panel_pose) = panel_pose(panel, frame) else {
        return;
    };
    for drone in drones.iter().filter(|drone| drone.alive) {
        let Some(pixel) = project_through_head(drone.stage, frame, panel, &panel_pose, visor.size)
        else {
            continue;
        };
        let distance = (drone.stage - frame.head_position).norm();
        let is_center = locked_center == Some(drone.entity);
        let is_lock = locks.contains(&drone.entity);
        let color = if drone.hit > 0.0 {
            faded(WHITE, 0.5 + drone.hit * 0.5)
        } else if is_lock {
            RED
        } else if is_center {
            AMBER
        } else {
            faded(CYAN, visor.boot * 0.9)
        };
        let arm = if is_center || is_lock {
            BRACKET_ARM + 4.0 + (visor.seconds * 6.0).sin() * 2.0
        } else {
            BRACKET_ARM
        };
        let half = arm + 6.0;
        for direction in [
            vec2(1.0, 1.0),
            vec2(-1.0, 1.0),
            vec2(1.0, -1.0),
            vec2(-1.0, -1.0),
        ] {
            let corner = pixel + vec2(half * direction.x, half * direction.y);
            ui_canvas_line(
                visor.storage,
                visor.canvas,
                corner,
                corner - vec2(arm * 0.6 * direction.x, 0.0),
                LINE,
                color,
            );
            ui_canvas_line(
                visor.storage,
                visor.canvas,
                corner,
                corner - vec2(0.0, arm * 0.6 * direction.y),
                LINE,
                color,
            );
        }
        let tag = format!("DRN-{:02}  {distance:.1} M", drone.label);
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            &tag,
            pixel + vec2(half + 6.0, -8.0),
            TEXT_SMALL,
            color,
        );
        let health = "|".repeat(drone.health.max(0) as usize);
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            &health,
            pixel + vec2(half + 6.0, 6.0),
            TEXT_SMALL,
            color,
        );
        if is_lock {
            let spin = visor.seconds * 4.0;
            let diamond: Vec<Vec2> = (0..4)
                .map(|index| {
                    let angle = spin + index as f32 * (TAU / 4.0);
                    pixel + vec2(angle.cos(), angle.sin()) * (half + 12.0)
                })
                .collect();
            ui_canvas_polyline(visor.storage, visor.canvas, &diamond, LINE, RED, true);
            let index = locks
                .iter()
                .position(|lock| *lock == drone.entity)
                .unwrap_or(0)
                + 1;
            ui_canvas_text(
                visor.storage,
                visor.canvas,
                &format!("LOCK {index}"),
                pixel + vec2(-half - 52.0, -8.0),
                TEXT_SMALL,
                RED,
            );
        } else if is_center {
            ui_canvas_text(
                visor.storage,
                visor.canvas,
                "TRACK",
                pixel + vec2(-half - 44.0, -8.0),
                TEXT_SMALL,
                AMBER,
            );
            ui_canvas_arc(
                visor.storage,
                visor.canvas,
                pixel,
                half + 10.0,
                (visor.seconds * 3.0, visor.seconds * 3.0 + PI),
                THIN,
                AMBER_DIM,
            );
        }
    }
}

fn warnings(visor: &mut Visor<'_>, frame: &XrFrame, drones: &[DroneMark], jarvis: &Jarvis) {
    let nearest = drones
        .iter()
        .filter(|drone| drone.alive)
        .map(|drone| (drone.stage - frame.head_position).norm())
        .fold(f32::INFINITY, f32::min);
    let origin = vec2(visor.center.x - 44.0, MARGIN + 52.0);
    if nearest < PROXIMITY_METERS {
        let blink = blink(visor.seconds, 8.0);
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            &format!("PROXIMITY {nearest:.1} M"),
            origin,
            TEXT_BODY,
            faded(RED, blink),
        );
        let edge = faded(RED_DIM, blink * 0.6);
        ui_canvas_rect_stroke(
            visor.storage,
            visor.canvas,
            vec2(MARGIN * 0.5, MARGIN * 0.5),
            visor.size - vec2(MARGIN, MARGIN),
            edge,
            HEAVY,
            6.0,
        );
    } else if jarvis.velocity.norm() > MAX_SPEED * 0.85 {
        let blink = blink(visor.seconds, 5.0);
        ui_canvas_text(
            visor.storage,
            visor.canvas,
            "MAX VELOCITY",
            origin,
            TEXT_BODY,
            faded(AMBER, blink),
        );
    }
}

fn panel_pose(panel: &XrUiPanel, frame: &XrFrame) -> Option<XrPose> {
    match panel.anchor {
        XrUiAnchor::Head => {
            let forward = nalgebra_glm::quat_rotate_vec3(&frame.head_orientation, &-Vec3::z());
            Some(XrPose {
                position: frame.head_position + forward * panel.distance,
                orientation: frame.head_orientation,
            })
        }
        XrUiAnchor::Stage(pose) => Some(pose),
    }
}

fn project_through_head(
    point: Vec3,
    frame: &XrFrame,
    panel: &XrUiPanel,
    panel_pose: &XrPose,
    size: Vec2,
) -> Option<Vec2> {
    let normal = nalgebra_glm::quat_rotate_vec3(&panel_pose.orientation, &Vec3::z());
    let direction = point - frame.head_position;
    let denominator = direction.dot(&normal);
    if denominator.abs() < 1.0e-4 {
        return None;
    }
    let along = (panel_pose.position - frame.head_position).dot(&normal) / denominator;
    if along <= 0.0 {
        return None;
    }
    let hit = frame.head_position + direction * along;
    let local = nalgebra_glm::quat_rotate_vec3(
        &nalgebra_glm::quat_inverse(&panel_pose.orientation),
        &(hit - panel_pose.position),
    );
    let width = panel.width;
    let height = width * size.y / size.x;
    let u = (local.x / width + 0.5) * size.x;
    let v = (0.5 - local.y / height) * size.y;
    let inside = u >= 0.0 && u <= size.x && v >= 0.0 && v <= size.y;
    inside.then_some(vec2(u, v))
}
