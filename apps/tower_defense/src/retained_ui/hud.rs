use crate::ecs::{
    GameState, TowerChipHandles, TowerDefenseResources, TowerType, UiAction, UiHandles,
};
use crate::retained_ui::theme::*;
use nightshade::prelude::*;

const HP_BAR_WIDTH: f32 = 220.0;
const HP_BAR_HEIGHT: f32 = 12.0;

pub fn build(tree: &mut UiTreeBuilder, selected_tower: TowerType) -> UiHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut money_value = Entity::default();
    let mut wave_value = Entity::default();
    let mut lives_value = Entity::default();
    let mut hp_value = Entity::default();
    let mut hp_fill = Entity::default();
    let mut speed_value = Entity::default();
    let mut speed_minus = Entity::default();
    let mut speed_reset = Entity::default();
    let mut speed_plus = Entity::default();
    let mut status_root = Entity::default();
    let mut status_label = Entity::default();
    let mut announce_root = Entity::default();
    let mut announce_label = Entity::default();
    let mut tower_chips: Vec<TowerChipHandles> = Vec::with_capacity(5);

    tree.in_parent(root, |tree| {
        let resource_chip = tree
            .add_node()
            .window(
                Ab(vec2(20.0, 20.0)),
                Ab(vec2(240.0, 110.0)),
                Anchor::TopLeft,
            )
            .with_rect(10.0, 1.5, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG)
            .with_shadow(SHADOW_DARK, vec2(0.0, 6.0), 18.0, 0.0)
            .flow(FlowDirection::Vertical, 14.0, 6.0)
            .entity();
        tree.in_parent(resource_chip, |tree| {
            tree.add_node()
                .flow_child(Ab(vec2(210.0, 14.0)))
                .with_text("CREDITS", 11.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM)
                .entity();
            money_value = tree
                .add_node()
                .flow_child(Ab(vec2(210.0, 30.0)))
                .with_text("$200", 26.0)
                .text_left()
                .with_text_outline(SHADOW_DARK, 1.5)
                .color_raw::<UiBase>(ACCENT_GOLD)
                .entity();
            tree.add_node()
                .flow_child(Ab(vec2(210.0, 12.0)))
                .with_text("WAVE", 10.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_FAINT)
                .entity();
            wave_value = tree
                .add_node()
                .flow_child(Ab(vec2(210.0, 18.0)))
                .with_text("0", 16.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_COLOR)
                .entity();
        });

        let status_chip = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-20.0, 20.0)),
                Ab(vec2(260.0, 110.0)),
                Anchor::TopRight,
            )
            .with_rect(10.0, 1.5, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG)
            .with_shadow(SHADOW_DARK, vec2(0.0, 6.0), 18.0, 0.0)
            .flow(FlowDirection::Vertical, 14.0, 6.0)
            .entity();
        tree.in_parent(status_chip, |tree| {
            tree.add_node()
                .flow_child(Ab(vec2(232.0, 12.0)))
                .with_text("LIVES", 10.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_FAINT)
                .entity();
            lives_value = tree
                .add_node()
                .flow_child(Ab(vec2(232.0, 24.0)))
                .with_text("x20", 22.0)
                .text_left()
                .with_text_outline(SHADOW_DARK, 1.5)
                .color_raw::<UiBase>(ACCENT_GREEN)
                .entity();
            hp_value = tree
                .add_node()
                .flow_child(Ab(vec2(232.0, 14.0)))
                .with_text("HP 20/20", 11.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM)
                .entity();

            let hp_track = tree
                .add_node()
                .flow_child(Ab(vec2(HP_BAR_WIDTH, HP_BAR_HEIGHT)))
                .with_rect(6.0, 1.0, PANEL_BORDER)
                .color_raw::<UiBase>(HP_TRACK)
                .entity();
            tree.in_parent(hp_track, |tree| {
                hp_fill = tree
                    .add_node()
                    .window(
                        Ab(vec2(0.0, 0.0)),
                        Ab(vec2(HP_BAR_WIDTH, HP_BAR_HEIGHT)),
                        Anchor::TopLeft,
                    )
                    .with_rect(6.0, 0.0, TRANSPARENT)
                    .color_raw::<UiBase>(HP_GREEN)
                    .entity();
            });
        });

        let speed_chip = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-20.0, 144.0)),
                Ab(vec2(260.0, 56.0)),
                Anchor::TopRight,
            )
            .with_rect(10.0, 1.5, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .with_shadow(SHADOW_DARK, vec2(0.0, 4.0), 12.0, 0.0)
            .flow_with_alignment(
                FlowDirection::Horizontal,
                10.0,
                8.0,
                FlowAlignment::Start,
                FlowAlignment::Center,
            )
            .entity();
        tree.in_parent(speed_chip, |tree| {
            tree.add_node()
                .flow_child(Ab(vec2(54.0, 18.0)))
                .with_text("SPEED", 10.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_FAINT)
                .entity();
            speed_value = tree
                .add_node()
                .flow_child(Ab(vec2(46.0, 24.0)))
                .with_text("1.0x", 18.0)
                .text_left()
                .color_raw::<UiBase>(ACCENT_BLUE)
                .entity();
            speed_minus = make_speed_button(tree, "-");
            speed_reset = make_speed_button(tree, "1x");
            speed_plus = make_speed_button(tree, "+");
        });

        announce_root = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 32.0)),
                Ab(vec2(420.0, 84.0)),
                Anchor::TopCenter,
            )
            .with_rect(12.0, 1.5, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .with_shadow(SHADOW_DARK, vec2(0.0, 8.0), 24.0, 0.0)
            .entity();
        tree.in_parent(announce_root, |tree| {
            announce_label = tree
                .add_node()
                .window(Rl(vec2(50.0, 50.0)), Ab(vec2(400.0, 60.0)), Anchor::Center)
                .with_text("WAVE 1", 36.0)
                .text_center()
                .with_text_outline(SHADOW_DARK, 2.5)
                .color_raw::<UiBase>(ACCENT_GOLD)
                .entity();
        });

        status_root = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(520.0, 120.0)), Anchor::Center)
            .with_rect(14.0, 2.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG_DEEP)
            .with_shadow(SHADOW_DARK, vec2(0.0, 10.0), 32.0, 0.0)
            .entity();
        tree.in_parent(status_root, |tree| {
            status_label = tree
                .add_node()
                .window(Rl(vec2(50.0, 50.0)), Ab(vec2(500.0, 80.0)), Anchor::Center)
                .with_text("GAME OVER", 48.0)
                .text_center()
                .with_text_outline(SHADOW_DARK, 3.0)
                .color_raw::<UiBase>(WHITE)
                .entity();
        });

        let tower_panel = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -110.0)),
                Ab(vec2(720.0, 90.0)),
                Anchor::TopCenter,
            )
            .with_rect(12.0, 1.5, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BG)
            .with_shadow(SHADOW_DARK, vec2(0.0, 8.0), 22.0, 0.0)
            .flow_with_alignment(
                FlowDirection::Horizontal,
                12.0,
                10.0,
                FlowAlignment::Center,
                FlowAlignment::Center,
            )
            .entity();
        tree.in_parent(tower_panel, |tree| {
            for tower_type in TowerType::all() {
                let chip = make_tower_chip(tree, tower_type, tower_type == selected_tower);
                tower_chips.push(chip);
            }
        });

        tree.add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -16.0)),
                Ab(vec2(560.0, 16.0)),
                Anchor::TopCenter,
            )
            .with_text(
                "LEFT CLICK PLACE  /  RIGHT CLICK SELL  /  1-5 SELECT  /  [ ]  SPEED",
                11.0,
            )
            .text_center()
            .color_raw::<UiBase>(TEXT_FAINT)
            .entity();
    });

    UiHandles {
        money_value,
        lives_value,
        wave_value,
        hp_value,
        hp_fill,
        speed_value,
        speed_minus,
        speed_reset,
        speed_plus,
        status_root,
        status_label,
        announce_root,
        announce_label,
        tower_chips,
    }
}

fn make_tower_chip(
    tree: &mut UiTreeBuilder,
    tower_type: TowerType,
    selected: bool,
) -> TowerChipHandles {
    let accent = tower_type.color();
    let chip = tree
        .add_node()
        .flow_child(Ab(vec2(132.0, 64.0)))
        .with_rect(8.0, 1.5, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG_DEEP)
        .color_raw::<UiHover>(PANEL_HOVER)
        .color_raw::<UiPressed>(PANEL_PRESSED)
        .color_raw::<UiSelected>(vec4(accent.x, accent.y, accent.z, 0.35))
        .with_transition::<UiHover>(12.0, 6.0)
        .with_transition::<UiPressed>(18.0, 10.0)
        .with_transition::<UiSelected>(10.0, 5.0)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();

    if let Some(color) = tree
        .world_mut()
        .get_mut::<nightshade::ecs::ui::components::UiNodeColor>(chip)
    {
        color.colors[UiSelected::INDEX] = if selected {
            Some(vec4(accent.x, accent.y, accent.z, 0.35))
        } else {
            None
        };
    }
    if let Some(weights) = tree
        .world_mut()
        .get_mut::<nightshade::ecs::ui::components::UiStateWeights>(chip)
    {
        weights.weights[UiSelected::INDEX] = if selected { 1.0 } else { 0.0 };
    }

    tree.in_parent(chip, |tree| {
        tree.add_node()
            .window(Ab(vec2(8.0, 6.0)), Ab(vec2(18.0, 18.0)), Anchor::TopLeft)
            .with_rect(4.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(vec4(accent.x, accent.y, accent.z, 1.0))
            .entity();
        tree.add_node()
            .window(Ab(vec2(8.0, 6.0)), Ab(vec2(18.0, 18.0)), Anchor::TopLeft)
            .with_text(tower_type.hotkey_label(), 12.0)
            .text_center()
            .color_raw::<UiBase>(WHITE)
            .entity();

        tree.add_node()
            .window(Ab(vec2(32.0, 6.0)), Ab(vec2(94.0, 20.0)), Anchor::TopLeft)
            .with_text(tower_type.name(), 16.0)
            .text_left()
            .color_raw::<UiBase>(WHITE)
            .entity();
        tree.add_node()
            .window(Ab(vec2(32.0, 26.0)), Ab(vec2(94.0, 14.0)), Anchor::TopLeft)
            .with_text(&format!("${}", tower_type.cost()), 12.0)
            .text_left()
            .color_raw::<UiBase>(ACCENT_GOLD)
            .entity();
        tree.add_node()
            .window(Ab(vec2(8.0, 44.0)), Ab(vec2(118.0, 14.0)), Anchor::TopLeft)
            .with_text(tower_type.description(), 9.5)
            .text_left()
            .color_raw::<UiBase>(TEXT_DIM)
            .entity();
    });

    TowerChipHandles { root: chip }
}

fn make_speed_button(tree: &mut UiTreeBuilder, glyph: &str) -> Entity {
    let button = tree
        .add_node()
        .flow_child(Ab(vec2(34.0, 28.0)))
        .with_rect(6.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG)
        .color_raw::<UiHover>(PANEL_HOVER)
        .color_raw::<UiPressed>(PANEL_PRESSED)
        .with_transition::<UiHover>(14.0, 6.0)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.in_parent(button, |tree| {
        tree.add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(30.0, 24.0)), Anchor::Center)
            .with_text(glyph, 14.0)
            .text_center()
            .color_raw::<UiBase>(TEXT_COLOR)
            .entity();
    });
    button
}

pub fn poll(game_world: &mut TowerDefenseResources, world: &mut World) {
    let handles = &game_world.ui_handles;
    let chip_targets: Vec<(Entity, TowerType)> = handles
        .tower_chips
        .iter()
        .enumerate()
        .map(|(index, chip)| (chip.root, TowerType::all()[index]))
        .collect();
    let speed_minus = handles.speed_minus;
    let speed_reset = handles.speed_reset;
    let speed_plus = handles.speed_plus;

    let mut actions: Vec<UiAction> = Vec::new();

    for event in ui_events(world) {
        if let UiEvent::ButtonClicked(entity) = event {
            if let Some((_, tower_type)) =
                chip_targets.iter().find(|(handle, _)| *handle == *entity)
            {
                actions.push(UiAction::SelectTower(*tower_type));
                continue;
            }
            if *entity == speed_minus {
                actions.push(UiAction::DecreaseSpeed);
            } else if *entity == speed_reset {
                actions.push(UiAction::ResetSpeed);
            } else if *entity == speed_plus {
                actions.push(UiAction::IncreaseSpeed);
            }
        }
    }

    game_world.ui_interaction.actions.extend(actions);
}

pub fn update(game_world: &TowerDefenseResources, world: &mut World) {
    let handles = &game_world.ui_handles;

    ui_set_text(
        world,
        handles.money_value,
        &format!("${}", game_world.money),
    );
    ui_set_text(world, handles.wave_value, &format!("{}", game_world.wave));
    ui_set_text(
        world,
        handles.lives_value,
        &format!("x{}", game_world.lives),
    );
    ui_set_text(
        world,
        handles.hp_value,
        &format!("HP {}/{}", game_world.current_hp, game_world.max_hp),
    );
    ui_set_text(
        world,
        handles.speed_value,
        &format!("{:.1}x", game_world.game_speed),
    );

    let hp_ratio = (game_world.current_hp as f32 / game_world.max_hp.max(1) as f32).clamp(0.0, 1.0);

    let combined_ratio = if game_world.lives == 0 {
        0.0
    } else {
        let total = game_world.lives as f32 * game_world.max_hp as f32;
        let current =
            (game_world.lives - 1) as f32 * game_world.max_hp as f32 + game_world.current_hp as f32;
        (current / total).clamp(0.0, 1.0)
    };

    let bar_color = if combined_ratio > 0.5 {
        HP_GREEN
    } else if combined_ratio > 0.25 {
        HP_AMBER
    } else {
        HP_RED
    };

    if let Some(node) =
        world.get_mut::<nightshade::ecs::ui::components::UiLayoutNode>(handles.hp_fill)
    {
        node.base_layout = Some(UiLayoutType::Window(WindowLayout {
            position: Ab(vec2(0.0, 0.0)).into(),
            size: Ab(vec2(HP_BAR_WIDTH * hp_ratio, HP_BAR_HEIGHT)).into(),
            anchor: Anchor::TopLeft,
        }));
    }
    if let Some(color) =
        world.get_mut::<nightshade::ecs::ui::components::UiNodeColor>(handles.hp_fill)
    {
        color.colors[UiBase::INDEX] = Some(bar_color);
    }

    let lives_color = if game_world.lives > 5 {
        ACCENT_GREEN
    } else if game_world.lives > 2 {
        ACCENT_GOLD
    } else {
        ACCENT_RED
    };
    if let Some(color) =
        world.get_mut::<nightshade::ecs::ui::components::UiNodeColor>(handles.lives_value)
    {
        color.colors[UiBase::INDEX] = Some(lives_color);
    }

    for (index, chip) in handles.tower_chips.iter().enumerate() {
        let tower_type = TowerType::all()[index];
        let is_selected = tower_type == game_world.selected_tower_type;
        ui_set_selected(world, chip.root, is_selected);
    }

    let announce_visible = game_world.wave_announce_timer > 0.0;
    ui_set_visible(world, handles.announce_root, announce_visible);
    if announce_visible {
        let wave_num = game_world.wave;
        let label = if wave_num.is_multiple_of(5) {
            format!("BOSS WAVE {wave_num}")
        } else {
            format!("WAVE {wave_num}")
        };
        ui_set_text(world, handles.announce_label, &label);
    }

    let status_visible = game_world.game_state == GameState::GameOver;
    ui_set_visible(world, handles.status_root, status_visible);
    if status_visible {
        ui_set_text(world, handles.status_label, "GAME OVER");
    }
}
