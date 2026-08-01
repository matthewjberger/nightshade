use crate::contract::BoomstickEvent;
use crate::data;
use crate::data::arsenal::{GUN_COUNT, GUNS};
use crate::data::levels::plan;
use crate::queries::{held, keys_line, rounds};
use crate::resources::{BoomstickResources, HudHandles};
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

const PANEL_BACKGROUND: Vec4 = Vec4::new(0.04, 0.03, 0.04, 0.70);
const PANEL_BORDER: Vec4 = Vec4::new(1.0, 0.56, 0.28, 0.26);
const HEALTH_COLOR: Vec4 = Vec4::new(0.92, 0.44, 0.34, 1.0);
const HURT_COLOR: Vec4 = Vec4::new(1.0, 0.22, 0.16, 1.0);
const ARMOR_COLOR: Vec4 = Vec4::new(0.52, 0.76, 0.98, 1.0);
const AMMO_COLOR: Vec4 = Vec4::new(0.98, 0.84, 0.46, 1.0);
const LABEL_COLOR: Vec4 = Vec4::new(0.66, 0.68, 0.76, 1.0);
const STATUS_COLOR: Vec4 = Vec4::new(0.94, 0.92, 0.88, 1.0);
const KEY_COLOR: Vec4 = Vec4::new(0.98, 0.86, 0.52, 1.0);
const OWNED_COLOR: Vec4 = Vec4::new(0.78, 0.80, 0.88, 1.0);
const EMPTY_COLOR: Vec4 = Vec4::new(0.28, 0.29, 0.34, 1.0);
const HELD_COLOR: Vec4 = Vec4::new(1.0, 0.88, 0.52, 1.0);
const AIM_COLOR: Vec4 = Vec4::new(0.92, 0.94, 0.96, 0.78);
const MARK_COLOR: Vec4 = Vec4::new(1.0, 0.42, 0.32, 1.0);
const CLEAR: Vec4 = Vec4::new(0.86, 0.16, 0.12, 0.0);

const PANEL_WIDTH: f32 = 178.0;
const PANEL_HEIGHT: f32 = 62.0;
const READOUT_SIZE: f32 = 34.0;
const LABEL_SIZE: f32 = 13.0;
const STATUS_WIDTH: f32 = 620.0;
const SLOT_WIDTH: f32 = 124.0;
const MARGIN: f32 = 20.0;
const AIM_GAP: f32 = 4.0;
const AIM_ARM: f32 = 8.0;
const AIM_WIDTH: f32 = 2.0;

pub fn build(tree: &mut UiTreeBuilder) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = HudHandles::default();

    tree.in_parent(root, |tree| {
        let health_panel = panel(
            tree,
            Rl(vec2(0.0, 100.0)) + Ab(vec2(MARGIN, -MARGIN)),
            Anchor::BottomLeft,
        );
        tree.in_parent(health_panel, |tree| {
            label(tree, "HEALTH");
            handles.health = readout(tree, HEALTH_COLOR);
        });

        let armor_panel = panel(
            tree,
            Rl(vec2(0.0, 100.0)) + Ab(vec2(MARGIN + PANEL_WIDTH + 10.0, -MARGIN)),
            Anchor::BottomLeft,
        );
        tree.in_parent(armor_panel, |tree| {
            label(tree, "ARMOR");
            handles.armor = readout(tree, ARMOR_COLOR);
        });

        let ammo_panel = panel(
            tree,
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-MARGIN, -MARGIN)),
            Anchor::BottomRight,
        );
        tree.in_parent(ammo_panel, |tree| {
            handles.gun = tree
                .add_node()
                .window(
                    Ab(vec2(14.0, 6.0)),
                    Ab(vec2(PANEL_WIDTH - 28.0, 18.0)),
                    Anchor::TopLeft,
                )
                .with_text("", LABEL_SIZE)
                .text_left()
                .color_raw::<UiBase>(LABEL_COLOR)
                .entity();
            handles.ammo = readout(tree, AMMO_COLOR);
        });

        for (slot, gun) in GUNS.iter().enumerate() {
            let shift = (slot as f32 - (GUN_COUNT as f32 - 1.0) * 0.5) * SLOT_WIDTH;
            handles.slots[slot] = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 100.0)) + Ab(vec2(shift - SLOT_WIDTH * 0.5, -MARGIN - 24.0)),
                    Ab(vec2(SLOT_WIDTH, 20.0)),
                    Anchor::TopLeft,
                )
                .with_text(gun.name, LABEL_SIZE)
                .color_raw::<UiBase>(EMPTY_COLOR)
                .entity();
        }

        handles.keys = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(-STATUS_WIDTH * 0.5, -MARGIN - 48.0)),
                Ab(vec2(STATUS_WIDTH, 20.0)),
                Anchor::TopLeft,
            )
            .with_text("", 15.0)
            .color_raw::<UiBase>(KEY_COLOR)
            .entity();

        handles.status = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(-STATUS_WIDTH * 0.5, MARGIN + 26.0)),
                Ab(vec2(STATUS_WIDTH, 30.0)),
                Anchor::TopLeft,
            )
            .with_text("", 21.0)
            .color_raw::<UiBase>(STATUS_COLOR)
            .entity();

        handles.tally = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(-STATUS_WIDTH * 0.5, MARGIN)),
                Ab(vec2(STATUS_WIDTH, 24.0)),
                Anchor::TopLeft,
            )
            .with_text("", 15.0)
            .color_raw::<UiBase>(LABEL_COLOR)
            .entity();

        let arms = [
            (
                Ab(vec2(-AIM_GAP - AIM_ARM, -AIM_WIDTH * 0.5)),
                Ab(vec2(AIM_ARM, AIM_WIDTH)),
            ),
            (
                Ab(vec2(AIM_GAP, -AIM_WIDTH * 0.5)),
                Ab(vec2(AIM_ARM, AIM_WIDTH)),
            ),
            (
                Ab(vec2(-AIM_WIDTH * 0.5, -AIM_GAP - AIM_ARM)),
                Ab(vec2(AIM_WIDTH, AIM_ARM)),
            ),
            (
                Ab(vec2(-AIM_WIDTH * 0.5, AIM_GAP)),
                Ab(vec2(AIM_WIDTH, AIM_ARM)),
            ),
        ];
        for (index, (at, size)) in arms.into_iter().enumerate() {
            handles.crosshair[index] = tree
                .add_node()
                .window(Rl(vec2(50.0, 50.0)) + at, size, Anchor::TopLeft)
                .with_rect(0.0, 0.0, AIM_COLOR)
                .color_raw::<UiBase>(AIM_COLOR)
                .entity();
        }

        handles.flash = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, CLEAR)
            .color_raw::<UiBase>(CLEAR)
            .entity();
    });

    handles.root = root;
    handles
}

fn panel(tree: &mut UiTreeBuilder, at: impl Into<UiValue<Vec2>>, anchor: Anchor) -> Entity {
    tree.add_node()
        .window(at, Ab(vec2(PANEL_WIDTH, PANEL_HEIGHT)), anchor)
        .with_rect(6.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BACKGROUND)
        .entity()
}

fn label(tree: &mut UiTreeBuilder, text: &str) {
    tree.add_node()
        .window(
            Ab(vec2(14.0, 6.0)),
            Ab(vec2(PANEL_WIDTH - 28.0, 18.0)),
            Anchor::TopLeft,
        )
        .with_text(text, LABEL_SIZE)
        .text_left()
        .color_raw::<UiBase>(LABEL_COLOR);
}

fn readout(tree: &mut UiTreeBuilder, color: Vec4) -> Entity {
    tree.add_node()
        .window(
            Ab(vec2(14.0, 22.0)),
            Ab(vec2(PANEL_WIDTH - 28.0, 36.0)),
            Anchor::TopLeft,
        )
        .with_text("", READOUT_SIZE)
        .text_left()
        .color_raw::<UiBase>(color)
        .entity()
}

pub fn show(
    time: Res<nightshade_platform::contract::time::Time>,
    mut resources: ResMut<BoomstickResources>,
    events: EventReader<BoomstickEvent>,
    mut commands: Commands,
) {
    let delta_time = time.delta_time;
    let handles = resources.hud;
    if handles.root == Entity::default() {
        return;
    }

    let mut picked = None;
    let mut landed = false;
    for event in &events {
        match event {
            BoomstickEvent::PickedUp { label } => picked = Some(format!("you got {label}")),
            BoomstickEvent::SecretFound => picked = Some("a secret opens".to_string()),
            BoomstickEvent::SwitchThrown => picked = Some("something grinds open".to_string()),
            BoomstickEvent::EnemyHit { .. } | BoomstickEvent::EnemyKilled => landed = true,
            _ => {}
        }
    }
    if landed {
        resources.player.hit_mark = data::HIT_MARK_TIME;
    } else {
        resources.player.hit_mark = (resources.player.hit_mark - delta_time).max(0.0);
    }

    let health = resources.player.health.max(0.0).round() as u32;
    if health != resources.shown.health {
        let low = resources.player.health <= data::LOW_HEALTH;
        resources.shown.health = health;
        ui_set_text(&mut commands, handles.health, &format!("{health}"));
        ui_set_color::<UiBase>(
            &mut commands,
            handles.health,
            if low { HURT_COLOR } else { HEALTH_COLOR },
        );
    }

    let armor = resources.player.armor.max(0.0).round() as u32;
    if armor != resources.shown.armor {
        resources.shown.armor = armor;
        ui_set_text(&mut commands, handles.armor, &format!("{armor}"));
    }

    let ammo = rounds(&resources.player);
    if ammo != resources.shown.ammo {
        resources.shown.ammo = ammo;
        ui_set_text(&mut commands, handles.ammo, &format!("{ammo}"));
    }

    let name = held(&resources.player).name.to_uppercase();
    if name != resources.shown.gun_label {
        resources.shown.gun_label = name.clone();
        ui_set_text(&mut commands, handles.gun, &name);
    }

    let arsenal = format!(
        "{}{}{}{}{}",
        resources.player.gun,
        resources.player.owned[0] as u32,
        resources.player.owned[1] as u32,
        resources.player.owned[2] as u32,
        resources.player.owned[3] as u32
    );
    if arsenal != resources.shown.slots {
        resources.shown.slots = arsenal;
        for slot in 0..GUN_COUNT {
            let color = if resources.player.gun == slot {
                HELD_COLOR
            } else if resources.player.owned[slot] {
                OWNED_COLOR
            } else {
                EMPTY_COLOR
            };
            ui_set_color::<UiBase>(&mut commands, handles.slots[slot], color);
        }
    }

    let keys = keys_line(&resources.player);
    if keys != resources.shown.keys {
        resources.shown.keys = keys.clone();
        ui_set_text(&mut commands, handles.keys, &keys);
    }

    let plan = plan(resources.mission.level);
    let tally = format!(
        "{}       {} of {} down       {} of {} secrets",
        plan.name,
        resources.mission.kills,
        resources.mission.standing,
        resources.mission.secrets,
        resources.mission.hidden
    );
    if tally != resources.shown.tally {
        resources.shown.tally = tally.clone();
        ui_set_text(&mut commands, handles.tally, &tally);
    }

    let standing = if resources.mission.banner_left > 0.0 {
        resources.mission.banner.clone()
    } else if let Some(message) = picked {
        resources.shown.message_left = data::MESSAGE_TIME;
        message
    } else if resources.shown.message_left > 0.0 {
        resources.shown.message_left = (resources.shown.message_left - delta_time).max(0.0);
        resources.shown.message.clone()
    } else {
        resources.mission.hint.clone()
    };
    if standing != resources.shown.message {
        resources.shown.message = standing.clone();
        ui_set_text(&mut commands, handles.status, &standing);
    }

    let mark = (resources.player.hit_mark * 16.0).round() as u32;
    if mark != resources.shown.mark {
        resources.shown.mark = mark;
        let share = (resources.player.hit_mark / data::HIT_MARK_TIME).clamp(0.0, 1.0);
        let color = AIM_COLOR * (1.0 - share) + MARK_COLOR * share;
        for arm in handles.crosshair {
            ui_set_color::<UiBase>(&mut commands, arm, color);
        }
    }

    let flash = (resources.player.flash * 32.0).round() as u32;
    if flash != resources.shown.flash {
        resources.shown.flash = flash;
        ui_set_color::<UiBase>(
            &mut commands,
            handles.flash,
            Vec4::new(0.86, 0.16, 0.12, resources.player.flash * data::FLASH_ALPHA),
        );
    }
}
