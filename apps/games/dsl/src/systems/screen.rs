use crate::data;
use crate::resources::{Screen, Store};
use nightshade::prelude::*;
use nightshade_text::contract::{TextAlignment, VerticalAlignment};
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

pub fn paint(store: Res<Store>, mut screen: ResMut<Screen>, mut build: UiBuild) {
    if store.signs == screen.shown && store.panels == screen.plates {
        return;
    }
    let wanted = store.signs.clone();
    let plates = store.panels.clone();
    let mut tree = UiTreeBuilder::new(build.context());
    if let Some(root) = screen.root.take() {
        tree.commands().despawn_recursive(root);
    }
    let root = tree
        .add_node()
        .window(
            Ab(Vec2::new(0.0, 0.0)),
            Rl(Vec2::new(100.0, 100.0)),
            Anchor::TopLeft,
        )
        .without_pointer_events()
        .entity();
    tree.in_parent(root, |tree| {
        for plate in &plates {
            let face = tree
                .add_node()
                .window(
                    Rl(Vec2::new(plate.anchor[0] * 100.0, plate.anchor[1] * 100.0)),
                    Rl(Vec2::new(plate.size[0] * 100.0, plate.size[1] * 100.0)),
                    Anchor::Center,
                )
                .with_rect(6.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<UiBase>(Vec4::new(
                    plate.tone[0],
                    plate.tone[1],
                    plate.tone[2],
                    plate.tone[3],
                ))
                .without_pointer_events()
                .entity();
            if plate.line.is_empty() {
                continue;
            }
            tree.in_parent(face, |tree| {
                tree.add_node()
                    .window(
                        Rl(Vec2::new(50.0, 50.0)),
                        Rl(Vec2::new(100.0, 100.0)),
                        Anchor::Center,
                    )
                    .with_text(&plate.line, plate.letters)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(Vec4::new(
                        plate.ink[0],
                        plate.ink[1],
                        plate.ink[2],
                        plate.ink[3],
                    ))
                    .without_pointer_events();
            });
        }
        for sign in &wanted {
            let wide = sign.line.chars().count() as f32 * sign.size;
            let (corner, line) = match sign.justify {
                0 => (Anchor::CenterLeft, TextAlignment::Left),
                2 => (Anchor::CenterRight, TextAlignment::Right),
                _ => (Anchor::Center, TextAlignment::Center),
            };
            tree.add_node()
                .window(
                    Rl(Vec2::new(sign.anchor[0] * 100.0, sign.anchor[1] * 100.0)),
                    Ab(Vec2::new(wide, sign.size * data::SIGN_LINE)),
                    corner,
                )
                .with_text(&sign.line, sign.size)
                .with_text_alignment(line, VerticalAlignment::Middle)
                .color_raw::<UiBase>(Vec4::new(
                    sign.tone[0],
                    sign.tone[1],
                    sign.tone[2],
                    sign.tone[3],
                ))
                .without_pointer_events();
        }
    });
    tree.runtime_mut().visible = true;
    tree.finish();
    screen.root = Some(root);
    screen.shown = wanted;
    screen.plates = plates;
}
