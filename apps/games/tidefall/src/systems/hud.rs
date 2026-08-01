use crate::commands::record_landing;
use crate::components::{Autopilot, Footing, Fuel, Grounded, Locked, Velocity};
use crate::contract::TidefallEvent;
use crate::data::{
    BODIES, BodyView, DRIFT_FULL, DRIFT_RING, Drift, GLOBE_MERIDIANS, GLOBE_PARALLELS,
    GLOBE_RADIUS, GLOBE_STEPS, GLOBE_TILT, Globe, HOME, Marker, PATH_STEP, PATH_STEPS, RELICS,
};
use crate::queries::{Bodies, altitude, collect_bodies, gravity_at, nearest_body};
use crate::resources::{HudHandles, Mode, TidefallResources};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_ui::components::UiLayoutNode;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

const PANEL_BACKGROUND: Vec4 = Vec4::new(0.03, 0.04, 0.07, 0.72);
const PANEL_BORDER: Vec4 = Vec4::new(0.55, 0.80, 1.0, 0.22);
const TEXT: Vec4 = Vec4::new(0.92, 0.95, 1.0, 1.0);
const TEXT_DIM: Vec4 = Vec4::new(0.70, 0.76, 0.86, 1.0);
const ACCENT: Vec4 = Vec4::new(0.55, 0.85, 1.0, 0.95);
const MARGIN: f32 = 16.0;
const FLIGHT_WIDTH: f32 = 380.0;
const FLIGHT_HEIGHT: f32 = 124.0;
const TALK_WIDTH: f32 = 820.0;
const TALK_HEIGHT: f32 = 74.0;
const TARGET_WIDTH: f32 = 380.0;
const TARGET_HEIGHT: f32 = 74.0;
const TALLY_WIDTH: f32 = 420.0;
const TALLY_HEIGHT: f32 = 42.0;
const LINE: f32 = 24.0;
const TEXT_SIZE: f32 = 18.0;
const RETICLE: f32 = 7.0;
const RETICLE_GAP: f32 = 4.0;
const RETICLE_TICK: f32 = 8.0;
const MARKER_MIN: f32 = 16.0;
const MARKER_MAX: f32 = 400.0;
const EDGE_MARGIN: f32 = 36.0;
const MARKER_TEXT: f32 = 15.0;
const STROKE: f32 = 1.6;
const GLOBE_DIM: Vec4 = Vec4::new(0.55, 0.85, 1.0, 0.22);
const SHIP_MARK: Vec4 = Vec4::new(1.0, 0.62, 0.25, 1.0);
const PATH_DOT: Vec4 = Vec4::new(0.45, 1.0, 0.55, 0.9);
const GRAVITY_TEXT: f32 = 15.0;
const GRAVITY_VALUE: f32 = 22.0;
const GRAVITY_DROP: f32 = 54.0;
const DRIFT_DROP: f32 = 190.0;
const DRIFT_DOTS: usize = 28;
const DRIFT_DOT: f32 = 1.4;
const CHEVRON: f32 = 9.0;

pub fn build(tree: &mut UiTreeBuilder) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .without_pointer_events()
        .with_visible(false)
        .entity();

    let mut canvas = Entity::default();
    let mut mode_line = Entity::default();
    let mut place_line = Entity::default();
    let mut motion_line = Entity::default();
    let mut target_panel = Entity::default();
    let mut target_line = Entity::default();
    let mut pilot_line = Entity::default();
    let mut tally_line = Entity::default();
    let mut near_line = Entity::default();
    let mut talk_panel = Entity::default();
    let mut talk_name = Entity::default();
    let mut talk_text = Entity::default();

    tree.in_parent(root, |tree| {
        canvas = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .without_pointer_events()
            .entity();

        let flight = panel(
            tree,
            Rl(vec2(0.0, 100.0)) + Ab(vec2(MARGIN, -MARGIN)),
            vec2(FLIGHT_WIDTH, FLIGHT_HEIGHT),
            Anchor::BottomLeft,
            true,
        );
        tree.in_parent(flight, |tree| {
            mode_line = line(tree, TEXT);
            place_line = line(tree, TEXT_DIM);
            motion_line = line(tree, TEXT_DIM);
            near_line = line(tree, ACCENT);
        });

        talk_panel = panel(
            tree,
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -MARGIN - FLIGHT_HEIGHT - 8.0)),
            vec2(TALK_WIDTH, TALK_HEIGHT),
            Anchor::BottomCenter,
            false,
        );
        tree.in_parent(talk_panel, |tree| {
            talk_name = line(tree, ACCENT);
            talk_text = line(tree, TEXT);
        });

        target_panel = panel(
            tree,
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN, MARGIN)),
            vec2(TARGET_WIDTH, TARGET_HEIGHT),
            Anchor::TopRight,
            false,
        );
        tree.in_parent(target_panel, |tree| {
            target_line = line(tree, ACCENT);
            pilot_line = line(tree, TEXT_DIM);
        });

        let tally = panel(
            tree,
            Ab(vec2(MARGIN, MARGIN)),
            vec2(TALLY_WIDTH, TALLY_HEIGHT),
            Anchor::TopLeft,
            true,
        );
        tree.in_parent(tally, |tree| {
            tally_line = line(tree, TEXT_DIM);
        });
    });

    HudHandles {
        root,
        canvas,
        mode_line,
        place_line,
        motion_line,
        target_panel,
        target_line,
        pilot_line,
        tally_line,
        near_line,
        talk_panel,
        talk_name,
        talk_text,
    }
}

fn panel(
    tree: &mut UiTreeBuilder,
    position: impl Into<UiValue<Vec2>>,
    size: Vec2,
    anchor: Anchor,
    visible: bool,
) -> Entity {
    tree.add_node()
        .window(position, Ab(size), anchor)
        .with_rect(8.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BACKGROUND)
        .flow(FlowDirection::Vertical, 10.0, 2.0)
        .without_pointer_events()
        .with_visible(visible)
        .entity()
}

fn line(tree: &mut UiTreeBuilder, color: Vec4) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, LINE)))
        .with_text("", TEXT_SIZE)
        .text_left()
        .color_raw::<UiBase>(color)
        .without_pointer_events()
        .entity()
}

pub fn measure(
    mut resources: ResMut<TidefallResources>,
    events: EventReader<TidefallEvent>,
    bodies: Bodies<'_, '_>,
    mut ships: Query<(&Velocity, &Footing, &Locked, &Autopilot, &LocalTransform)>,
    mut cameras: Query<(&Camera, &LocalTransform)>,
    mut walkers: Query<(&Fuel, &Grounded, &Velocity, &LocalTransform)>,
    mut nodes: Query<(&UiLayoutNode,)>,
) {
    for event in events.iter() {
        if let TidefallEvent::Landed { .. } = event {
            record_landing(&mut resources);
        }
    }
    let (Some(ship), Some(walker)) = (resources.ship, resources.walker) else {
        return;
    };
    let views = collect_bodies(bodies);
    let worlds = resources.visited.len();
    let shards = resources.found.len();
    let aboard = resources.mode == Mode::Aboard;
    let canvas = resources.hud.canvas;
    let landings = resources.landings;

    let mut extent = Vec2::new(1.0, 1.0);
    nodes.for_entity(canvas, |(node,)| {
        extent = node.computed_rect.max - node.computed_rect.min;
    });
    let mut lens = None;
    cameras.for_entity(walker, |(camera, transform)| {
        let aspect = (extent.x / extent.y.max(1.0)).max(0.1);
        let projection = camera.projection.matrix_with_aspect(aspect);
        let view = transform
            .as_matrix()
            .try_inverse()
            .unwrap_or_else(Mat4::identity);
        lens = Some((projection * view, transform.translation));
    });

    let readout = &mut resources.readout;
    readout.tally_line = format!(
        "worlds {worlds} / {}      landings {landings}      shards {shards} / {}",
        views.len(),
        RELICS.len()
    );
    readout.reticle = aboard;
    readout.marker = None;
    readout.globe = None;
    readout.drift = None;
    readout.gravity = None;
    readout.target_shown = aboard;
    if aboard {
        readout.target_line = "TARGET none".to_string();
        readout.pilot_line = "autopilot off".to_string();
    }

    let mut place = None;
    ships.for_entity(ship, |(velocity, footing, locked, autopilot, transform)| {
        let Some(index) = nearest_body(transform.translation, &views) else {
            return;
        };
        let near = &views[index];
        if aboard {
            let speed = (velocity.0 - near.motion).magnitude();
            let height = altitude(transform.translation, near).max(0.0);
            readout.mode_line = match footing {
                Footing::Landed => format!("SHIP      landed on {}", near.name),
                Footing::Flying => format!("SHIP      near {}", near.name),
            };
            readout.place_line = format!("altitude {height:.0}");
            readout.motion_line = format!("speed {speed:.1}");
        } else {
            place = Some(index);
        }
        if let Some(target) = locked
            .0
            .and_then(|entity| views.iter().find(|view| view.entity == entity))
        {
            let relative = transform.translation - target.center;
            let distance = relative.magnitude();
            let closing = -(velocity.0 - target.motion).dot(&(relative / distance.max(1.0e-3)));
            readout.target_line = format!(
                "TARGET {}      {:.0} away      closing {closing:.1}",
                target.name,
                (distance - target.radius).max(0.0)
            );
            readout.pilot_line = match autopilot {
                Autopilot::Off => "autopilot off".to_string(),
                Autopilot::Align => "autopilot aligning".to_string(),
                Autopilot::Burn => "autopilot burning".to_string(),
                Autopilot::Brake => "autopilot braking".to_string(),
                Autopilot::Descend => "autopilot landing".to_string(),
            };
            if let Some((lens, eye)) = lens {
                readout.marker = Some(mark(
                    lens,
                    eye,
                    extent,
                    target.center,
                    target.radius,
                    &target.name,
                ));
            }
        }
    });

    if place.is_some() {
        walkers.for_entity(walker, |(fuel, grounded, velocity, transform)| {
            let Some(index) = nearest_body(transform.translation, &views) else {
                return;
            };
            let near = &views[index];
            let height = altitude(transform.translation, near).max(0.0);
            readout.mode_line = format!("ON FOOT      {}", near.name);
            readout.place_line = format!("altitude {height:.1}");
            readout.motion_line = format!("jetpack {:.0}%", fuel.0 * 100.0);
            if !grounded.0 {
                readout.gravity = Some(
                    gravity_at(transform.translation, &views).magnitude() / BODIES[HOME].gravity,
                );
                readout.globe = Some(globe(
                    near,
                    transform.translation,
                    velocity.0 + near.motion,
                    extent,
                ));
                readout.drift = Some(drift_of(transform.rotation, velocity.0));
            }
        });
    }
}

fn globe(focus: &BodyView, ship: Vec3, velocity: Vec3, extent: Vec2) -> Globe {
    let relative = ship - focus.center;
    let distance = relative.magnitude().max(1.0e-3);
    let toward = relative / distance;
    let mut pole = nalgebra_glm::quat_rotate_vec3(&focus.rotation, &Vec3::y());
    if toward.dot(&pole).abs() > 0.95 {
        pole = nalgebra_glm::quat_rotate_vec3(&focus.rotation, &Vec3::x());
    }
    let side = pole.cross(&toward).normalize();
    let north = toward.cross(&side).normalize();
    let viewer = (toward * GLOBE_TILT.cos() + north * GLOBE_TILT.sin()).normalize();
    let up = viewer.cross(&side).normalize();
    let center = Vec2::new(
        MARGIN + GLOBE_RADIUS + 60.0,
        MARGIN + TALLY_HEIGHT + 40.0 + GLOBE_RADIUS * 2.0,
    );
    let project =
        |point: Vec3| center + Vec2::new(point.dot(&side), -point.dot(&up)) * GLOBE_RADIUS;
    let front = |point: Vec3| point.dot(&viewer) > 0.0;

    let mut lines = Vec::new();
    let mut ring = |points: Vec<Vec3>| {
        for pair in points.windows(2) {
            lines.push((
                project(pair[0]),
                project(pair[1]),
                front(pair[0]) && front(pair[1]),
            ));
        }
    };
    for latitude in GLOBE_PARALLELS {
        let points: Vec<Vec3> = (0..=GLOBE_STEPS)
            .map(|step| {
                let longitude = step as f32 / GLOBE_STEPS as f32 * std::f32::consts::TAU;
                let local = Vec3::new(
                    latitude.cos() * longitude.cos(),
                    latitude.sin(),
                    latitude.cos() * longitude.sin(),
                );
                nalgebra_glm::quat_rotate_vec3(&focus.rotation, &local)
            })
            .collect();
        ring(points);
    }
    for meridian in 0..GLOBE_MERIDIANS {
        let longitude = meridian as f32 / GLOBE_MERIDIANS as f32 * std::f32::consts::PI;
        let points: Vec<Vec3> = (0..=GLOBE_STEPS)
            .map(|step| {
                let latitude = step as f32 / GLOBE_STEPS as f32 * std::f32::consts::TAU;
                let local = Vec3::new(
                    latitude.cos() * longitude.cos(),
                    latitude.sin(),
                    latitude.cos() * longitude.sin(),
                );
                nalgebra_glm::quat_rotate_vec3(&focus.rotation, &local)
            })
            .collect();
        ring(points);
    }

    let mut position = relative;
    let mut motion = velocity - focus.motion;
    let mut path = Vec::with_capacity(PATH_STEPS);
    for _ in 0..PATH_STEPS {
        let range = position.magnitude();
        if range <= focus.radius {
            break;
        }
        let pull = -position / range * (focus.pull / range);
        motion += pull * PATH_STEP;
        position += motion * PATH_STEP;
        let point = project(position / focus.radius);
        if point.x < 0.0 || point.y < 0.0 || point.x > extent.x || point.y > extent.y {
            break;
        }
        path.push(point);
    }

    Globe {
        center,
        radius: GLOBE_RADIUS,
        lines,
        ship: project(relative / focus.radius),
        path,
    }
}

fn drift_of(rotation: Quat, drift: Vec3) -> Drift {
    let seen = nalgebra_glm::quat_rotate_vec3(&nalgebra_glm::quat_conjugate(&rotation), &drift);
    let planar = Vec2::new(seen.x, -seen.y);
    let length = planar.magnitude();
    Drift {
        direction: if length > 1.0e-3 {
            planar / length
        } else {
            Vec2::zeros()
        },
        planar: length,
        forward: -seen.z,
    }
}

fn mark(lens: Mat4, eye: Vec3, extent: Vec2, center: Vec3, radius: f32, name: &str) -> Marker {
    let clip = lens * nalgebra_glm::vec4(center.x, center.y, center.z, 1.0);
    let in_front = clip.w > 0.0;
    let mut screen = if in_front {
        Vec2::new(
            (clip.x / clip.w + 1.0) * 0.5 * extent.x,
            (1.0 - clip.y / clip.w) * 0.5 * extent.y,
        )
    } else {
        Vec2::new(
            (1.0 - clip.x / clip.w) * 0.5 * extent.x,
            (1.0 + clip.y / clip.w) * 0.5 * extent.y,
        )
    };
    let middle = extent * 0.5;
    let inside = in_front
        && screen.x >= EDGE_MARGIN
        && screen.y >= EDGE_MARGIN
        && screen.x <= extent.x - EDGE_MARGIN
        && screen.y <= extent.y - EDGE_MARGIN;
    if !inside {
        let away = screen - middle;
        let reach = (middle.x - EDGE_MARGIN).min(middle.y - EDGE_MARGIN);
        let length = away.magnitude().max(1.0e-3);
        screen = middle + away / length * reach;
    }
    let distance = (center - eye).magnitude().max(radius + 1.0);
    let apparent = radius / distance * middle.y * 1.2;
    Marker {
        screen,
        radius: apparent.clamp(MARKER_MIN, MARKER_MAX),
        name: name.to_string(),
        in_view: inside,
    }
}

pub fn paint(
    mut resources: ResMut<TidefallResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let resources = &mut *resources;
    let handles = resources.hud;
    if handles.root == Entity::default() {
        return;
    }
    let wanted = &resources.readout;
    let shown = &mut resources.shown;

    for (entity, text, last) in [
        (handles.mode_line, &wanted.mode_line, &mut shown.mode_line),
        (
            handles.place_line,
            &wanted.place_line,
            &mut shown.place_line,
        ),
        (
            handles.motion_line,
            &wanted.motion_line,
            &mut shown.motion_line,
        ),
        (
            handles.target_line,
            &wanted.target_line,
            &mut shown.target_line,
        ),
        (
            handles.pilot_line,
            &wanted.pilot_line,
            &mut shown.pilot_line,
        ),
        (
            handles.tally_line,
            &wanted.tally_line,
            &mut shown.tally_line,
        ),
        (handles.near_line, &wanted.near_line, &mut shown.near_line),
        (handles.talk_name, &wanted.talk_name, &mut shown.talk_name),
        (handles.talk_text, &wanted.talk_text, &mut shown.talk_text),
    ] {
        if *last != *text {
            *last = text.clone();
            ui_set_text(&mut commands, entity, text);
        }
    }
    if shown.talk_shown != Some(wanted.talk_shown) {
        shown.talk_shown = Some(wanted.talk_shown);
        ui_set_visible(
            &mut dirty,
            &mut commands,
            handles.talk_panel,
            wanted.talk_shown,
        );
    }
    if shown.target_shown != Some(wanted.target_shown) {
        shown.target_shown = Some(wanted.target_shown);
        ui_set_visible(
            &mut dirty,
            &mut commands,
            handles.target_panel,
            wanted.target_shown,
        );
    }

    let canvas = handles.canvas;
    ui_canvas_clear(storage, canvas);
    let extent = storage
        .get::<UiLayoutNode>(canvas)
        .map(|node| node.computed_rect.max - node.computed_rect.min)
        .unwrap_or_else(|| Vec2::new(1.0, 1.0));
    if wanted.reticle {
        let middle = extent * 0.5;
        ui_canvas_arc(
            storage,
            canvas,
            middle,
            RETICLE,
            (0.0, std::f32::consts::TAU),
            STROKE,
            ACCENT,
        );
        for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            let direction = Vec2::new(dx, dy);
            ui_canvas_line(
                storage,
                canvas,
                middle + direction * (RETICLE + RETICLE_GAP),
                middle + direction * (RETICLE + RETICLE_GAP + RETICLE_TICK),
                STROKE,
                ACCENT,
            );
        }
    }
    if let Some(globe) = &wanted.globe {
        for (start, end, front) in &globe.lines {
            let color = if *front { ACCENT } else { GLOBE_DIM };
            ui_canvas_line(storage, canvas, *start, *end, STROKE * 0.8, color);
        }
        for point in &globe.path {
            ui_canvas_circle(storage, canvas, *point, DRIFT_DOT, PATH_DOT);
        }
        ui_canvas_circle(storage, canvas, globe.ship, 4.0, SHIP_MARK);
    }
    if let Some(gravity) = wanted.gravity {
        let middle = extent * 0.5;
        let label = "GRAVITY";
        let value = format!("{gravity:.1}x");
        ui_canvas_text(
            storage,
            canvas,
            label,
            middle + Vec2::new(-(label.len() as f32) * GRAVITY_TEXT * 0.3, GRAVITY_DROP),
            GRAVITY_TEXT,
            ACCENT,
        );
        ui_canvas_text(
            storage,
            canvas,
            &value,
            middle
                + Vec2::new(
                    -(value.len() as f32) * GRAVITY_VALUE * 0.3,
                    GRAVITY_DROP + GRAVITY_TEXT + 4.0,
                ),
            GRAVITY_VALUE,
            TEXT,
        );
    }
    if let Some(drift) = &wanted.drift {
        let middle = extent * 0.5 + Vec2::new(0.0, DRIFT_DROP);
        for index in 0..DRIFT_DOTS {
            let angle = index as f32 / DRIFT_DOTS as f32 * std::f32::consts::TAU;
            let dot = middle + Vec2::new(angle.cos(), angle.sin()) * DRIFT_RING;
            ui_canvas_circle(storage, canvas, dot, DRIFT_DOT, ACCENT);
        }
        for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            let direction = Vec2::new(dx, dy);
            ui_canvas_line(
                storage,
                canvas,
                middle + direction * 6.0,
                middle + direction * (DRIFT_RING - 8.0),
                STROKE,
                ACCENT,
            );
        }
        let reach = (drift.planar / DRIFT_FULL).min(1.0) * DRIFT_RING;
        if reach > 2.0 {
            let tip = middle + drift.direction * reach;
            let back = Vec2::new(-drift.direction.x, -drift.direction.y);
            let wing = Vec2::new(-drift.direction.y, drift.direction.x);
            for step in [0.0, 1.0] {
                let head = tip + back * step * CHEVRON * 0.9;
                ui_canvas_line(
                    storage,
                    canvas,
                    head,
                    head + (back + wing) * CHEVRON,
                    STROKE * 1.3,
                    SHIP_MARK,
                );
                ui_canvas_line(
                    storage,
                    canvas,
                    head,
                    head + (back - wing) * CHEVRON,
                    STROKE * 1.3,
                    SHIP_MARK,
                );
            }
        } else {
            let color = if drift.forward >= 0.0 {
                SHIP_MARK
            } else {
                TEXT_DIM
            };
            ui_canvas_arc(
                storage,
                canvas,
                middle,
                5.0,
                (0.0, std::f32::consts::TAU),
                STROKE * 1.3,
                color,
            );
        }
    }
    if let Some(marker) = &wanted.marker {
        if marker.in_view {
            ui_canvas_arc(
                storage,
                canvas,
                marker.screen,
                marker.radius,
                (0.0, std::f32::consts::TAU),
                STROKE,
                ACCENT,
            );
            ui_canvas_text(
                storage,
                canvas,
                &marker.name,
                marker.screen + Vec2::new(marker.radius + 8.0, -MARKER_TEXT * 0.5),
                MARKER_TEXT,
                ACCENT,
            );
        } else {
            ui_canvas_circle(storage, canvas, marker.screen, MARKER_MIN * 0.4, ACCENT);
            let middle = extent * 0.5;
            let toward = (middle - marker.screen).normalize();
            ui_canvas_text(
                storage,
                canvas,
                &marker.name,
                marker.screen + toward * 22.0 - Vec2::new(0.0, MARKER_TEXT * 0.5),
                MARKER_TEXT,
                ACCENT,
            );
        }
    }
    dirty.render_dirty = true;
}
