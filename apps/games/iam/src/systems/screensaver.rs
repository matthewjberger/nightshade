use crate::data::machine::{
    PIPES_FALL_SECONDS, PIPES_HOLD_SECONDS, PIPES_RISE_SECONDS, PIPES_RUN_SECONDS, Screensaver,
};
use crate::data::pipes::{SEGMENTS, Segment};
use crate::resources::{AppResources, Ink};
use nightshade::prelude::*;
use nightshade_platform::prelude::{Input, MouseState, Time};
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

const STRIDE: f32 = 0.40;
const REACH: Vec3 = Vec3::new(2.10, 1.25, 0.72);
const DEPTH: f32 = 1.75;

const WAYS: [Vec3; 6] = [
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, -1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(0.0, 0.0, -1.0),
];

fn inside(at: Vec3) -> bool {
    at.x.abs() <= REACH.x && at.y.abs() <= REACH.y && (at.z - DEPTH).abs() <= REACH.z
}

fn lay_one(resources: &mut AppResources) {
    let boot = &mut resources.boot;
    let head = boot
        .pipes
        .last()
        .map(|segment: &Segment| segment.tail)
        .unwrap_or(vec3(0.0, 0.0, DEPTH));
    let last = boot.pipes_heading;

    let mut chosen = None;
    for _ in 0..12 {
        let way = WAYS[boot.pipes_rng.u32(..6) as usize];
        if way == -last {
            continue;
        }
        let run = 1 + boot.pipes_rng.u32(..3);
        let tail = head + way * (STRIDE * run as f32);
        if inside(tail) {
            chosen = Some((way, tail));
            break;
        }
    }

    let Some((way, tail)) = chosen else {
        boot.pipes.clear();
        boot.pipes_heading = WAYS[boot.pipes_rng.u32(..6) as usize];
        boot.pipes_enamel = boot.pipes_rng.u32(..6);
        return;
    };

    if boot.pipes_rng.u32(..5) == 0 {
        boot.pipes_enamel = boot.pipes_rng.u32(..6);
    }
    boot.pipes.push(Segment {
        head,
        tail,
        enamel: boot.pipes_enamel,
    });
    boot.pipes_heading = way;
    while boot.pipes.len() > SEGMENTS {
        boot.pipes.remove(0);
    }
}

pub fn begin(resources: &mut AppResources, uptime: f64) {
    let seed =
        (resources.session.night as u32).wrapping_mul(2_654_435_761) ^ (uptime * 1000.0) as u32;
    resources.boot.pipes_rng = fastrand::Rng::with_seed(seed as u64);
    resources.boot.pipes.clear();
    resources.boot.pipes_heading = WAYS[0];
    resources.boot.pipes_enamel = resources.boot.pipes_rng.u32(..6);
    resources.boot.pipes_grown = 0.0;
    resources.boot.screensaver = Screensaver::Rising;
    resources.boot.screensaver_elapsed = 0.0;
    lay_one(resources);
}

pub fn fade(resources: &AppResources) -> f32 {
    let elapsed = resources.boot.screensaver_elapsed;
    match resources.boot.screensaver {
        Screensaver::Idle => 0.0,
        Screensaver::Rising => (elapsed / PIPES_RISE_SECONDS).min(1.0),
        Screensaver::Running => 1.0,
        Screensaver::Falling => 1.0 - (elapsed / PIPES_FALL_SECONDS).min(1.0),
    }
}

pub fn covering(resources: &AppResources) -> bool {
    !matches!(resources.boot.screensaver, Screensaver::Idle)
}

fn dismissible(resources: &AppResources) -> bool {
    matches!(resources.boot.screensaver, Screensaver::Running)
        && resources.boot.screensaver_elapsed >= PIPES_HOLD_SECONDS
}

fn touched(input: &Input) -> bool {
    input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED)
        || !input.keyboard.frame_chars.is_empty()
        || !input.keyboard.just_pressed_keys.is_empty()
}

pub fn advance(
    mut resources: ResMut<AppResources>,
    time: Res<Time>,
    input: Res<Input>,
    config: Res<crate::data::config::Config>,
    mut gathered: ResMut<crate::systems::desk::Gathered>,
    mut build: UiBuild,
    nodes: Query<&mut UiLayoutNode>,
    contents: Query<&mut UiNodeContent>,
) {
    let mut ink = Ink {
        build: build.context(),
        nodes,
        contents,
        gathered: &mut gathered,
    };
    let ink = &mut ink;
    let resources = &mut *resources;
    if matches!(resources.boot.screensaver, Screensaver::Idle) {
        return;
    }
    let delta = time.delta_time;
    let seconds = config.pipes_segment_seconds;
    resources.boot.screensaver_elapsed += delta;

    resources.boot.pipes_grown += delta / seconds.max(0.02);
    while resources.boot.pipes_grown >= 1.0 {
        resources.boot.pipes_grown -= 1.0;
        lay_one(resources);
    }

    match resources.boot.screensaver {
        Screensaver::Rising => {
            if resources.boot.screensaver_elapsed >= PIPES_RISE_SECONDS {
                resources.boot.screensaver = Screensaver::Running;
                resources.boot.screensaver_elapsed = 0.0;
            }
        }
        Screensaver::Running => {
            let laid_enough = resources.boot.screensaver_elapsed >= PIPES_RUN_SECONDS;
            if laid_enough || (dismissible(resources) && touched(&input)) {
                resources.boot.screensaver = Screensaver::Falling;
                resources.boot.screensaver_elapsed = 0.0;
            }
        }
        Screensaver::Falling => {
            if resources.boot.screensaver_elapsed >= PIPES_FALL_SECONDS {
                resources.boot.screensaver = Screensaver::Idle;
                resources.boot.screensaver_elapsed = 0.0;
                resources.redraw = crate::resources::redraw_everything();
                if resources.boot.night_waiting {
                    resources.boot.night_waiting = false;
                    crate::systems::conversation::advance_night(resources, ink, &config);
                }
            }
        }
        Screensaver::Idle => {}
    }
}
