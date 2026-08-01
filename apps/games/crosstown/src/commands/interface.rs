use crate::data;
use crate::resources::{Interface, Talk};
use nalgebra_glm::vec2;
use nightshade::prelude::Commands;
use nightshade_ui::components::layout_types::WindowLayout;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::commands::add_menu_panel;
use nightshade_ui_menu::data::MenuStyle;

pub fn say(talk: &mut Talk, line: &str) {
    talk.queue.push(line.to_string());
}

pub fn set_fill(dirty: &mut RetainedUiDirty, commands: &mut Commands, entity: Entity, width: f32) {
    ui_set_window_layout(
        dirty,
        commands,
        entity,
        WindowLayout {
            position: Ab(vec2(3.0, 3.0)).into(),
            size: Ab(vec2(width, data::BAR_HEIGHT - 6.0)).into(),
            anchor: Anchor::TopLeft,
        },
    );
}

pub fn build(tree: &mut UiTreeBuilder) -> Interface {
    let mut handles = Interface::default();
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    tree.in_parent(root, |tree| {
        handles.head_up = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_visible(false)
            .entity();
        tree.in_parent(handles.head_up, |tree| {
            head_up(tree, &mut handles);
        });

        handles.title = add_menu_panel(tree, &menu_style(), 560.0, 300.0);
        tree.in_parent(handles.title, |tree| {
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 86.0)))
                .with_text(data::TITLE, data::TITLE_SIZE)
                .text_center()
                .color_raw::<UiBase>(colour(data::GO_COLOR));
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)))
                .with_text(data::TAGLINE, data::CAPTION_SIZE)
                .text_center()
                .color_raw::<UiBase>(colour(data::DIM_COLOR));
            handles.story = tree.add_button("STORY");
            handles.arcade = tree.add_button("ARCADE");
            handles.leave = tree.add_button("EXIT");
        });

        handles.pause = add_menu_panel(tree, &menu_style(), 560.0, 210.0);
        tree.in_parent(handles.pause, |tree| {
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 50.0)))
                .with_text("PAUSED", 42.0)
                .text_center()
                .color_raw::<UiBase>(colour(data::TEXT_COLOR));
            handles.resume = tree.add_button("RESUME");
            handles.menu = tree.add_button("MAIN MENU");
        });

        handles.over = add_menu_panel(tree, &menu_style(), 520.0, 280.0);
        tree.in_parent(handles.over, |tree| {
            handles.over_title = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 58.0)))
                .with_text("GAME OVER", 46.0)
                .text_center()
                .color_raw::<UiBase>(colour(data::GO_COLOR))
                .entity();
            handles.over_score = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)))
                .with_text("", data::SCORE_SIZE)
                .text_center()
                .color_raw::<UiBase>(colour(data::TEXT_COLOR))
                .entity();
            handles.again = tree.add_button("FIGHT AGAIN");
            handles.back = tree.add_button("MAIN MENU");
        });
    });

    handles
}

fn head_up(tree: &mut UiTreeBuilder, handles: &mut Interface) {
    let hero_panel = tree
        .add_node()
        .window(
            Ab(vec2(data::BAR_MARGIN, data::BAR_MARGIN)),
            Ab(vec2(data::BAR_WIDTH, data::BAR_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(4.0, 1.0, colour(data::PANEL_EDGE))
        .color_raw::<UiBase>(colour(data::PANEL_BACK))
        .entity();
    tree.in_parent(hero_panel, |tree| {
        handles.hero_fill = tree
            .add_node()
            .window(
                Ab(vec2(3.0, 3.0)),
                Ab(vec2(data::BAR_WIDTH - 6.0, data::BAR_HEIGHT - 6.0)),
                Anchor::TopLeft,
            )
            .with_rect(3.0, 0.0, colour(data::CLEAR_EDGE))
            .color_raw::<UiBase>(colour(data::HERO_BAR_COLOR))
            .entity();
    });
    tree.add_node()
        .window(
            Ab(vec2(
                data::BAR_MARGIN,
                data::BAR_MARGIN + data::BAR_HEIGHT + 4.0,
            )),
            Ab(vec2(data::BAR_WIDTH, 20.0)),
            Anchor::TopLeft,
        )
        .with_text(data::HERO_NAME, 15.0)
        .text_left()
        .color_raw::<UiBase>(colour(data::DIM_COLOR));
    handles.weapon = tree
        .add_node()
        .window(
            Ab(vec2(
                data::BAR_MARGIN,
                data::BAR_MARGIN + data::BAR_HEIGHT + 24.0,
            )),
            Ab(vec2(data::BAR_WIDTH, 20.0)),
            Anchor::TopLeft,
        )
        .with_text("", 15.0)
        .text_left()
        .color_raw::<UiBase>(colour(data::GO_COLOR))
        .entity();

    handles.thug_panel = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-data::BAR_MARGIN, data::BAR_MARGIN)),
            Ab(vec2(data::BAR_WIDTH, data::BAR_HEIGHT)),
            Anchor::TopRight,
        )
        .with_rect(4.0, 1.0, colour(data::PANEL_EDGE))
        .color_raw::<UiBase>(colour(data::PANEL_BACK))
        .with_visible(false)
        .entity();
    tree.in_parent(handles.thug_panel, |tree| {
        handles.thug_fill = tree
            .add_node()
            .window(
                Ab(vec2(3.0, 3.0)),
                Ab(vec2(data::BAR_WIDTH - 6.0, data::BAR_HEIGHT - 6.0)),
                Anchor::TopLeft,
            )
            .with_rect(3.0, 0.0, colour(data::CLEAR_EDGE))
            .color_raw::<UiBase>(colour(data::THUG_BAR_COLOR))
            .entity();
    });
    handles.thug_name = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0))
                + Ab(vec2(
                    -data::BAR_MARGIN,
                    data::BAR_MARGIN + data::BAR_HEIGHT + 4.0,
                )),
            Ab(vec2(data::BAR_WIDTH, 20.0)),
            Anchor::TopRight,
        )
        .with_text("", 15.0)
        .text_left()
        .color_raw::<UiBase>(colour(data::DIM_COLOR))
        .entity();

    handles.score = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, data::BAR_MARGIN)),
            Ab(vec2(320.0, 28.0)),
            Anchor::TopCenter,
        )
        .with_text("", data::SCORE_SIZE)
        .text_center()
        .color_raw::<UiBase>(colour(data::GO_COLOR))
        .entity();
    handles.stage = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, data::BAR_MARGIN + 32.0)),
            Ab(vec2(330.0, 24.0)),
            Anchor::TopCenter,
        )
        .with_text("", data::STAGE_SIZE)
        .text_center()
        .color_raw::<UiBase>(colour(data::DIM_COLOR))
        .entity();

    handles.go = tree
        .add_node()
        .window(Rl(vec2(78.0, 46.0)), Ab(vec2(340.0, 110.0)), Anchor::Center)
        .with_text("GO!!", data::GO_SIZE)
        .text_center()
        .color_raw::<UiBase>(colour(data::GO_COLOR))
        .with_visible(false)
        .entity();

    handles.caption = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -42.0)),
            Ab(vec2(920.0, 32.0)),
            Anchor::BottomCenter,
        )
        .with_text("", data::CAPTION_SIZE)
        .text_center()
        .color_raw::<UiBase>(colour(data::TEXT_COLOR))
        .entity();
}

fn menu_style() -> MenuStyle {
    MenuStyle {
        background: colour(data::PANEL_BACK),
        border: colour(data::PANEL_EDGE),
        corner_radius: 8.0,
        border_width: 1.0,
        padding: 22.0,
        spacing: 10.0,
        visible: false,
        ..Default::default()
    }
}

fn colour(value: [f32; 4]) -> Vec4 {
    Vec4::new(value[0], value[1], value[2], value[3])
}
