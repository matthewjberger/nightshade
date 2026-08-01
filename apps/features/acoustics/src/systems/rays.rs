use crate::data::{RAY_SPEED, RAY_TRAIL};
use crate::resources::AcousticsDemo;
use nightshade::prelude::*;
use nightshade_acoustics::contract::{AcousticsRays, RayKind, RayStep};
use nightshade_platform::prelude::*;
use nightshade_render::components::lines::{Line, Lines};

const CLEAR_TONE: Vec4 = Vec4::new(0.30, 0.95, 0.40, 1.0);
const BLOCKED_TONE: Vec4 = Vec4::new(0.98, 0.55, 0.15, 1.0);
const BOUNCE_TONE: Vec4 = Vec4::new(0.35, 0.55, 1.00, 1.0);
const HOME_TONE: Vec4 = Vec4::new(0.40, 0.90, 1.00, 1.0);
const FOUND_TONE: Vec4 = Vec4::new(0.55, 1.00, 0.35, 1.0);

pub fn flow_the_rays(
    input: Res<Input>,
    time: Res<Time>,
    rays: Res<AcousticsRays>,
    mut demo: ResMut<AcousticsDemo>,
    mut paths: Query<(&mut Lines,)>,
) {
    if !demo.built {
        return;
    }
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        demo.flowing = !demo.flowing;
        demo.front = 0.0;
    }

    let flowing = demo.flowing;
    if flowing {
        demo.front += time.delta_time * RAY_SPEED;
        if demo.front > rays.span + RAY_TRAIL {
            demo.front = 0.0;
        }
    }

    let front = demo.front;
    let steps = &rays.steps;
    paths.for_entity(demo.paths, |(lines,)| {
        lines.lines.clear();
        if flowing {
            for step in steps {
                if let Some(line) = lit_part(step, front) {
                    lines.lines.push(line);
                }
            }
        }
        lines.mark_dirty();
    });
}

fn lit_part(step: &RayStep, front: f32) -> Option<Line> {
    let reach = step.to_travel - step.from_travel;
    if reach <= 1.0e-4 {
        return None;
    }
    let low = (front - RAY_TRAIL).max(step.from_travel);
    let high = front.min(step.to_travel);
    if high <= low {
        return None;
    }
    let first = (low - step.from_travel) / reach;
    let last = (high - step.from_travel) / reach;
    Some(Line {
        start: step.start + (step.end - step.start) * first,
        end: step.start + (step.end - step.start) * last,
        color: tone_of(step.kind),
    })
}

fn tone_of(kind: RayKind) -> Vec4 {
    match kind {
        RayKind::Clear => CLEAR_TONE,
        RayKind::Blocked => BLOCKED_TONE,
        RayKind::Bounce => BOUNCE_TONE,
        RayKind::Home => HOME_TONE,
        RayKind::Found => FOUND_TONE,
    }
}
