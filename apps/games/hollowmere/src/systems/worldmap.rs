use crate::commands::frame;
use crate::components::{Cooldowns, Health, Worth};
use crate::data::{
    Ask, Biome, GIVERS, MAP_MARKS, Objective, PICKUPS, PLACES, QUESTS, SPAWNS, TOWN,
};
use crate::queries::{
    biome_weights, dominant_biome, harbor_pier, is_alive, land_height, place_spot, road_near,
    spot_of, turn_in_giver, zone, zone_center,
};
use crate::resources::{Generation, HudHandles, HudText, Icons, Journal, Places};
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_kit::prelude::MAP_ICONS;
use nightshade_scene::prelude::{LocalTransform, Name};
use nightshade_ui::components::UiNodeContent;
use nightshade_ui::components::layout_types::WindowLayout;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiRuntime, UiImageLayerAllocator};

const SCRIM: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.6);
const PANEL: Vec4 = Vec4::new(0.16, 0.12, 0.08, 0.98);
const BORDER: Vec4 = Vec4::new(0.85, 0.70, 0.40, 0.9);
const TITLE: Vec4 = Vec4::new(1.0, 0.88, 0.55, 1.0);
const LABEL: Vec4 = Vec4::new(1.0, 0.97, 0.90, 1.0);
const PLAYER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const PLAYER_RING: Vec4 = Vec4::new(0.1, 0.5, 1.0, 1.0);
const MARK: Vec4 = Vec4::new(1.0, 0.85, 0.30, 1.0);
const NOTE: Vec4 = Vec4::new(0.92, 0.90, 0.84, 1.0);
const FRAME: f32 = 24.0;
const TITLE_HEIGHT: f32 = 44.0;
const NOTE_HEIGHT: f32 = 58.0;
const ICON: f32 = 40.0;
const PLAYER_DOT: f32 = 14.0;
const MARK_DOT: f32 = 16.0;
const LABEL_WIDTH: f32 = 180.0;
const LABEL_HEIGHT: f32 = 22.0;

fn to_map(town: Vec2, spot: Vec2) -> Vec2 {
    let scale = theme::MAP_PANEL / (theme::MAP_REACH * 2.0);
    vec2(
        (spot.x - town.x + theme::MAP_REACH) * scale,
        (spot.y - town.y + theme::MAP_REACH) * scale,
    )
}

fn mark_region(icons: &Icons, name: &str) -> Option<(u32, Vec2, Vec2)> {
    let layer = icons.mark_layer?;
    let slot = MAP_ICONS.iter().position(|(icon, _)| *icon == name)?;
    let (uv_min, uv_max) = icons.marks.get(slot).copied()?;
    Some((layer, uv_min, uv_max))
}

pub fn build(tree: &mut UiTreeBuilder, icons: &Icons, handles: &mut HudHandles) {
    handles.map_root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(SCRIM)
        .without_pointer_events()
        .with_visible(false)
        .entity();
    let width = theme::MAP_PANEL + FRAME * 2.0;
    let height = theme::MAP_PANEL + FRAME * 2.0 + TITLE_HEIGHT + NOTE_HEIGHT;
    tree.in_parent(handles.map_root, |tree| {
        let panel = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(width, height)),
                Anchor::Center,
            )
            .with_rect(10.0, 2.0, BORDER)
            .color_raw::<UiBase>(PANEL)
            .without_pointer_events()
            .entity();
        tree.in_parent(panel, |tree| {
            frame(
                tree,
                icons.frame_layer,
                icons.tint,
                theme::FRAME_CORNER * 1.4,
            );
            tree.add_node()
                .window(
                    Ab(vec2(0.0, 10.0)),
                    Ab(vec2(width, TITLE_HEIGHT - 6.0)),
                    Anchor::TopLeft,
                )
                .with_text("The Hollowmere Reach", 28.0)
                .color_raw::<UiBase>(TITLE)
                .without_pointer_events()
                .entity();
            handles.map_picture = tree
                .add_node()
                .window(
                    Ab(vec2(FRAME, TITLE_HEIGHT + FRAME / 2.0)),
                    Ab(vec2(theme::MAP_PANEL, theme::MAP_PANEL)),
                    Anchor::TopLeft,
                )
                .with_image_uv(icons.map_layer.unwrap_or(0), vec2(0.0, 0.0), vec2(1.0, 1.0))
                .color_raw::<UiBase>(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .without_pointer_events()
                .entity();
            let picture = handles.map_picture;
            tree.in_parent(picture, |tree| {
                for (icon, _) in MAP_MARKS {
                    let (layer, uv_min, uv_max) =
                        mark_region(icons, icon).unwrap_or((0, vec2(0.0, 0.0), vec2(1.0, 1.0)));
                    let node = tree
                        .add_node()
                        .window(Ab(vec2(0.0, 0.0)), Ab(vec2(ICON, ICON)), Anchor::TopLeft)
                        .with_image_uv(layer, uv_min, uv_max)
                        .color_raw::<UiBase>(Vec4::new(1.0, 1.0, 1.0, 1.0))
                        .without_pointer_events()
                        .with_visible(false)
                        .entity();
                    handles.map_icons.push(node);
                }
                for place in &PLACES {
                    let node = tree
                        .add_node()
                        .window(
                            Ab(vec2(0.0, 0.0)),
                            Ab(vec2(LABEL_WIDTH, LABEL_HEIGHT)),
                            Anchor::TopLeft,
                        )
                        .with_text(place.name, 15.0)
                        .with_text_outline(Vec4::new(0.0, 0.0, 0.0, 0.7), 1.0)
                        .color_raw::<UiBase>(LABEL)
                        .without_pointer_events()
                        .with_visible(false)
                        .entity();
                    handles.map_labels.push(node);
                }
                for _ in 0..theme::MAP_MARK_COUNT {
                    let mark = tree
                        .add_node()
                        .window(
                            Ab(vec2(0.0, 0.0)),
                            Ab(vec2(MARK_DOT, MARK_DOT)),
                            Anchor::TopLeft,
                        )
                        .with_rect(MARK_DOT / 2.0, 2.0, Vec4::new(0.0, 0.0, 0.0, 0.55))
                        .color_raw::<UiBase>(MARK)
                        .without_pointer_events()
                        .with_visible(false)
                        .entity();
                    handles.map_marks.push(mark);
                }
                handles.map_player = tree
                    .add_node()
                    .window(
                        Ab(vec2(0.0, 0.0)),
                        Ab(vec2(PLAYER_DOT, PLAYER_DOT)),
                        Anchor::TopLeft,
                    )
                    .with_rect(PLAYER_DOT / 2.0, 3.0, PLAYER_RING)
                    .color_raw::<UiBase>(PLAYER)
                    .without_pointer_events()
                    .entity();
            });
            handles.map_note = tree
                .add_node()
                .window(
                    Ab(vec2(FRAME, TITLE_HEIGHT + FRAME + theme::MAP_PANEL + 4.0)),
                    Ab(vec2(theme::MAP_PANEL, NOTE_HEIGHT - 8.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 16.0)
                .text_left()
                .color_raw::<UiBase>(NOTE)
                .without_pointer_events()
                .entity();
        });
    });
}

fn shade(color: [f32; 3], light: f32) -> [u8; 4] {
    [
        (color[0] * light * 255.0).clamp(0.0, 255.0) as u8,
        (color[1] * light * 255.0).clamp(0.0, 255.0) as u8,
        (color[2] * light * 255.0).clamp(0.0, 255.0) as u8,
        255,
    ]
}

fn biome_color(biome: Biome) -> [f32; 3] {
    match biome {
        Biome::Plains => [0.38, 0.60, 0.27],
        Biome::Forest => [0.22, 0.46, 0.20],
        Biome::Desert => [0.82, 0.74, 0.46],
        Biome::Hills => [0.47, 0.56, 0.30],
        Biome::Ocean => [0.76, 0.71, 0.52],
        Biome::Snowfield => [0.90, 0.92, 0.95],
        Biome::Peaks => [0.56, 0.56, 0.57],
        Biome::Swamp => [0.31, 0.42, 0.26],
        Biome::Badlands => [0.72, 0.43, 0.28],
    }
}

fn map_pixels(generation: &Generation, town: Vec2) -> Vec<u8> {
    let side = theme::MAP_IMAGE as usize;
    let mut pixels = vec![0u8; side * side * 4];
    let mut heights = vec![0i32; side * side];
    let step = theme::MAP_REACH * 2.0 / (side - 1) as f32;
    for row in 0..side {
        for column in 0..side {
            let x = (town.x - theme::MAP_REACH + column as f32 * step).floor() as i32;
            let z = (town.y - theme::MAP_REACH + row as f32 * step).floor() as i32;
            heights[row * side + column] = land_height(generation, x, z);
        }
    }
    let wall = zone(generation, TOWN);
    let fort = zone(generation, crate::data::FORT);
    let pier = harbor_pier(generation);
    let stairs = generation.crypt;
    for row in 0..side {
        for column in 0..side {
            let x = (town.x - theme::MAP_REACH + column as f32 * step).floor() as i32;
            let z = (town.y - theme::MAP_REACH + row as f32 * step).floor() as i32;
            let point = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
            let height = heights[row * side + column];
            let west = heights[row * side + column.saturating_sub(1)];
            let north = heights[row.saturating_sub(1) * side + column];
            let slope = ((height - west) + (height - north)) as f32 / 8.0;
            let light = (0.85 + slope).clamp(0.55, 1.25);
            let border = column < 3 || row < 3 || column >= side - 3 || row >= side - 3;
            let color = if border {
                shade([0.78, 0.66, 0.44], 1.0)
            } else if height <= theme::SEA_LEVEL {
                let depth = ((theme::SEA_LEVEL - height) as f32 / 8.0).clamp(0.0, 1.0);
                shade(
                    [
                        0.30 - 0.12 * depth,
                        0.52 - 0.16 * depth,
                        0.74 - 0.14 * depth,
                    ],
                    1.0,
                )
            } else if wall.is_some_and(|wall| {
                ((point - zone_center(&wall)).magnitude() - wall.radius).abs() <= 1.6
            }) || fort.is_some_and(|fort| {
                let offset = point - zone_center(&fort);
                let reach = offset.x.abs().max(offset.y.abs());
                reach <= theme::FORT_HALF as f32 + 0.5 && reach >= theme::FORT_HALF as f32 - 1.5
            }) {
                shade([0.30, 0.30, 0.32], 1.0)
            } else if pier.is_some_and(|(start, end)| {
                crate::queries::along_segment(point, start, end).1 <= 1.6
            }) || stairs.as_ref().is_some_and(|stairs| {
                crate::queries::along_segment(point, stairs.stair_bottom, stairs.stair_top).1 <= 1.6
            }) {
                shade([0.50, 0.36, 0.22], 1.0)
            } else if road_near(generation, point, theme::ROAD_HALF + 0.8) {
                shade([0.74, 0.64, 0.46], light)
            } else {
                let weights = biome_weights(&generation.noises, x, z);
                let biome = dominant_biome(weights);
                let rise = ((height - theme::SEA_LEVEL) as f32 / 40.0).clamp(0.0, 1.0);
                let base = biome_color(biome);
                shade(
                    [
                        base[0] + 0.18 * rise,
                        base[1] + 0.06 * rise,
                        base[2] + 0.14 * rise,
                    ],
                    light,
                )
            };
            let index = (row * side + column) * 4;
            pixels[index..index + 4].copy_from_slice(&color);
        }
    }
    pixels
}

fn layout(
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    node: Entity,
    at: (i16, i16),
    size: Vec2,
) {
    ui_set_window_layout(
        dirty,
        commands,
        node,
        WindowLayout {
            position: Ab(vec2(at.0 as f32, at.1 as f32)).into(),
            size: Ab(size).into(),
            anchor: Anchor::TopLeft,
        },
    );
}

fn quantize(at: Vec2, size: f32) -> (i16, i16) {
    (
        (at.x - size / 2.0).round() as i16,
        (at.y - size / 2.0).round() as i16,
    )
}

pub fn paint(
    generation: Res<Generation>,
    handles: Res<HudHandles>,
    mut shown: ResMut<HudText>,
    mut icons: ResMut<Icons>,
    mut images: ResMut<UiImageLayerAllocator>,
    mut runtime: ResMut<RetainedUiRuntime>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if handles.root == Entity::default() || generation.zones.is_empty() {
        return;
    }
    let Some(town) = zone(&generation, TOWN) else {
        return;
    };
    let world = (generation.seed, town.center);
    if shown.map_world == Some(world) {
        return;
    }
    shown.map_world = Some(world);
    let town_center = zone_center(&town);
    let pixels = map_pixels(&generation, town_center);
    let Some(upload) = ui_upload_image(
        &mut images,
        &mut runtime,
        &pixels,
        theme::MAP_IMAGE,
        theme::MAP_IMAGE,
    ) else {
        return;
    };
    icons.map_layer = Some(upload.layer);
    icons.map_extent = upload.uv_max;
    let picture = handles.map_picture;
    let extent = upload.uv_max;
    let layer = upload.layer;
    commands.queue(move |storage, _| {
        if let Some(UiNodeContent::Image {
            texture_index,
            uv_min,
            uv_max,
        }) = storage.get_mut::<UiNodeContent>(picture)
        {
            *texture_index = layer;
            *uv_min = vec2(0.0, 0.0);
            *uv_max = extent;
        }
    });
    for (slot, (_, place)) in MAP_MARKS.iter().enumerate() {
        let Some(zone) = zone(&generation, *place) else {
            continue;
        };
        let at = to_map(town_center, zone_center(&zone)) - vec2(0.0, 14.0);
        if let Some(node) = handles.map_icons.get(slot) {
            layout(
                &mut dirty,
                &mut commands,
                *node,
                quantize(at, ICON),
                vec2(ICON, ICON),
            );
            ui_set_visible(&mut dirty, &mut commands, *node, true);
        }
    }
    for (place, zone) in generation.zones.iter().enumerate() {
        let at = to_map(town_center, zone_center(zone));
        if let Some(node) = handles.map_labels.get(place) {
            layout(
                &mut dirty,
                &mut commands,
                *node,
                (
                    (at.x - LABEL_WIDTH / 2.0).round() as i16,
                    (at.y + 8.0).round() as i16,
                ),
                vec2(LABEL_WIDTH, LABEL_HEIGHT),
            );
            ui_set_visible(&mut dirty, &mut commands, *node, true);
        }
    }
}

pub fn show(
    actions: Res<Actions<Ask>>,
    generation: Res<Generation>,
    places: Res<Places>,
    handles: Res<HudHandles>,
    mut journal: ResMut<Journal>,
    mut shown: ResMut<HudText>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
    mobs: Query<(&Name, &Health, &LocalTransform), With<Worth>>,
) {
    if handles.root == Entity::default() {
        return;
    }
    if actions.just_pressed(Ask::Map) {
        journal.clock.map_open = !journal.clock.map_open;
    }
    let open = journal.clock.map_open;
    if shown.map_open != open {
        ui_set_visible(&mut dirty, &mut commands, handles.map_root, open);
        shown.map_open = open;
    }
    if !open {
        return;
    }
    let Some(town) = zone(&generation, TOWN) else {
        return;
    };
    let town_center = zone_center(&town);

    let mut position = None;
    players.for_each(|_entity, (transform,)| position = Some(transform.translation));
    let Some(position) = position else {
        return;
    };
    let player = quantize(
        to_map(town_center, vec2(position.x, position.z)),
        PLAYER_DOT,
    );
    if player != shown.map_player {
        layout(
            &mut dirty,
            &mut commands,
            handles.map_player,
            player,
            vec2(PLAYER_DOT, PLAYER_DOT),
        );
        shown.map_player = player;
    }

    let mut alive: Vec<(String, Vec2)> = Vec::new();
    mobs.for_each(|_entity, (name, health, transform)| {
        if is_alive(health.0) {
            alive.push((
                name.0.clone(),
                vec2(transform.translation.x, transform.translation.z),
            ));
        }
    });
    let here = vec2(position.x, position.z);
    let mut targets: Vec<(u8, i16, i16)> = Vec::new();
    let mut titles = Vec::new();
    for state in &journal.quests.active {
        let quest = &QUESTS[state.quest];
        titles.push(quest.title);
        let giver_spot = |giver: usize| spot_of(&places, GIVERS[giver].place, GIVERS[giver].offset);
        let spot = if state.complete {
            giver_spot(turn_in_giver(state.quest))
        } else {
            match quest.objective {
                Objective::Kill { prey, .. } => {
                    let mut nearest = None;
                    let mut best = f32::MAX;
                    for (name, spot) in &alive {
                        if name != prey {
                            continue;
                        }
                        let distance = (spot - here).magnitude();
                        if distance < best {
                            best = distance;
                            nearest = Some(*spot);
                        }
                    }
                    nearest.unwrap_or_else(|| {
                        SPAWNS
                            .iter()
                            .find(|spawn| spawn.name == prey)
                            .map(|spawn| place_spot(&places, spawn.place).center)
                            .unwrap_or_else(|| giver_spot(quest.giver))
                    })
                }
                Objective::Gather { pickup, .. } => {
                    place_spot(&places, PICKUPS[pickup].place).center
                }
                Objective::Deliver { giver } => giver_spot(giver),
                Objective::Discover { place } => place_spot(&places, place).center,
            }
        };
        let at = quantize(to_map(town_center, spot), MARK_DOT);
        targets.push((1, at.0, at.1));
    }
    targets.truncate(theme::MAP_MARK_COUNT);
    if targets != shown.map_marks {
        for index in 0..theme::MAP_MARK_COUNT {
            let node = handles.map_marks[index];
            match (targets.get(index), shown.map_marks.get(index)) {
                (Some(new), old) => {
                    if old.is_none() {
                        ui_set_visible(&mut dirty, &mut commands, node, true);
                    }
                    if old != Some(new) {
                        layout(
                            &mut dirty,
                            &mut commands,
                            node,
                            (new.1, new.2),
                            vec2(MARK_DOT, MARK_DOT),
                        );
                    }
                }
                (None, Some(_)) => ui_set_visible(&mut dirty, &mut commands, node, false),
                (None, None) => {}
            }
        }
        shown.map_marks = targets;
    }

    let note = if titles.is_empty() {
        "No quests. The notice board in town has work.".to_string()
    } else {
        format!("Quests: {}", titles.join("   "))
    };
    if note != shown.map_note {
        ui_set_text(&mut commands, handles.map_note, &note);
        shown.map_note = note;
    }
}
