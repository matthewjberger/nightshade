use crate::data::{CORNER, FINISH, HEADINGS, SIDES, STRAIGHT, TILE, WALL_INSET};
use crate::resources::{Bounds, Lap, TrackLimits};
use nightshade::prelude::{Vec2, Vec3};
use nightshade_platform::prelude::{Input, MouseState, TouchPhase, Window};

pub fn track_tiles() -> Vec<(usize, Vec3, f32)> {
    let mut tiles = Vec::new();
    let mut at = Vec3::zeros();
    let mut heading = 0;
    for (side, count) in SIDES.iter().enumerate() {
        for index in 0..*count {
            let kind = if side == 0 && index == 0 {
                FINISH
            } else {
                STRAIGHT
            };
            tiles.push((kind, at, yaw_of(HEADINGS[heading])));
            at += HEADINGS[heading] * TILE;
        }
        heading = (heading + 1) % HEADINGS.len();
        tiles.push((CORNER, at, yaw_of(HEADINGS[heading])));
        at += HEADINGS[heading] * TILE;
    }
    tiles
}

pub fn yaw_of(direction: Vec3) -> f32 {
    direction.x.atan2(direction.z)
}

pub fn track_limits(tiles: &[(usize, Vec3, f32)]) -> TrackLimits {
    let mut minimum = Vec2::new(f32::MAX, f32::MAX);
    let mut maximum = Vec2::new(f32::MIN, f32::MIN);
    for (_, at, _) in tiles {
        minimum = Vec2::new(minimum.x.min(at.x), minimum.y.min(at.z));
        maximum = Vec2::new(maximum.x.max(at.x), maximum.y.max(at.z));
    }
    let half = Vec2::new(TILE * 0.5, TILE * 0.5);
    let width = Vec2::new(TILE, TILE);
    let outer = Bounds {
        minimum: minimum - half,
        maximum: maximum + half,
    };
    TrackLimits {
        outer,
        inner: Bounds {
            minimum: outer.minimum + width,
            maximum: outer.maximum - width,
        },
    }
}

pub fn wall_push(position: Vec3, limits: &TrackLimits) -> Option<Vec3> {
    let inset = Vec2::new(WALL_INSET, WALL_INSET);
    let outer = Bounds {
        minimum: limits.outer.minimum + inset,
        maximum: limits.outer.maximum - inset,
    };
    let inner = Bounds {
        minimum: limits.inner.minimum - inset,
        maximum: limits.inner.maximum + inset,
    };
    let point = Vec2::new(position.x, position.z);
    let mut held = Vec2::new(
        point.x.clamp(outer.minimum.x, outer.maximum.x),
        point.y.clamp(outer.minimum.y, outer.maximum.y),
    );

    if inside(&inner, held) {
        let ways = [
            (held.x - inner.minimum.x, Vec2::new(inner.minimum.x, held.y)),
            (inner.maximum.x - held.x, Vec2::new(inner.maximum.x, held.y)),
            (held.y - inner.minimum.y, Vec2::new(held.x, inner.minimum.y)),
            (inner.maximum.y - held.y, Vec2::new(held.x, inner.maximum.y)),
        ];
        if let Some((_, nearest)) = ways
            .iter()
            .min_by(|first, second| first.0.total_cmp(&second.0))
        {
            held = *nearest;
        }
    }

    let push = held - point;
    (push.norm_squared() > 0.0).then(|| Vec3::new(push.x, 0.0, push.y))
}

fn inside(bounds: &Bounds, point: Vec2) -> bool {
    point.x > bounds.minimum.x
        && point.x < bounds.maximum.x
        && point.y > bounds.minimum.y
        && point.y < bounds.maximum.y
}

pub fn viewport_size(window: &Window) -> Vec2 {
    let (width, height) = window.cached_viewport_size.unwrap_or((1, 1));
    Vec2::new((width as f32).max(1.0), (height as f32).max(1.0))
}

pub fn pad_pressed(area: [f32; 4], point: Vec2, viewport: Vec2) -> bool {
    let minimum = Vec2::new(area[0] * viewport.x, area[1] * viewport.y);
    let bounds = Bounds {
        minimum,
        maximum: minimum + Vec2::new(area[2] * viewport.x, area[3] * viewport.y),
    };
    inside(&bounds, point)
}

pub fn pointers_held(input: &Input) -> impl Iterator<Item = Vec2> + '_ {
    let mouse = input
        .mouse
        .state
        .contains(MouseState::LEFT_CLICKED)
        .then_some(input.mouse.position);
    input
        .touch
        .touches
        .values()
        .filter(|touch| touch.phase != TouchPhase::Ended && touch.phase != TouchPhase::Cancelled)
        .map(|touch| touch.position)
        .chain(mouse)
}

pub fn touch_starts(input: &Input) -> impl Iterator<Item = Vec2> + '_ {
    input
        .touch
        .touches
        .values()
        .filter(|touch| touch.phase == TouchPhase::Started)
        .map(|touch| touch.start_position)
}

pub fn pointer_starts(input: &Input) -> impl Iterator<Item = Vec2> + '_ {
    let mouse = input
        .mouse
        .state
        .contains(MouseState::LEFT_JUST_PRESSED)
        .then_some(input.mouse.position);
    touch_starts(input).chain(mouse)
}

pub fn lap_readout(lap: &Lap, speed: f32) -> String {
    let best = match lap.best {
        Some(seconds) => clock(seconds),
        None => "--".to_string(),
    };
    format!(
        "LAP {}   {}   BEST {}   {} KM/H",
        lap.count + 1,
        clock(lap.current),
        best,
        (speed * 3.6).round() as u32
    )
}

pub fn clock(seconds: f32) -> String {
    format!("{}:{:04.1}", (seconds / 60.0) as u32, seconds % 60.0)
}
