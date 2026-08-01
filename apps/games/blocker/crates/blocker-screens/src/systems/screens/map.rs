use crate::resources::Screens;
use crate::resources::{MapArt, MapPanel};
use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::queries::decode_atlas;
use blocker_blocks::resources::Art;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_quests::resources::Waypoint;
use blocker_screen::data::screen::Screen;
use blocker_world::data::world::places::{BARROWS, CAMPS, CAPITALS, POIS};
use blocker_world::queries::{biome_for, surface_height};
use blocker_world::resources::{Bench, Generation};
use nalgebra_glm::{Vec2, Vec3, vec2};
use nightshade::prelude::*;
use nightshade_platform::prelude::Input;
use nightshade_scene::prelude::LocalTransform;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::layout_types::WindowLayout;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{
    CardStyle, add_card, add_cell_text, add_footer_buttons, add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};
use std::collections::{HashMap, HashSet};

struct Mark {
    offset_x: f32,
    offset_z: f32,
    kind: usize,
    away: f32,
    name: String,
}

struct Field<'life> {
    anchor: Vec2,
    span: f32,
    scale: f32,
    nearby: &'life mut Vec<Mark>,
    seen: &'life mut HashSet<(i32, i32)>,
    named: &'life mut HashSet<String>,
}

pub fn build_map(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut map_panel: ResMut<MapPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    let blank = {
        let (images, runtime) = tree.images_and_runtime();
        ui_upload_image(images, runtime, &theme::MAP_BLANK, 1, 1)
    };
    map_panel.layer = blank.as_ref().map(|upload| upload.layer);
    map_panel.zoom = theme::MAP_START_ZOOM;
    map_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::MAP_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    tree.in_parent(map_panel.panel, |tree| {
        add_title_bar(tree, &style, blocker_hud::theme::MENU_WIDTH, "The map");
        let field = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0))
                    + Ab(vec2(
                        0.0,
                        blocker_hud::theme::TITLE_HEIGHT + blocker_hud::theme::BODY_PADDING,
                    )),
                Ab(vec2(theme::MAP_FIELD, theme::MAP_FIELD)),
                Anchor::TopCenter,
            )
            .with_rect(8.0, 2.0, blocker_hud::theme::PANEL_BORDER)
            .color_raw::<UiBase>(theme::MAP_FIELD_COLOR)
            .entity();
        tree.in_parent(field, |tree| {
            let mut picture = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 50.0)),
                    Ab(vec2(theme::MAP_FIELD - 4.0, theme::MAP_FIELD - 4.0)),
                    Anchor::Center,
                )
                .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT);
            if let Some(upload) = &blank {
                picture = picture.with_image_uv(upload.layer, upload.uv_min, upload.uv_max);
            }
            map_panel.picture = picture.entity();
            for (letter, across, down) in [
                ("N", 50.0, 3.0),
                ("E", 97.0, 50.0),
                ("S", 50.0, 97.0),
                ("W", 3.0, 50.0),
            ] {
                tree.add_node()
                    .window(Rl(vec2(across, down)), Ab(vec2(22.0, 18.0)), Anchor::Center)
                    .with_text(letter, blocker_hud::theme::SMALL_SIZE)
                    .text_center()
                    .with_text_outline(theme::MAP_INK, theme::MAP_INK_WIDTH)
                    .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT);
            }
            let bar =
                theme::MAP_BAR_BLOCKS[theme::MAP_START_ZOOM] * map_scale(theme::MAP_START_ZOOM);
            map_panel.bar = tree
                .add_node()
                .window(
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(12.0, -14.0)),
                    Ab(vec2(bar, theme::MAP_BAR_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_rect(1.0, 1.0, theme::MAP_INK)
                .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT)
                .entity();
            map_panel.bar_label = tree
                .add_node()
                .window(
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(12.0, -34.0)),
                    Ab(vec2(120.0, 18.0)),
                    Anchor::TopLeft,
                )
                .with_text("", blocker_hud::theme::SMALL_SIZE)
                .text_left()
                .with_text_outline(theme::MAP_INK, theme::MAP_INK_WIDTH)
                .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT)
                .entity();
            let mut markers = Vec::new();
            let mut labels = Vec::new();
            for index in 0..theme::MAP_MARKERS {
                markers.push(
                    tree.add_node()
                        .window(
                            Rl(vec2(50.0, 50.0)),
                            Ab(vec2(theme::MAP_MARK_SIZE, theme::MAP_MARK_SIZE)),
                            Anchor::Center,
                        )
                        .with_text("", theme::MAP_GLYPH_SIZE)
                        .text_center()
                        .with_text_outline(theme::MAP_INK, theme::MAP_INK_WIDTH)
                        .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT)
                        .with_visible(false)
                        .entity(),
                );
                if index < theme::MAP_LABELS {
                    labels.push(
                        tree.add_node()
                            .window(
                                Rl(vec2(50.0, 50.0)),
                                Ab(vec2(theme::MAP_LABEL_WIDTH, 16.0)),
                                Anchor::Center,
                            )
                            .with_text("", blocker_hud::theme::SMALL_SIZE)
                            .text_center()
                            .with_text_outline(theme::MAP_INK, theme::MAP_INK_WIDTH)
                            .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT)
                            .with_visible(false)
                            .entity(),
                    );
                }
            }
            map_panel.markers = markers;
            map_panel.labels = labels;
            map_panel.nose = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 50.0)),
                    Ab(vec2(theme::MAP_NOSE, theme::MAP_NOSE)),
                    Anchor::Center,
                )
                .with_rect(theme::MAP_NOSE * 0.5, 0.0, theme::CLEAR_COLOR)
                .color_raw::<UiBase>(theme::MAP_HERE_COLOR)
                .entity();
            map_panel.here = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 50.0)),
                    Ab(vec2(theme::MAP_DOT + 4.0, theme::MAP_DOT + 4.0)),
                    Anchor::Center,
                )
                .with_rect((theme::MAP_DOT + 4.0) * 0.5, 2.0, theme::MAP_HERE_EDGE)
                .color_raw::<UiBase>(theme::MAP_HERE_COLOR)
                .entity();
        });
        let legend_width = (blocker_hud::theme::MENU_WIDTH
            - blocker_hud::theme::BODY_PADDING * 2.0)
            / theme::MAP_LEGEND.len() as f32;
        for (index, word) in theme::MAP_LEGEND.iter().enumerate() {
            let (glyph, color) = theme::MAP_GLYPHS[index];
            tree.add_node()
                .window(
                    Ab(vec2(
                        blocker_hud::theme::BODY_PADDING + legend_width * index as f32,
                        blocker_hud::theme::TITLE_HEIGHT
                            + blocker_hud::theme::BODY_PADDING
                            + theme::MAP_FIELD
                            + 4.0,
                    )),
                    Ab(vec2(legend_width, theme::MAP_LEGEND_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_text(&format!("{glyph}  {word}"), blocker_hud::theme::SMALL_SIZE)
                .text_center()
                .color_raw::<UiBase>(color);
        }
        tree.add_node()
            .size_px(
                blocker_hud::theme::MENU_WIDTH,
                theme::MAP_FIELD
                    + blocker_hud::theme::BODY_PADDING
                    + 8.0
                    + theme::MAP_LEGEND_HEIGHT,
            )
            .entity();
        map_panel.readout = add_cell_text(
            tree,
            &style,
            "",
            blocker_hud::theme::ROW_WIDTH,
            blocker_hud::theme::SMALL_SIZE,
            blocker_hud::theme::DIM_TEXT_COLOR,
        );
        let buttons = add_footer_buttons(tree, &style, blocker_hud::theme::MENU_WIDTH, &["Close"]);
        map_panel.close = buttons[0];
    });
    menu_show_on(&mut menus, Screen::Map, map_panel.panel);
    built_tree.finish();
}

fn map_scale(zoom: usize) -> f32 {
    (theme::MAP_FIELD * 0.5 - theme::MAP_DOT) / theme::MAP_SPANS[zoom]
}

fn wind(offset_x: f32, offset_z: f32) -> &'static str {
    let turn = offset_x.atan2(-offset_z);
    let eighth = std::f32::consts::TAU / theme::MAP_WINDS.len() as f32;
    let index = ((turn / eighth).round() as i32).rem_euclid(theme::MAP_WINDS.len() as i32);
    theme::MAP_WINDS[index as usize]
}

fn map_cell(stood: Vec3) -> [i32; 2] {
    [
        (stood.x / theme::MAP_REPAINT_STEP).round() as i32,
        (stood.z / theme::MAP_REPAINT_STEP).round() as i32,
    ]
}

fn map_anchor(cell: [i32; 2]) -> Vec2 {
    vec2(
        cell[0] as f32 * theme::MAP_REPAINT_STEP,
        cell[1] as f32 * theme::MAP_REPAINT_STEP,
    )
}

fn cell_color(pixels: &[u8], width: u32, cell: (u32, u32)) -> [f32; 3] {
    let texels = blocker_blocks::data::atlas::TILE_TEXELS;
    let mut sum = [0.0f32; 3];
    for row in 0..texels {
        for column in 0..texels {
            let x = cell.0 * texels + column;
            let y = cell.1 * texels + row;
            let offset = ((y * width + x) * 4) as usize;
            for (channel, total) in sum.iter_mut().enumerate() {
                *total += pixels[offset + channel] as f32 / 255.0;
            }
        }
    }
    let count = (texels * texels) as f32;
    [sum[0] / count, sum[1] / count, sum[2] / count]
}

fn paint_the_ground(bench: &Bench, art: &MapArt, anchor: Vec2, span: f32) -> Vec<u8> {
    let mut colors: HashMap<BlockKind, [f32; 3]> = HashMap::new();
    let mut color_of = |kind: BlockKind| -> [f32; 3] {
        *colors
            .entry(kind)
            .or_insert_with(|| cell_color(&art.pixels, art.width, kind.info().top))
    };
    let water = color_of(BlockKind::Water);
    let sea = blocker_world::data::world::SEA_LEVEL;
    let size = theme::MAP_PIXELS;
    let step = span * 2.0 / size as f32;
    let mut picture = vec![0u8; (size * size * 4) as usize];
    let mut heights = vec![0i32; size as usize + 1];
    for row in 0..size {
        let z = (anchor.y - span + (row as f32 + 0.5) * step).round() as i32;
        for (column, height) in heights.iter_mut().enumerate() {
            let x = (anchor.x - span + (column as f32 + 0.5) * step).round() as i32;
            *height = surface_height(bench, x, z);
        }
        for column in 0..size as usize {
            let x = (anchor.x - span + (column as f32 + 0.5) * step).round() as i32;
            let height = heights[column];
            let east = heights[column + 1];
            let color = if height <= sea {
                let depth = ((sea - height) as f32 / theme::MAP_DEPTH).clamp(0.0, 1.0);
                let dim = 1.0 - depth * theme::MAP_DEPTH_SHADE;
                [water[0] * dim, water[1] * dim, water[2] * dim]
            } else {
                let surface = biome_for(bench, x, z, height).info().surface;
                let base = color_of(surface);
                let slope = ((east - height) as f32 * theme::MAP_SHADE)
                    .clamp(-theme::MAP_SHADE_LIMIT, theme::MAP_SHADE_LIMIT);
                let lift = theme::MAP_TONE * (1.0 + slope);
                [base[0] * lift, base[1] * lift, base[2] * lift]
            };
            let offset = (row as usize * size as usize + column) * 4;
            for (channel, value) in color.iter().enumerate() {
                picture[offset + channel] = (value.clamp(0.0, 1.0) * 255.0) as u8;
            }
            picture[offset + 3] = 255;
        }
    }
    picture
}

pub fn paint_the_map(
    bench: Res<Bench>,
    art: Res<Art>,
    hero: Res<Hero>,
    mut map_art: ResMut<MapArt>,
    mut map_panel: ResMut<MapPanel>,
    mut build: UiBuild,
    mut placed: Query<(&Eye, &LocalTransform)>,
) {
    let Some(layer) = map_panel.layer else {
        return;
    };
    if map_art.palette != Some(art.palette) {
        let (pixels, width, _) = decode_atlas(art.palette);
        map_art.palette = Some(art.palette);
        map_art.pixels = pixels;
        map_art.width = width;
    }
    let Some(stood) = read_the_hero(&hero, &mut placed, |(_, transform)| transform.translation)
    else {
        return;
    };
    let zoom = map_panel.zoom.min(theme::MAP_SPANS.len() - 1);
    let cell = map_cell(stood);
    if map_panel.painted == Some((cell, zoom)) {
        return;
    }
    map_panel.painted = Some((cell, zoom));
    let picture = paint_the_ground(&bench, &map_art, map_anchor(cell), theme::MAP_SPANS[zoom]);
    let mut tree = UiTreeBuilder::new(build.context());
    let (_, runtime) = tree.images_and_runtime();
    if ui_repaint_image(
        runtime,
        layer,
        &picture,
        theme::MAP_PIXELS,
        theme::MAP_PIXELS,
    )
    .is_none()
    {
        map_panel.painted = None;
    }
}

fn map_consider(field: &mut Field<'_>, x: i32, z: i32, kind: usize, name: &str) {
    let mut offset_x = x as f32 - field.anchor.x;
    let mut offset_z = z as f32 - field.anchor.y;
    let reach = offset_x.abs().max(offset_z.abs());
    if reach > field.span {
        if kind != theme::MAP_QUEST_MARK {
            return;
        }
        offset_x *= field.span / reach;
        offset_z *= field.span / reach;
    }
    if !field.seen.insert((x.div_euclid(8), z.div_euclid(8))) {
        return;
    }
    if !field.named.insert(name.to_string()) {
        return;
    }
    field.nearby.push(Mark {
        offset_x: offset_x * field.scale,
        offset_z: offset_z * field.scale,
        kind,
        away: (offset_x * offset_x + offset_z * offset_z).sqrt(),
        name: name.to_string(),
    });
}

pub fn show_the_map(
    bench: Res<Bench>,
    generation: Res<Generation>,
    waypoint: Res<Waypoint>,
    hero: Res<Hero>,
    mut map_panel: ResMut<MapPanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut placed: Query<(&Eye, &Gaze, &LocalTransform)>,
) {
    if map_panel.markers.is_empty() {
        return;
    }
    let Some((stood, gaze)) = read_the_hero(&hero, &mut placed, |(_, gaze, transform)| {
        (transform.translation, gaze.0)
    }) else {
        return;
    };
    let (anchor, zoom) = match map_panel.painted {
        Some((cell, zoom)) => (map_anchor(cell), zoom),
        None => (vec2(stood.x, stood.z), map_panel.zoom),
    };
    let zoom = zoom.min(theme::MAP_SPANS.len() - 1);
    let span = theme::MAP_SPANS[zoom];
    let scale = map_scale(zoom);
    if map_panel.shown_zoom != Some(zoom) {
        map_panel.shown_zoom = Some(zoom);
        let blocks = theme::MAP_BAR_BLOCKS[zoom];
        ui_set_window_layout(
            &mut dirty,
            &mut commands,
            map_panel.bar,
            WindowLayout {
                position: Rl(vec2(0.0, 100.0)) + Ab(vec2(12.0, -14.0)),
                size: Ab(vec2(blocks * scale, theme::MAP_BAR_HEIGHT)).into(),
                anchor: Anchor::TopLeft,
            },
        );
        ui_set_text(
            &mut commands,
            map_panel.bar_label,
            &format!("{} blocks", blocks as i32),
        );
    }
    let mut nearby: Vec<Mark> = Vec::new();
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut named: HashSet<String> = HashSet::new();
    let mut field = Field {
        anchor,
        span,
        scale,
        nearby: &mut nearby,
        seen: &mut seen,
        named: &mut named,
    };
    if let Some(at) = waypoint.at {
        map_consider(
            &mut field,
            at.x.floor() as i32,
            at.z.floor() as i32,
            theme::MAP_QUEST_MARK,
            &waypoint.name,
        );
    }
    for (index, capital) in CAPITALS.iter().enumerate() {
        let seat = bench.seats[index];
        for (kind, at, name) in [
            (theme::MAP_CITY_MARK, seat.city, capital.name),
            (theme::MAP_BARROW_MARK, seat.barrow, BARROWS[index].name),
            (theme::MAP_CAMP_MARK, seat.camp, CAMPS[index].name),
        ] {
            map_consider(&mut field, at[0], at[1], kind, name);
        }
    }
    for poi in POIS.iter() {
        map_consider(
            &mut field,
            poi.at[0],
            poi.at[1],
            theme::MAP_PLACE_MARK,
            poi.name,
        );
    }
    for (sign, name) in generation.signs.iter() {
        map_consider(&mut field, sign[0], sign[2], theme::MAP_VILLAGE_MARK, name);
    }
    nearby.sort_by(|left, right| {
        (left.kind != theme::MAP_QUEST_MARK, left.away)
            .partial_cmp(&(right.kind != theme::MAP_QUEST_MARK, right.away))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (index, marker) in map_panel.markers.iter().enumerate() {
        match nearby.get(index) {
            Some(mark) => {
                ui_set_window_layout(
                    &mut dirty,
                    &mut commands,
                    *marker,
                    WindowLayout {
                        position: Rl(vec2(50.0, 50.0)) + Ab(vec2(mark.offset_x, mark.offset_z)),
                        size: Ab(vec2(theme::MAP_MARK_SIZE, theme::MAP_MARK_SIZE)).into(),
                        anchor: Anchor::Center,
                    },
                );
                let (glyph, color) = theme::MAP_GLYPHS[mark.kind];
                ui_set_text(&mut commands, *marker, glyph);
                ui_set_color::<UiBase>(&mut commands, *marker, color);
                ui_set_visible(&mut dirty, &mut commands, *marker, true);
            }
            None => ui_set_visible(&mut dirty, &mut commands, *marker, false),
        }
    }
    let mut placed_labels: Vec<(f32, f32)> = Vec::new();
    let widest = zoom + 1 == theme::MAP_SPANS.len();
    let mut titled = nearby
        .iter()
        .filter(|mark| !(widest && mark.kind == theme::MAP_VILLAGE_MARK));
    for label in map_panel.labels.iter() {
        let Some(mark) = titled.next() else {
            ui_set_visible(&mut dirty, &mut commands, *label, false);
            continue;
        };
        let Some(lift) = clear_row(&placed_labels, mark.offset_x, mark.offset_z) else {
            ui_set_visible(&mut dirty, &mut commands, *label, false);
            continue;
        };
        placed_labels.push((mark.offset_x, lift));
        ui_set_window_layout(
            &mut dirty,
            &mut commands,
            *label,
            WindowLayout {
                position: Rl(vec2(50.0, 50.0)) + Ab(vec2(mark.offset_x, lift)),
                size: Ab(vec2(theme::MAP_LABEL_WIDTH, 16.0)).into(),
                anchor: Anchor::Center,
            },
        );
        ui_set_text(&mut commands, *label, &mark.name);
        ui_set_visible(&mut dirty, &mut commands, *label, true);
    }
    let here = vec2((stood.x - anchor.x) * scale, (stood.z - anchor.y) * scale);
    ui_set_window_layout(
        &mut dirty,
        &mut commands,
        map_panel.here,
        WindowLayout {
            position: Rl(vec2(50.0, 50.0)) + Ab(here),
            size: Ab(vec2(theme::MAP_DOT + 4.0, theme::MAP_DOT + 4.0)).into(),
            anchor: Anchor::Center,
        },
    );
    let facing = vec2(gaze.x, gaze.z);
    let facing = if facing.magnitude() > 0.001 {
        facing.normalize()
    } else {
        vec2(0.0, -1.0)
    };
    ui_set_window_layout(
        &mut dirty,
        &mut commands,
        map_panel.nose,
        WindowLayout {
            position: Rl(vec2(50.0, 50.0)) + Ab(here + facing * theme::MAP_NOSE_REACH),
            size: Ab(vec2(theme::MAP_NOSE, theme::MAP_NOSE)).into(),
            anchor: Anchor::Center,
        },
    );
    let from_you = |mark: &Mark| {
        let offset_x = mark.offset_x / scale + anchor.x - stood.x;
        let offset_z = mark.offset_z / scale + anchor.y - stood.z;
        format!(
            "{} paces {}",
            (offset_x * offset_x + offset_z * offset_z).sqrt().round() as i32,
            wind(offset_x, offset_z)
        )
    };
    let errand = match nearby
        .iter()
        .find(|mark| mark.kind == theme::MAP_QUEST_MARK)
    {
        Some(mark) => format!("Errand at {}, {}.", mark.name, from_you(mark)),
        None => "No errand.".to_string(),
    };
    let nearest = nearby
        .iter()
        .filter(|mark| mark.kind != theme::MAP_QUEST_MARK)
        .min_by(|left, right| left.away.total_cmp(&right.away))
        .map(|mark| format!("{}, {}.  ", mark.name, from_you(mark)))
        .unwrap_or_default();
    let readout = format!("{nearest}{errand}");
    ui_set_text(&mut commands, map_panel.readout, &readout);
}

fn clear_row(placed: &[(f32, f32)], x: f32, z: f32) -> Option<f32> {
    theme::MAP_LABEL_LIFTS
        .iter()
        .map(|drop| z + drop)
        .find(|lift| {
            !placed.iter().any(|(other_x, other_y)| {
                (other_x - x).abs() < theme::MAP_LABEL_WIDTH * 0.7
                    && (other_y - lift).abs() < theme::MAP_LABEL_STEP
            })
        })
}

pub fn map_buttons(
    input: Res<Input>,
    screens: Res<Screens>,
    mut map_panel: ResMut<MapPanel>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, map_panel.close) {
        next.pending = Some(screens.return_to);
        return;
    }
    let wheel = input.mouse.wheel_delta.y;
    if wheel > 0.01 {
        map_panel.zoom = map_panel.zoom.saturating_sub(1);
    } else if wheel < -0.01 {
        map_panel.zoom = (map_panel.zoom + 1).min(theme::MAP_SPANS.len() - 1);
    }
}
