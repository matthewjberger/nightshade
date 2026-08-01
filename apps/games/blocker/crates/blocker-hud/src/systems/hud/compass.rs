use crate::resources::CompassPanel;
use crate::resources::HudBuild;
use crate::theme;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::queries::{angle_delta, read_the_hero};
use blocker_figure::resources::Hero;
use blocker_quests::resources::Waypoint;
use blocker_world::data::world::places::{BARROWS, CAMPS, CAPITALS, POIS};
use blocker_world::resources::Bench;
use nalgebra_glm::{Vec2, Vec3, vec2, vec4};
use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::components::TextOverflow;
use nightshade_ui::components::layout_types::WindowLayout;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

struct Sighting {
    bearing: f32,
    kind: usize,
    name: String,
    away: f32,
}

pub fn build_compass(tree: &mut UiTreeBuilder<'_, '_>, hud: &mut HudBuild<'_>) {
    let band = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, theme::COMPASS_TOP)),
            Ab(vec2(theme::COMPASS_WIDTH, theme::COMPASS_HEIGHT)),
            Anchor::TopCenter,
        )
        .with_rect(6.0, 1.0, theme::COMPASS_BORDER)
        .color_raw::<UiBase>(theme::COMPASS_COLOR)
        .entity();
    hud.compass.panel = band;
    tree.in_parent(band, |tree| {
        let mut points = Vec::new();
        for _ in 0..theme::COMPASS_POINTS.len() {
            points.push(
                tree.add_node()
                    .window(
                        Rl(vec2(50.0, 50.0)),
                        Ab(vec2(theme::COMPASS_TICK_WIDTH, 18.0)),
                        Anchor::Center,
                    )
                    .with_text("", theme::SMALL_SIZE)
                    .text_center()
                    .color_raw::<UiBase>(theme::COMPASS_TEXT)
                    .with_visible(false)
                    .entity(),
            );
        }
        hud.compass.points = points;
        let mut marks = Vec::new();
        for _ in 0..theme::COMPASS_MARKS {
            marks.push(
                tree.add_node()
                    .window(
                        Rl(vec2(50.0, 50.0)),
                        Ab(vec2(theme::COMPASS_DOT, theme::COMPASS_DOT)),
                        Anchor::Center,
                    )
                    .with_text("", theme::BODY_SIZE)
                    .text_center()
                    .color_raw::<UiBase>(theme::COMPASS_PLACE)
                    .with_visible(false)
                    .entity(),
            );
        }
        hud.compass.marks = marks;
        hud.compass.notch = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 3.0)),
                Ab(vec2(2.0, 16.0)),
                Anchor::TopCenter,
            )
            .with_rect(1.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(theme::COMPASS_NOTCH)
            .entity();
    });
    hud.compass.readout_panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, theme::COMPASS_TOP + theme::COMPASS_HEIGHT + 4.0)),
            Ab(vec2(
                theme::COMPASS_READOUT_WIDTH,
                theme::COMPASS_READOUT_HEIGHT,
            )),
            Anchor::TopCenter,
        )
        .with_rect(6.0, 1.0, theme::COMPASS_BORDER)
        .color_raw::<UiBase>(theme::COMPASS_COLOR)
        .with_visible(false)
        .entity();
    hud.compass.readout = tree.in_parent(hud.compass.readout_panel, |tree| {
        tree.add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(theme::COMPASS_READOUT_WIDTH - 16.0, 18.0)),
                Anchor::Center,
            )
            .with_text("", theme::SMALL_SIZE)
            .with_text_overflow(TextOverflow::Ellipsis)
            .text_center()
            .color_raw::<UiBase>(theme::COMPASS_TEXT)
            .entity()
    });
}

fn bearing_to(from: Vec3, to: Vec3) -> f32 {
    (to.x - from.x).atan2(to.z - from.z)
}

fn slide(delta: f32) -> Option<f32> {
    let half = theme::COMPASS_FIELD.to_radians() * 0.5;
    (delta.abs() < half).then(|| delta / half * (theme::COMPASS_WIDTH * 0.5))
}

fn gather_the_sightings(bench: &Bench, waypoint: &Waypoint, eye: Vec3) -> Vec<Sighting> {
    let mut wanted: Vec<Sighting> = Vec::new();
    let here = Vec2::new(eye.x, eye.z);
    if let Some(at) = waypoint.at {
        wanted.push(Sighting {
            bearing: bearing_to(eye, at),
            kind: theme::COMPASS_QUEST_MARK,
            name: waypoint.name.clone(),
            away: (Vec2::new(at.x, at.z) - here).magnitude(),
        });
    }
    let mut places: Vec<(usize, [i32; 2], &str)> = Vec::new();
    for (index, capital) in CAPITALS.iter().enumerate() {
        let seat = bench.seats[index];
        places.push((theme::COMPASS_CITY_MARK, seat.city, capital.name));
        places.push((theme::COMPASS_BARROW_MARK, seat.barrow, BARROWS[index].name));
        places.push((theme::COMPASS_CAMP_MARK, seat.camp, CAMPS[index].name));
    }
    for poi in POIS.iter() {
        places.push((theme::COMPASS_PLACE_MARK, poi.at, poi.name));
    }
    for (kind, at, name) in places {
        let spot = Vec2::new(at[0] as f32, at[1] as f32);
        let away = (spot - here).magnitude();
        if away > theme::COMPASS_RANGE {
            continue;
        }
        wanted.push(Sighting {
            bearing: bearing_to(eye, Vec3::new(spot.x, eye.y, spot.y)),
            kind,
            name: name.to_string(),
            away,
        });
    }
    wanted.sort_by(|left, right| left.away.total_cmp(&right.away));
    wanted
}

pub fn show_the_compass(
    bench: Res<Bench>,
    waypoint: Res<Waypoint>,
    hero: Res<Hero>,
    mut compass_panel: ResMut<CompassPanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut sighted: Query<(&Eye, &Gaze)>,
) {
    if compass_panel.points.is_empty() {
        return;
    }
    let Some((eye, gaze)) = read_the_hero(&hero, &mut sighted, |(eye, gaze)| (eye.0, gaze.0))
    else {
        return;
    };
    let facing = gaze.x.atan2(gaze.z);
    for (index, (label, turn)) in theme::COMPASS_POINTS.iter().enumerate() {
        let entity = compass_panel.points[index];
        let offset = slide(angle_delta(facing, turn.to_radians()));
        place_mark(
            &mut dirty,
            &mut commands,
            entity,
            offset,
            theme::COMPASS_TICK_ROW,
            theme::COMPASS_TICK_WIDTH,
            18.0,
        );
        if offset.is_some() && compass_panel.shown.get(index) != Some(&true) {
            ui_set_text(&mut commands, entity, label);
        }
        set_shown(&mut compass_panel.shown, index, offset.is_some());
    }
    let mut wanted = gather_the_sightings(&bench, &waypoint, eye);
    wanted.retain(|sighting| slide(angle_delta(facing, sighting.bearing)).is_some());
    wanted.sort_by(|left, right| {
        angle_delta(facing, left.bearing)
            .abs()
            .total_cmp(&angle_delta(facing, right.bearing).abs())
    });
    let mut ahead: Option<(f32, usize)> = None;
    for index in 0..compass_panel.marks.len() {
        let entity = compass_panel.marks[index];
        let turn = wanted
            .get(index)
            .map(|sighting| angle_delta(facing, sighting.bearing));
        let offset = turn.and_then(slide);
        place_mark(
            &mut dirty,
            &mut commands,
            entity,
            offset,
            theme::COMPASS_MARK_ROW,
            theme::COMPASS_DOT,
            theme::COMPASS_DOT,
        );
        let (Some(sighting), Some(turn)) = (wanted.get(index), turn) else {
            continue;
        };
        if offset.is_none() {
            continue;
        }
        if ahead.is_none_or(|(best, _)| turn.abs() < best) {
            ahead = Some((turn.abs(), index));
        }
        let (glyph, color) = theme::COMPASS_GLYPHS[sighting.kind];
        if compass_panel.shown_marks.get(index) != Some(&sighting.kind) {
            ui_set_text(&mut commands, entity, glyph);
            ui_set_color::<UiBase>(&mut commands, entity, color);
        }
        mark_the_kind(&mut compass_panel.shown_marks, index, sighting.kind);
    }
    let readout = match ahead.and_then(|(_, index)| wanted.get(index)) {
        Some(sighting) => format!("{}  {} paces", sighting.name, sighting.away.round() as i32),
        None => String::new(),
    };
    if compass_panel.shown_readout != readout {
        if compass_panel.shown_readout.is_empty() != readout.is_empty() {
            ui_set_visible(
                &mut dirty,
                &mut commands,
                compass_panel.readout_panel,
                !readout.is_empty(),
            );
        }
        compass_panel.shown_readout = readout.clone();
        ui_set_text(&mut commands, compass_panel.readout, &readout);
    }
}

fn set_shown(shown: &mut Vec<bool>, index: usize, visible: bool) {
    if shown.len() <= index {
        shown.resize(index + 1, false);
    }
    shown[index] = visible;
}

fn mark_the_kind(shown: &mut Vec<usize>, index: usize, kind: usize) {
    if shown.len() <= index {
        shown.resize(index + 1, usize::MAX);
    }
    shown[index] = kind;
}

fn place_mark(
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    entity: Entity,
    offset: Option<f32>,
    row: f32,
    width: f32,
    height: f32,
) {
    ui_set_visible(dirty, commands, entity, offset.is_some());
    let Some(offset) = offset else {
        return;
    };
    ui_set_window_layout(
        dirty,
        commands,
        entity,
        WindowLayout {
            position: Rl(vec2(50.0, 50.0)) + Ab(vec2(offset, row)),
            size: Ab(vec2(width, height)).into(),
            anchor: Anchor::Center,
        },
    );
}
