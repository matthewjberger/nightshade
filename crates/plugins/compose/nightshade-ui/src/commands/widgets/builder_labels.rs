use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::{UiNodeBuilder, UiTreeBuilder};
use crate::components::layout_types::FlowDirection;
use crate::components::*;
use crate::data::state::UiBase;
use crate::data::units::{Ab, Rl};
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_label(&mut self, text: &str) -> nightshade::ecs::Entity {
        let font_size = self.active_theme().font_size;
        let text_slot = self.text_state_mut().cache.add_text(text);

        self.add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)))
            .with_text_slot(text_slot, font_size)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::Text)
            .entity()
    }

    pub fn add_heading(&mut self, text: &str) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size * 1.4;

        let text_slot = self.text_state_mut().cache.add_text(text);

        self.add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)))
            .with_text_slot(text_slot, font_size)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::TextAccent)
            .entity()
    }

    pub fn label_node(&mut self, text: &str, font_size: f32) -> UiNodeBuilder<'_, 'a, 'c> {
        let text_slot = self.text_state_mut().cache.add_text(text);
        self.add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)))
            .with_text_slot(text_slot, font_size)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
    }

    pub fn separator_node(&mut self, thickness: f32) -> UiNodeBuilder<'_, 'a, 'c> {
        self.add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, thickness)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
    }

    pub fn card_node(
        &mut self,
        corner_radius: f32,
        border_width: f32,
        border: Vec4,
    ) -> UiNodeBuilder<'_, 'a, 'c> {
        self.add_node()
            .with_rect(corner_radius, border_width, border)
    }

    pub fn add_separator(&mut self) -> nightshade::ecs::Entity {
        self.add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 1.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::Border)
            .entity()
    }

    pub fn add_spacing(&mut self, amount: f32) -> nightshade::ecs::Entity {
        self.add_node()
            .flow_child(Ab(Vec2::new(amount, amount)))
            .entity()
    }

    pub fn add_row(&mut self) -> nightshade::ecs::Entity {
        self.add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Horizontal, 0.0, 4.0)
            .entity()
    }
}
