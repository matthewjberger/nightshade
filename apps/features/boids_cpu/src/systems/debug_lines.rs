use crate::components::{Position, Shade, Velocity};
use crate::data::Watched;
use crate::resources::{BoidsResources, GridBuilder, Integration, SEARCH_RADIUS};
use nalgebra_glm::{Vec3, Vec4};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;

const BOUNDS_COLOR: Vec4 = Vec4::new(0.40, 0.40, 0.50, 0.65);
const RADIUS_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.30, 0.80);
const NEIGHBOR_COLOR: Vec4 = Vec4::new(1.0, 0.35, 0.35, 0.90);
const BEACON_COLOR: Vec4 = Vec4::new(0.35, 1.0, 0.85, 0.55);

const HEADING_SECONDS: f32 = 0.35;

fn heading_start(position: Vec3, heading: Vec3) -> Vec3 {
    position + heading * (crate::systems::instances::BOID_SCALE.y * 0.5)
}

pub fn draw(
    resources: Res<BoidsResources>,
    integration: Option<Res<Integration>>,
    grid: Option<Res<GridBuilder>>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(lines_entity) = resources.lines else {
        return;
    };

    let mut lines: Vec<Line> = Vec::new();

    if resources.show_headings {
        lines.extend(heading_lines(storage));
    }

    if resources.show_bounds {
        let bounds = integration
            .map(|integration| integration.bounds)
            .unwrap_or_default();
        lines.extend(bounds_lines(bounds));
    }

    if resources.show_neighbors {
        lines.extend(watched_lines(grid, storage));
    }

    storage.set(lines_entity, Lines::new(lines));
}

fn heading_lines(storage: &nightshade::ecs::Storage) -> Vec<Line> {
    let mut lines = Vec::new();

    for (_entity, (position, velocity, shade)) in
        storage.query_ref::<(&Position, &Velocity, &Shade)>().iter()
    {
        let speed = nalgebra_glm::length(&velocity.0);
        if speed < f32::EPSILON {
            continue;
        }
        let heading = velocity.0 / speed;

        lines.push(Line {
            start: heading_start(position.0, heading),
            end: heading_start(position.0, heading) + velocity.0 * HEADING_SECONDS,
            color: Vec4::new(shade.0[0], shade.0[1], shade.0[2], 0.85),
        });
    }

    lines
}

pub fn cuboid_lines(low: Vec3, high: Vec3, color: Vec4) -> Vec<Line> {
    let corners = [
        Vec3::new(low.x, low.y, low.z),
        Vec3::new(high.x, low.y, low.z),
        Vec3::new(high.x, low.y, high.z),
        Vec3::new(low.x, low.y, high.z),
        Vec3::new(low.x, high.y, low.z),
        Vec3::new(high.x, high.y, low.z),
        Vec3::new(high.x, high.y, high.z),
        Vec3::new(low.x, high.y, high.z),
    ];
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    edges
        .iter()
        .map(|&(start, end)| Line {
            start: corners[start],
            end: corners[end],
            color,
        })
        .collect()
}

pub fn bounds_lines(bounds: f32) -> Vec<Line> {
    cuboid_lines(
        Vec3::new(-bounds, -bounds, -bounds),
        Vec3::new(bounds, bounds, bounds),
        BOUNDS_COLOR,
    )
}

fn watched_lines(grid: Option<Res<GridBuilder>>, storage: &nightshade::ecs::Storage) -> Vec<Line> {
    let Some((_entity, (position,))) = storage
        .query_ref::<(&Position,)>()
        .with_tag_type::<Watched>()
        .iter()
        .next()
    else {
        return Vec::new();
    };

    let center = position.0;
    let mut lines = beacon_lines(center);
    lines.extend(radius_lines(center, SEARCH_RADIUS));

    let Some(builder) = grid else {
        return lines;
    };

    let radius_squared = SEARCH_RADIUS * SEARCH_RADIUS;
    builder.grid.for_each_near(center, |slot| {
        let neighbor = builder.grid.sorted_positions[slot];
        let distance_squared = nalgebra_glm::length2(&(neighbor - center));
        if distance_squared > f32::EPSILON && distance_squared < radius_squared {
            lines.push(Line {
                start: center,
                end: neighbor,
                color: NEIGHBOR_COLOR,
            });
        }
        true
    });

    lines
}

fn beacon_lines(center: Vec3) -> Vec<Line> {
    const REACH: f32 = 100_000.0;

    (0..3)
        .map(|axis| {
            let mut along = Vec3::zeros();
            along[axis] = REACH;
            Line {
                start: center - along,
                end: center + along,
                color: BEACON_COLOR,
            }
        })
        .collect()
}

pub fn radius_lines(center: Vec3, radius: f32) -> Vec<Line> {
    const SEGMENTS: usize = 32;
    let mut lines = Vec::with_capacity(SEGMENTS * 3);

    for segment in 0..SEGMENTS {
        let first = segment as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let second = (segment + 1) as f32 / SEGMENTS as f32 * std::f32::consts::TAU;

        for axis in 0..3 {
            lines.push(Line {
                start: center + ring_point(axis, first, radius),
                end: center + ring_point(axis, second, radius),
                color: RADIUS_COLOR,
            });
        }
    }

    lines
}

fn ring_point(axis: usize, angle: f32, radius: f32) -> Vec3 {
    let (first, second) = (angle.cos() * radius, angle.sin() * radius);
    match axis {
        0 => Vec3::new(first, second, 0.0),
        1 => Vec3::new(first, 0.0, second),
        _ => Vec3::new(0.0, first, second),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bounding_box_has_twelve_edges() {
        assert_eq!(bounds_lines(10.0).len(), 12);
    }

    #[test]
    fn every_bounding_edge_runs_along_one_axis() {
        for line in bounds_lines(10.0) {
            let offset = line.end - line.start;
            let moving = (0..3)
                .filter(|&axis| offset[axis].abs() > f32::EPSILON)
                .count();
            assert_eq!(moving, 1);
        }
    }

    #[test]
    fn a_heading_line_starts_clear_of_the_cone() {
        let heading = Vec3::new(0.0, 0.0, 1.0);
        let start = heading_start(Vec3::zeros(), heading);

        assert!(start.z >= crate::systems::instances::BOID_SCALE.y * 0.5 - 1.0e-4);
    }

    #[test]
    fn a_heading_line_starts_along_the_heading_it_is_given() {
        let heading = nalgebra_glm::normalize(&Vec3::new(1.0, 1.0, 0.0));
        let start = heading_start(Vec3::new(5.0, 0.0, 0.0), heading);

        let offset = start - Vec3::new(5.0, 0.0, 0.0);
        assert!(nalgebra_glm::length(&nalgebra_glm::cross(&offset, &heading)) < 1.0e-4);
    }

    #[test]
    fn the_beacon_crosses_the_watched_boid_on_every_axis() {
        let center = Vec3::new(4.0, -2.0, 7.0);

        let lines = beacon_lines(center);

        assert_eq!(lines.len(), 3);
        for (axis, line) in lines.iter().enumerate() {
            let middle = (line.start + line.end) * 0.5;
            assert!(nalgebra_glm::length(&(middle - center)) < 1.0e-4);
            assert!((line.end - line.start)[axis].abs() > 1_000.0);
        }
    }

    #[test]
    fn every_radius_point_sits_on_the_sphere() {
        let center = Vec3::new(1.0, 2.0, 3.0);

        for line in radius_lines(center, 5.0) {
            assert!((nalgebra_glm::length(&(line.start - center)) - 5.0).abs() < 1.0e-4);
        }
    }
}
