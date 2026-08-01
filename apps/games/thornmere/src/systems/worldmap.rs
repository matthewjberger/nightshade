use crate::components::{Cooldowns, Health, Worth};
use crate::data::Ask;
use crate::data::{
    FRAME_CORNER, GIVERS, MAP_MARKS, MAP_PANEL, Objective, PICKUPS, PLACES, QUESTS, SPAWNS,
    ZONE_SIZE,
};
use crate::queries::is_alive;
use crate::resources::{HudHandles, Icons, MAP_MARK_COUNT, ThornmereResources};
use crate::systems::hud::frame;
use crate::systems::quests::turn_in_giver;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_scene::prelude::{LocalTransform, Name};
use nightshade_ui::components::layout_types::WindowLayout;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

const SCRIM: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.6);
const PANEL: Vec4 = Vec4::new(0.16, 0.12, 0.08, 0.98);
const BORDER: Vec4 = Vec4::new(0.85, 0.70, 0.40, 0.9);
const TITLE: Vec4 = Vec4::new(1.0, 0.88, 0.55, 1.0);
const LABEL: Vec4 = Vec4::new(1.0, 0.97, 0.90, 1.0);
const LABEL_SHADOW: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.55);
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

fn to_map(spot: Vec2) -> Vec2 {
    let half = ZONE_SIZE / 2.0;
    vec2(
        (spot.x + half) / ZONE_SIZE * MAP_PANEL,
        (spot.y + half) / ZONE_SIZE * MAP_PANEL,
    )
}

fn mark_icon(tree: &mut UiTreeBuilder, icons: &Icons, name: &str, at: Vec2) {
    let Some(layer) = icons.mark_layer else {
        return;
    };
    let Some(slot) = nightshade_kit::prelude::MAP_ICONS
        .iter()
        .position(|(icon, _)| *icon == name)
    else {
        return;
    };
    let Some((uv_min, uv_max)) = icons.marks.get(slot).copied() else {
        return;
    };
    tree.add_node()
        .window(
            Ab(vec2(at.x - ICON / 2.0, at.y - ICON / 2.0)),
            Ab(vec2(ICON, ICON)),
            Anchor::TopLeft,
        )
        .with_image_uv(layer, uv_min, uv_max)
        .color_raw::<UiBase>(Vec4::new(1.0, 1.0, 1.0, 1.0))
        .without_pointer_events()
        .entity();
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
    let width = MAP_PANEL + FRAME * 2.0;
    let height = MAP_PANEL + FRAME * 2.0 + TITLE_HEIGHT + NOTE_HEIGHT;
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
            frame(tree, icons.frame_layer, icons.tint, FRAME_CORNER * 1.4);
            tree.add_node()
                .window(
                    Ab(vec2(0.0, 10.0)),
                    Ab(vec2(width, TITLE_HEIGHT - 6.0)),
                    Anchor::TopLeft,
                )
                .with_text("Thornmere Vale", 28.0)
                .color_raw::<UiBase>(TITLE)
                .without_pointer_events()
                .entity();
            let picture = tree
                .add_node()
                .window(
                    Ab(vec2(FRAME, TITLE_HEIGHT + FRAME / 2.0)),
                    Ab(vec2(MAP_PANEL, MAP_PANEL)),
                    Anchor::TopLeft,
                )
                .with_image_uv(
                    icons.map_layer.unwrap_or(0),
                    vec2(0.0, 0.0),
                    icons.map_extent,
                )
                .color_raw::<UiBase>(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .without_pointer_events()
                .entity();
            tree.in_parent(picture, |tree| {
                for (icon, place) in MAP_MARKS {
                    let at = to_map(PLACES[place].center);
                    mark_icon(tree, icons, icon, at - vec2(0.0, 14.0));
                }
                for place in &PLACES {
                    let at = to_map(place.center);
                    for (offset, color, size) in [
                        (vec2(1.5, 1.5), LABEL_SHADOW, 15.0),
                        (vec2(0.0, 0.0), LABEL, 15.0),
                    ] {
                        tree.add_node()
                            .window(
                                Ab(vec2(at.x - 90.0 + offset.x, at.y + 8.0 + offset.y)),
                                Ab(vec2(180.0, 22.0)),
                                Anchor::TopLeft,
                            )
                            .with_text(place.name, size)
                            .color_raw::<UiBase>(color)
                            .without_pointer_events()
                            .entity();
                    }
                }
                let compass = to_map(vec2(ZONE_SIZE / 2.0 - 22.0, ZONE_SIZE / 2.0 - 22.0));
                mark_icon(tree, icons, "compass", compass);
                for _ in 0..MAP_MARK_COUNT {
                    let mark = tree
                        .add_node()
                        .window(
                            Ab(vec2(0.0, 0.0)),
                            Ab(vec2(MARK_DOT, MARK_DOT)),
                            Anchor::TopLeft,
                        )
                        .with_rect(MARK_DOT / 2.0, 2.0, LABEL_SHADOW)
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
                    Ab(vec2(FRAME, TITLE_HEIGHT + FRAME + MAP_PANEL + 4.0)),
                    Ab(vec2(MAP_PANEL, NOTE_HEIGHT - 8.0)),
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

fn quantize(at: Vec2, size: f32) -> (i16, i16) {
    (
        (at.x - size / 2.0).round() as i16,
        (at.y - size / 2.0).round() as i16,
    )
}

fn layout(
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    node: Entity,
    at: (i16, i16),
    size: f32,
) {
    ui_set_window_layout(
        dirty,
        commands,
        node,
        WindowLayout {
            position: Ab(vec2(at.0 as f32, at.1 as f32)).into(),
            size: Ab(vec2(size, size)).into(),
            anchor: Anchor::TopLeft,
        },
    );
}

pub fn show(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<ThornmereResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
    mobs: Query<(&Name, &Health, &LocalTransform), With<Worth>>,
) {
    let resources = &mut *resources;
    if resources.hud.root == Entity::default() {
        return;
    }
    if actions.just_pressed(Ask::Map) {
        resources.clock.map_open = !resources.clock.map_open;
    }
    let open = resources.clock.map_open;
    if resources.hud_text.map_open != open {
        ui_set_visible(&mut dirty, &mut commands, resources.hud.map_root, open);
        resources.hud_text.map_open = open;
    }
    if !open {
        return;
    }

    let mut position = None;
    players.for_each(|_entity, (transform,)| position = Some(transform.translation));
    let Some(position) = position else {
        return;
    };
    let player = quantize(to_map(vec2(position.x, position.z)), PLAYER_DOT);
    if player != resources.hud_text.map_player {
        layout(
            &mut dirty,
            &mut commands,
            resources.hud.map_player,
            player,
            PLAYER_DOT,
        );
        resources.hud_text.map_player = player;
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
    for state in &resources.quests.active {
        let quest = &QUESTS[state.quest];
        titles.push(quest.title);
        let spot = if state.complete {
            GIVERS[turn_in_giver(state.quest)].spot
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
                            .map(|spawn| PLACES[spawn.place].center)
                            .unwrap_or(GIVERS[quest.giver].spot)
                    })
                }
                Objective::Gather { pickup, .. } => PLACES[PICKUPS[pickup].place].center,
                Objective::Deliver { giver } => GIVERS[giver].spot,
                Objective::Discover { place } => PLACES[place].center,
            }
        };
        let at = quantize(to_map(spot), MARK_DOT);
        targets.push((1, at.0, at.1));
    }
    targets.truncate(MAP_MARK_COUNT);
    if targets != resources.hud_text.map_marks {
        let shown = resources.hud_text.map_marks.clone();
        for index in 0..MAP_MARK_COUNT {
            let node = resources.hud.map_marks[index];
            match (targets.get(index), shown.get(index)) {
                (Some(new), old) => {
                    if old.is_none() {
                        ui_set_visible(&mut dirty, &mut commands, node, true);
                    }
                    if old != Some(new) {
                        layout(&mut dirty, &mut commands, node, (new.1, new.2), MARK_DOT);
                    }
                }
                (None, Some(_)) => ui_set_visible(&mut dirty, &mut commands, node, false),
                (None, None) => {}
            }
        }
        resources.hud_text.map_marks = targets;
    }

    let note = if titles.is_empty() {
        "No quests. The notice board in Trade Square has work.".to_string()
    } else {
        format!("Quests: {}", titles.join("  ·  "))
    };
    if note != resources.hud_text.map_note {
        ui_set_text(&mut commands, resources.hud.map_note, &note);
        resources.hud_text.map_note = note;
    }
}
