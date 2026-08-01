use crate::data::{MARKER_LARGEST, MARKER_SMALLEST};
use crate::resources::AcousticsDemo;
use nightshade::prelude::*;
use nightshade_acoustics::contract::{AcousticsEcho, EchoPoint, EmitterVoice};
use nightshade_platform::prelude::*;
use nightshade_render::components::lines::{Line, Lines};

const TONE_COLOR: Vec4 = Vec4::new(0.98, 0.72, 0.22, 1.0);
const STEP_COLOR: Vec4 = Vec4::new(0.35, 0.90, 0.45, 1.0);
const BLAST_COLOR: Vec4 = Vec4::new(0.98, 0.32, 0.30, 1.0);

pub fn show_the_echoes(
    input: Res<Input>,
    mut demo: ResMut<AcousticsDemo>,
    echo: Res<AcousticsEcho>,
    mut markers: Query<(&mut Lines,)>,
) {
    if !demo.built {
        return;
    }

    let toggled = input.keyboard.just_pressed(KeyCode::KeyV);
    if toggled {
        demo.showing = !demo.showing;
    }
    if !toggled && demo.drawn == echo.revision {
        return;
    }
    demo.drawn = echo.revision;

    let showing = demo.showing;
    let points = &echo.points;
    markers.for_entity(demo.markers, |(lines,)| {
        lines.lines.clear();
        if showing {
            for point in points {
                push_marker(&mut lines.lines, point);
            }
        }
        lines.mark_dirty();
    });
}

fn push_marker(lines: &mut Vec<Line>, point: &EchoPoint) {
    let strength = point.strength.clamp(0.0, 1.0);
    let reach = MARKER_SMALLEST + (MARKER_LARGEST - MARKER_SMALLEST) * strength;
    let color = match point.voice {
        EmitterVoice::Tone => TONE_COLOR,
        EmitterVoice::Step => STEP_COLOR,
        EmitterVoice::Blast => BLAST_COLOR,
    };
    match point.voice {
        EmitterVoice::Tone => cross(lines, point.place, reach, color),
        EmitterVoice::Step => ring(lines, point.place, reach, color),
        EmitterVoice::Blast => upright(lines, point.place, reach, color),
    }
}

fn cross(lines: &mut Vec<Line>, place: Vec3, reach: f32, color: Vec4) {
    for axis in [Vec3::x(), Vec3::y(), Vec3::z()] {
        lines.push(Line {
            start: place - axis * reach,
            end: place + axis * reach,
            color,
        });
    }
}

fn ring(lines: &mut Vec<Line>, place: Vec3, reach: f32, color: Vec4) {
    let corners = [
        place + Vec3::new(reach, 0.0, 0.0),
        place + Vec3::new(0.0, 0.0, reach),
        place + Vec3::new(-reach, 0.0, 0.0),
        place + Vec3::new(0.0, 0.0, -reach),
    ];
    for index in 0..corners.len() {
        lines.push(Line {
            start: corners[index],
            end: corners[(index + 1) % corners.len()],
            color,
        });
    }
}

fn upright(lines: &mut Vec<Line>, place: Vec3, reach: f32, color: Vec4) {
    let corners = [
        place + Vec3::new(-reach, -reach, 0.0),
        place + Vec3::new(reach, -reach, 0.0),
        place + Vec3::new(reach, reach, 0.0),
        place + Vec3::new(-reach, reach, 0.0),
    ];
    for index in 0..corners.len() {
        lines.push(Line {
            start: corners[index],
            end: corners[(index + 1) % corners.len()],
            color,
        });
    }
}
