use crate::queries::{count_text, fill_width, still_reading};
use crate::resources::{Handles, Progress};
use crate::theme;
use nightshade::prelude::*;
use nightshade_ui::components::layout_types::{FlowDirection, WindowLayout};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn build(tree: &mut UiTreeBuilder, handles: &mut Handles) {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    tree.in_parent(root, |tree| {
        handles.card = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(theme::CARD_WIDTH, theme::CARD_HEIGHT)),
                Anchor::Center,
            )
            .with_rect(10.0, 1.0, theme::CARD_BORDER)
            .color_raw::<UiBase>(theme::CARD_BACKGROUND)
            .entity();

        tree.in_parent(handles.card, |tree| {
            tree.add_node()
                .window(Ab(vec2(48.0, 34.0)), Ab(vec2(400.0, 48.0)), Anchor::TopLeft)
                .with_text(theme::HEADING, theme::TITLE_SIZE)
                .text_left()
                .color_raw::<UiBase>(theme::TEXT);

            let track = tree
                .add_node()
                .window(
                    Ab(vec2(48.0, 104.0)),
                    Ab(vec2(theme::BAR_WIDTH, theme::BAR_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_rect(4.0, 1.0, theme::CARD_BORDER)
                .color_raw::<UiBase>(theme::TRACK)
                .entity();

            tree.in_parent(track, |tree| {
                handles.bar_fill = tree
                    .add_node()
                    .window(
                        Ab(vec2(theme::BAR_INSET, theme::BAR_INSET)),
                        Ab(vec2(1.0, theme::BAR_HEIGHT - theme::BAR_INSET * 2.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(3.0, 0.0, theme::FILL)
                    .color_raw::<UiBase>(theme::FILL)
                    .entity();
            });

            handles.count = tree
                .add_node()
                .window(
                    Ab(vec2(48.0, 136.0)),
                    Ab(vec2(theme::BAR_WIDTH, 26.0)),
                    Anchor::TopLeft,
                )
                .with_text("", theme::COUNT_SIZE)
                .text_left()
                .color_raw::<UiBase>(theme::TEXT)
                .entity();

            handles.name = tree
                .add_node()
                .window(
                    Ab(vec2(48.0, 164.0)),
                    Ab(vec2(theme::BAR_WIDTH, 24.0)),
                    Anchor::TopLeft,
                )
                .with_text("", theme::NAME_SIZE)
                .text_left()
                .color_raw::<UiBase>(theme::TEXT_DIM)
                .entity();
        });

        handles.play_root = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::MARGIN)),
                Ab(vec2(theme::BUTTON_WIDTH, theme::BUTTON_HEIGHT)),
                Anchor::BottomCenter,
            )
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        tree.in_parent(handles.play_root, |tree| {
            handles.reload = tree.add_button(theme::RELOAD);
        });
    });
}

pub fn write_the_card(
    progress: &mut Progress,
    handles: &Handles,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    let count = count_text(progress);
    if count != progress.last_count {
        progress.last_count = count.clone();
        ui_set_text(commands, handles.count, &count);
        ui_set_window_layout(
            dirty,
            commands,
            handles.bar_fill,
            WindowLayout {
                position: Ab(vec2(theme::BAR_INSET, theme::BAR_INSET)).into(),
                size: Ab(vec2(
                    fill_width(progress),
                    theme::BAR_HEIGHT - theme::BAR_INSET * 2.0,
                ))
                .into(),
                anchor: Anchor::TopLeft,
            },
        );
    }
    let reading = still_reading(progress);
    if reading != progress.last_reading {
        progress.last_reading = reading.clone();
        ui_set_text(commands, handles.name, &reading);
    }
}

pub fn show_the_picture(
    build: &mut nightshade_ui::commands::builder::UiBuild,
    picture: &nightshade_render::data::decoded_image::DecodedImage,
) -> Entity {
    let mut tree = UiTreeBuilder::new(build.context());
    let (images, runtime) = tree.images_and_runtime();
    let Some(upload) = ui_upload_image(
        images,
        runtime,
        &picture.rgba,
        picture.width,
        picture.height,
    ) else {
        return Entity::default();
    };
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-theme::MARGIN, theme::MARGIN)),
                Ab(vec2(theme::PICTURE_SIDE, theme::PICTURE_SIDE)),
                Anchor::TopRight,
            )
            .with_image_uv(upload.layer, upload.uv_min, upload.uv_max);
    });
    tree.finish();
    root
}
