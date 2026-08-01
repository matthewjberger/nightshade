use nalgebra_glm::vec2;

use crate::commands::builder::UiTreeBuilder;
use crate::data::state::UiBase;
use crate::data::stats::StatPanel;
use crate::data::units::Ab;
use nightshade_paint::data::Anchor;

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_stat_header(
        &mut self,
        style: &StatPanel,
        y: f32,
        title: &str,
    ) -> nightshade::ecs::Entity {
        self.add_node()
            .window(
                Ab(vec2(style.margin, y)),
                Ab(vec2(style.width - style.margin * 2.0, 18.0)),
                Anchor::TopLeft,
            )
            .with_text(title, style.header_size)
            .text_left()
            .color_raw::<UiBase>(style.header_color)
            .entity()
    }

    pub fn add_stat_row(
        &mut self,
        style: &StatPanel,
        y: f32,
        label: &str,
        value: &str,
    ) -> nightshade::ecs::Entity {
        self.add_node()
            .window(
                Ab(vec2(style.margin, y)),
                Ab(vec2(style.label_width, style.row_height)),
                Anchor::TopLeft,
            )
            .with_text(label, style.font_size)
            .text_left()
            .color_raw::<UiBase>(style.label_color)
            .entity();

        self.add_node()
            .window(
                Ab(vec2(style.margin + style.label_width, y)),
                Ab(vec2(
                    style.width - style.margin * 2.0 - style.label_width,
                    style.row_height,
                )),
                Anchor::TopLeft,
            )
            .with_text(value, style.font_size)
            .text_left()
            .color_raw::<UiBase>(style.value_color)
            .entity()
    }
}
