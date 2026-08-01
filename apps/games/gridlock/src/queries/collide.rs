use crate::queries::grid::cell_of;
use crate::resources::{City, Hull, Lot};
use nightshade::prelude::{Vec2, Vec3};

pub fn lots_near(city: &City, position: Vec3) -> Vec<Lot> {
    let center = cell_of(position);
    let mut lots = Vec::new();
    for dz in -1..=1 {
        for dx in -1..=1 {
            if let Some(cell) = city.cells.get(&[center[0] + dx, center[1] + dz]) {
                lots.extend(cell.lots.iter().copied());
            }
        }
    }
    lots
}

pub fn props_near(city: &City, position: Vec3) -> Vec<([i32; 2], usize, Lot)> {
    let center = cell_of(position);
    let mut props = Vec::new();
    for dz in -1..=1 {
        for dx in -1..=1 {
            let key = [center[0] + dx, center[1] + dz];
            if let Some(cell) = city.cells.get(&key) {
                props.extend(
                    cell.props
                        .iter()
                        .enumerate()
                        .filter_map(|(index, lot)| lot.map(|lot| (key, index, lot))),
                );
            }
        }
    }
    props
}

pub fn hull_at(local: &Hull, position: Vec3, yaw: f32) -> Hull {
    let (sin, cos) = yaw.sin_cos();
    let heading = Vec2::new(sin, cos);
    let across = Vec2::new(cos, -sin);
    Hull {
        center: Vec2::new(position.x, position.z)
            + across * local.center.x
            + heading * local.center.y,
        heading,
        half: local.half,
    }
}

fn lot_hull(lot: &Lot) -> Hull {
    Hull {
        center: (lot.minimum + lot.maximum) * 0.5,
        heading: Vec2::new(0.0, 1.0),
        half: (lot.maximum - lot.minimum) * 0.5,
    }
}

fn hull_axes(hull: &Hull) -> [Vec2; 2] {
    [Vec2::new(hull.heading.y, -hull.heading.x), hull.heading]
}

fn hull_reach(hull: &Hull, axis: &Vec2) -> f32 {
    let [across, along] = hull_axes(hull);
    across.dot(axis).abs() * hull.half.x + along.dot(axis).abs() * hull.half.y
}

fn hull_overlap(first: &Hull, second: &Hull) -> Option<Vec2> {
    let offset = second.center - first.center;
    let mut best: Option<(f32, Vec2)> = None;
    for axis in hull_axes(first).into_iter().chain(hull_axes(second)) {
        let overlap =
            hull_reach(first, &axis) + hull_reach(second, &axis) - offset.dot(&axis).abs();
        if overlap <= 0.0 {
            return None;
        }
        if best.is_none_or(|(least, _)| overlap < least) {
            let sign = if offset.dot(&axis) > 0.0 { -1.0 } else { 1.0 };
            best = Some((overlap, axis * (overlap * sign)));
        }
    }
    best.map(|(_, push)| push)
}

pub fn hull_touches(hull: &Hull, lot: &Lot) -> bool {
    hull_overlap(hull, &lot_hull(lot)).is_some()
}

fn hull_local(hull: &Hull, point: Vec3) -> Vec2 {
    let offset = Vec2::new(point.x, point.z) - hull.center;
    let [across, along] = hull_axes(hull);
    Vec2::new(offset.dot(&across), offset.dot(&along))
}

pub fn hull_holds_point(hull: &Hull, point: Vec3, radius: f32) -> bool {
    let local = hull_local(hull, point);
    let nearest = Vec2::new(
        local.x.clamp(-hull.half.x, hull.half.x),
        local.y.clamp(-hull.half.y, hull.half.y),
    );
    (local - nearest).norm() < radius
}

pub fn hull_push_point(hull: &Hull, point: Vec3, radius: f32) -> Vec3 {
    let local = hull_local(hull, point);
    let nearest = Vec2::new(
        local.x.clamp(-hull.half.x, hull.half.x),
        local.y.clamp(-hull.half.y, hull.half.y),
    );
    let away = local - nearest;
    let distance = away.norm();
    if distance >= radius {
        return Vec3::zeros();
    }
    let out = if distance < 1e-4 {
        let gap_x = hull.half.x - local.x.abs();
        let gap_y = hull.half.y - local.y.abs();
        if gap_x < gap_y {
            Vec2::new((gap_x + radius) * local.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, (gap_y + radius) * local.y.signum())
        }
    } else {
        away / distance * (radius - distance)
    };
    let [across, along] = hull_axes(hull);
    let world = across * out.x + along * out.y;
    Vec3::new(world.x, 0.0, world.y)
}

pub fn lots_overlap(first: &Lot, second: &Lot, gap: f32) -> bool {
    first.minimum.x < second.maximum.x + gap
        && first.maximum.x + gap > second.minimum.x
        && first.minimum.y < second.maximum.y + gap
        && first.maximum.y + gap > second.minimum.y
}

pub fn lot_push(position: Vec3, radius: f32, lots: &[Lot]) -> Vec3 {
    let point = Vec2::new(position.x, position.z);
    let mut push = Vec2::zeros();
    for lot in lots {
        let nearest = Vec2::new(
            point.x.clamp(lot.minimum.x, lot.maximum.x),
            point.y.clamp(lot.minimum.y, lot.maximum.y),
        );
        let away = point + push - nearest;
        let distance = away.norm();
        if distance >= radius {
            continue;
        }
        if distance < 1e-4 {
            let center = (lot.minimum + lot.maximum) * 0.5;
            let half = (lot.maximum - lot.minimum) * 0.5;
            let offset = point - center;
            let gap_x = half.x - offset.x.abs();
            let gap_z = half.y - offset.y.abs();
            if gap_x < gap_z {
                push.x += (gap_x + radius) * offset.x.signum();
            } else {
                push.y += (gap_z + radius) * offset.y.signum();
            }
        } else {
            push += away / distance * (radius - distance);
        }
    }
    Vec3::new(push.x, 0.0, push.y)
}

pub fn circle_push(position: Vec3, radius: f32, others: &[(Vec3, f32)]) -> Vec3 {
    let mut push = Vec3::zeros();
    for (other, other_radius) in others {
        let away = Vec3::new(position.x - other.x, 0.0, position.z - other.z);
        let distance = away.norm();
        let reach = radius + other_radius;
        if distance >= reach || distance < 1e-4 {
            continue;
        }
        push += away / distance * (reach - distance);
    }
    push
}
