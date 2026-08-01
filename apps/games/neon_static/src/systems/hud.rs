use crate::data;
use crate::resources::{FloorPhase, NeonStaticResources, Overlay};
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

pub fn build_overlay(
    mut resources: ResMut<NeonStaticResources>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let overlay = tree.in_parent(root, |tree| {
        let mut overlay = Overlay {
            flash: tree
                .add_node()
                .absolute_fill(0.0)
                .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<UiBase>(Vec4::new(
                    data::FLASH_COLOR.x,
                    data::FLASH_COLOR.y,
                    data::FLASH_COLOR.z,
                    0.0,
                ))
                .without_pointer_events()
                .entity(),
            ..Default::default()
        };

        let panel = tree
            .add_node()
            .window(
                Ab(vec2(data::MARGIN, data::MARGIN)),
                Ab(vec2(data::READOUT_WIDTH, data::READOUT_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(8.0, 1.0, data::PANEL_BORDER)
            .color_raw::<UiBase>(data::PANEL_BACKGROUND)
            .without_pointer_events()
            .entity();

        tree.in_parent(panel, |tree| {
            overlay.readout = tree
                .add_node()
                .window(
                    Ab(vec2(14.0, 12.0)),
                    Ab(vec2(data::READOUT_WIDTH - 28.0, 30.0)),
                    Anchor::TopLeft,
                )
                .with_text("", data::READOUT_TEXT_SIZE)
                .text_left()
                .color_raw::<UiBase>(data::TEXT_PRIMARY)
                .without_pointer_events()
                .entity();
        });

        overlay.banner = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)),
                Ab(vec2(data::BANNER_WIDTH, data::BANNER_HEIGHT)),
                Anchor::Center,
            )
            .with_text("", data::BANNER_TEXT_SIZE)
            .text_center()
            .with_text_outline(Vec4::new(0.0, 0.0, 0.0, 1.0), 2.0)
            .color_raw::<UiBase>(data::TEXT_CLEARED)
            .without_pointer_events()
            .entity();

        overlay
    });

    tree.finish();
    resources.overlay = overlay;
}

pub fn show(mut resources: ResMut<NeonStaticResources>, mut commands: Commands) {
    let resources = &mut *resources;
    if resources.overlay.readout == Entity::default() {
        return;
    }

    let readout = format!(
        "LEFT {}   DOWN {}   TIME {:.1}",
        resources.floor.remaining, resources.floor.killed, resources.floor.elapsed
    );
    if readout != resources.readout {
        ui_set_text(&mut commands, resources.overlay.readout, &readout);
        resources.readout = readout;
    }

    let banner = match resources.floor.phase {
        FloorPhase::Fighting => String::new(),
        FloorPhase::Cleared => format!("FLOOR CLEAR   {:.1}s", resources.floor.elapsed),
        FloorPhase::Dead => "YOU DIED".to_string(),
    };
    if banner != resources.banner {
        let color = match resources.floor.phase {
            FloorPhase::Dead => data::TEXT_DEAD,
            _ => data::TEXT_CLEARED,
        };
        ui_set_text(&mut commands, resources.overlay.banner, &banner);
        ui_set_color::<UiBase>(&mut commands, resources.overlay.banner, color);
        resources.banner = banner;
    }

    let step = (resources.floor.flash * 64.0).round() as u32;
    if step != resources.last_flash {
        resources.last_flash = step;
        ui_set_color::<UiBase>(
            &mut commands,
            resources.overlay.flash,
            Vec4::new(
                data::FLASH_COLOR.x,
                data::FLASH_COLOR.y,
                data::FLASH_COLOR.z,
                step as f32 / 64.0 * 0.55,
            ),
        );
    }
}
