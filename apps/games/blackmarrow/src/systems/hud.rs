use crate::data::rules::MAX_HEALTH;
use crate::data::theme::*;
use crate::queries::room_at;
use crate::resources::{BlackmarrowResources, HudHandles, Screen};
use crate::tags::Torch;
use nightshade::prelude::*;
use nightshade_flashlight::queries::{remaining, shining};
use nightshade_flashlight::{Battery, FlashlightSettings, Lit};
use nightshade_text::prelude::{TextAlignment, VerticalAlignment};
use nightshade_ui::prelude::*;

pub fn build(tree: &mut UiTreeBuilder) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .without_pointer_events()
        .entity();

    let mut handles = HudHandles {
        root,
        ..Default::default()
    };

    tree.in_parent(root, |tree| {
        handles.state = line(
            tree,
            Rl(vec2(0.0, 0.0)) + Ab(vec2(26.0, 22.0)),
            Anchor::TopLeft,
            TextAlignment::Left,
            HUD_SIZE,
        );
        handles.cell = line(
            tree,
            Rl(vec2(0.0, 0.0)) + Ab(vec2(26.0, 46.0)),
            Anchor::TopLeft,
            TextAlignment::Left,
            LABEL_SIZE,
        );
        handles.message = line(
            tree,
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 30.0)),
            Anchor::TopCenter,
            TextAlignment::Center,
            LABEL_SIZE,
        );
        handles.prompt = line(
            tree,
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -78.0)),
            Anchor::BottomCenter,
            TextAlignment::Center,
            LABEL_SIZE,
        );
        handles.hurt = tree
            .add_node()
            .window(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(Vec4::new(0.45, 0.04, 0.03, 0.0))
            .without_pointer_events()
            .entity();
        tree.add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(22.0, 22.0)), Anchor::Center)
            .with_text("+", 15.0)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(TEXT_DIM)
            .without_pointer_events();
    });

    handles
}

pub fn show(
    resources: Res<BlackmarrowResources>,
    settings: Res<FlashlightSettings>,
    mut commands: Commands,
    mut torches: Query<(&Battery, &Lit), WithTag<Torch>>,
) {
    let handles = resources.hud;
    if handles.root == Entity::default() || resources.screen != Screen::Playing {
        return;
    }

    let room = room_at(resources.player.at)
        .map(|room| room.name)
        .unwrap_or("the dark");
    let health = (resources.player.health / MAX_HEALTH * 100.0)
        .round()
        .max(0.0) as u32;
    let stance = if resources.player.crouching {
        "low"
    } else {
        "upright"
    };
    ui_set_text(
        &mut commands,
        handles.state,
        &format!("{room}   {health}   {stance}"),
    );

    let mut charge = 0.0;
    let mut lit = false;
    torches.for_each_mut(|_entity, (battery, on)| {
        charge = remaining(battery, &settings);
        lit = shining(on);
    });
    let cells = resources.player.spare_cells;
    let beam = if lit { "on" } else { "off" };
    ui_set_text(
        &mut commands,
        handles.cell,
        &format!(
            "torch {beam}   cell {}%   spares {cells}",
            (charge * 100.0).round().max(0.0) as u32
        ),
    );
    ui_set_color::<UiBase>(
        &mut commands,
        handles.cell,
        if charge < 0.2 { TEXT_WARN } else { TEXT_DIM },
    );

    ui_set_text(&mut commands, handles.prompt, &resources.run.prompt);
    let message = if resources.run.message_until > resources.run.clock {
        resources.run.message.as_str()
    } else {
        ""
    };
    ui_set_text(&mut commands, handles.message, message);

    let hurt = (resources.run.hurt_until - resources.run.clock).clamp(0.0, 0.35) / 0.35;
    ui_set_color::<UiBase>(
        &mut commands,
        handles.hurt,
        Vec4::new(0.45, 0.04, 0.03, hurt * 0.55),
    );
}

fn line(
    tree: &mut UiTreeBuilder,
    position: UiValue<Vec2>,
    anchor: Anchor,
    alignment: TextAlignment,
    size: f32,
) -> Entity {
    tree.add_node()
        .window(position, Ab(vec2(760.0, size + 12.0)), anchor)
        .with_text("", size)
        .with_text_alignment(alignment, VerticalAlignment::Middle)
        .with_text_outline(TEXT_OUTLINE, 1.2)
        .color_raw::<UiBase>(TEXT_PRIMARY)
        .without_pointer_events()
        .entity()
}

pub fn menu_style() -> nightshade_ui_menu::prelude::MenuStyle {
    nightshade_ui_menu::prelude::MenuStyle {
        background: Vec4::new(0.02, 0.021, 0.024, 0.95),
        border: Vec4::new(0.3, 0.32, 0.36, 0.8),
        row_color: TEXT_DIM,
        corner_radius: 2.0,
        border_width: 1.0,
        padding: 16.0,
        spacing: 9.0,
        visible: true,
        ..Default::default()
    }
}
