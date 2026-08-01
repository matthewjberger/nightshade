use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::DataGridColumn;

use crate::prelude::*;
pub struct Ui<'t, 'w, 'c> {
    tree: &'t mut UiTreeBuilder<'w, 'c>,
    parent: nightshade::ecs::Entity,
}

impl<'t, 'w, 'c> Ui<'t, 'w, 'c> {
    fn scoped<R>(&mut self, f: impl FnOnce(&mut UiTreeBuilder<'w, 'c>) -> R) -> R {
        let needs_push = self.tree.current_parent() != self.parent;
        if needs_push {
            self.tree.push_parent(self.parent);
        }
        let result = f(&mut *self.tree);
        if needs_push {
            self.tree.pop_parent();
        }
        result
    }

    fn container(
        &mut self,
        add_fn: impl FnOnce(&mut UiTreeBuilder<'w, 'c>) -> nightshade::ecs::Entity,
        f: impl FnOnce(&mut Ui<'_, 'w, 'c>),
    ) -> nightshade::ecs::Entity {
        let entity = self.scoped(add_fn);
        let content = entity;
        let mut child_ui = Ui {
            tree: &mut *self.tree,
            parent: content,
        };
        f(&mut child_ui);
        entity
    }

    pub fn tree(&mut self) -> &mut UiTreeBuilder<'w, 'c> {
        self.tree
    }

    pub fn set_test_id(&mut self, entity: nightshade::ecs::Entity, test_id: &str) {
        let (accessibility, commands) = self.tree.accessibility_and_commands();
        ui_set_test_id(accessibility, commands, entity, test_id);
    }

    pub fn label(&mut self, text: &str) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_label(text))
    }

    pub fn heading(&mut self, text: &str) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_heading(text))
    }

    pub fn separator(&mut self) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_separator())
    }

    pub fn spacing(&mut self, amount: f32) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_spacing(amount))
    }

    pub fn button(&mut self, text: &str) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_button(text))
    }

    pub fn slider(&mut self, min: f32, max: f32, initial: f32) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_slider(min, max, initial))
    }

    pub fn canvas(&mut self, size: Vec2) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_canvas(size))
    }

    pub fn toggle(&mut self, initial: bool) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_toggle(initial))
    }

    pub fn checkbox(&mut self, label: &str, initial: bool) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_checkbox(label, initial))
    }

    pub fn radio(
        &mut self,
        label: &str,
        group_id: u32,
        option_index: usize,
    ) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_radio(label, group_id, option_index))
    }

    pub fn progress_bar(&mut self, initial: f32) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_progress_bar(initial))
    }

    pub fn label_id(&mut self, text: &str, test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.label(text);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn button_id(&mut self, text: &str, test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.button(text);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn slider_id(
        &mut self,
        min: f32,
        max: f32,
        initial: f32,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.slider(min, max, initial);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn toggle_id(&mut self, initial: bool, test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.toggle(initial);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn checkbox_id(
        &mut self,
        label: &str,
        initial: bool,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.checkbox(label, initial);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn text_input_id(&mut self, placeholder: &str, test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.text_input(placeholder);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn dropdown_id(
        &mut self,
        options: &[&str],
        initial: usize,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.dropdown(options, initial);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn drag_value_id(
        &mut self,
        min: f32,
        max: f32,
        initial: f32,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.drag_value(min, max, initial);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn heading_id(&mut self, text: &str, test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.heading(text);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn progress_bar_id(&mut self, initial: f32, test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.progress_bar(initial);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn radio_id(
        &mut self,
        label: &str,
        group_id: u32,
        option_index: usize,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.radio(label, group_id, option_index);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn tab_bar_id(
        &mut self,
        labels: &[&str],
        initial: usize,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.tab_bar(labels, initial);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn color_picker_id(
        &mut self,
        initial_color: Vec4,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.color_picker(initial_color);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn range_slider_id(
        &mut self,
        min: f32,
        max: f32,
        initial_low: f32,
        initial_high: f32,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.range_slider(min, max, initial_low, initial_high);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn selectable_label_id(
        &mut self,
        text: &str,
        group_id: Option<u32>,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.selectable_label(text, group_id);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn multi_select_id(&mut self, options: &[&str], test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.multi_select(options);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn date_picker_id(
        &mut self,
        year: i32,
        month: u32,
        day: u32,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.date_picker(year, month, day);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn menu_id(
        &mut self,
        label: &str,
        items: &[&str],
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.menu(label, items);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn splitter_id(
        &mut self,
        direction: crate::components::SplitDirection,
        initial_ratio: f32,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.splitter(direction, initial_ratio);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn text_area_id(
        &mut self,
        placeholder: &str,
        rows: usize,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.text_area(placeholder, rows);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn canvas_id(&mut self, size: Vec2, test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.canvas(size);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn data_grid_id(
        &mut self,
        columns: &[DataGridColumn],
        pool_size: usize,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.data_grid(columns, pool_size);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn command_palette_id(
        &mut self,
        pool_size: usize,
        test_id: &str,
    ) -> nightshade::ecs::Entity {
        let entity = self.command_palette(pool_size);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn tile_container_id(&mut self, size: Vec2, test_id: &str) -> nightshade::ecs::Entity {
        let entity = self.tile_container(size);
        self.set_test_id(entity, test_id);
        entity
    }

    pub fn text_input(&mut self, placeholder: &str) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_text_input(placeholder))
    }

    pub fn text_area(&mut self, placeholder: &str, rows: usize) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_text_area(placeholder, rows))
    }

    pub fn dropdown(&mut self, options: &[&str], initial: usize) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_dropdown(options, initial))
    }

    pub fn multi_select(&mut self, options: &[&str]) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_multi_select(options))
    }

    pub fn date_picker(&mut self, year: i32, month: u32, day: u32) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_date_picker(year, month, day))
    }

    pub fn menu(&mut self, label: &str, items: &[&str]) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_menu(label, items))
    }

    pub fn color_picker(&mut self, initial_color: Vec4) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_color_picker(initial_color))
    }

    pub fn splitter(
        &mut self,
        direction: crate::components::SplitDirection,
        initial_ratio: f32,
    ) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_splitter(direction, initial_ratio))
    }

    pub fn range_slider(
        &mut self,
        min: f32,
        max: f32,
        initial_low: f32,
        initial_high: f32,
    ) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_range_slider(min, max, initial_low, initial_high))
    }

    pub fn selectable_label(
        &mut self,
        text: &str,
        group_id: Option<u32>,
    ) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_selectable_label(text, group_id))
    }

    pub fn drag_value(&mut self, min: f32, max: f32, initial: f32) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_drag_value(min, max, initial))
    }

    pub fn tab_bar(&mut self, labels: &[&str], initial: usize) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_tab_bar(labels, initial))
    }

    pub fn spinner(&mut self) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_spinner())
    }

    pub fn breadcrumb(&mut self, segments: &[&str]) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_breadcrumb(segments))
    }

    pub fn virtual_list(&mut self, item_height: f32, pool_size: usize) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_virtual_list(item_height, pool_size))
    }

    pub fn rich_text_editor(&mut self, placeholder: &str, rows: usize) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_rich_text_editor(placeholder, rows))
    }

    pub fn data_grid(
        &mut self,
        columns: &[DataGridColumn],
        pool_size: usize,
    ) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_data_grid(columns, pool_size))
    }

    pub fn command_palette(&mut self, pool_size: usize) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_command_palette(pool_size))
    }

    pub fn property_grid(&mut self, label_width: f32) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_property_grid(label_width))
    }

    pub fn row(&mut self, build: impl FnOnce(&mut Ui<'_, 'w, 'c>)) -> nightshade::ecs::Entity {
        self.container(|tree| tree.add_row(), build)
    }

    pub fn collapsing_header(
        &mut self,
        label: &str,
        default_open: bool,
        f: impl FnOnce(&mut Ui<'_, 'w, 'c>),
    ) -> nightshade::ecs::Entity {
        self.container(|tree| tree.add_collapsing_header(label, default_open), f)
    }

    pub fn scroll_area(
        &mut self,
        size: Vec2,
        f: impl FnOnce(&mut Ui<'_, 'w, 'c>),
    ) -> nightshade::ecs::Entity {
        self.container(|tree| tree.add_scroll_area(size), f)
    }

    pub fn scroll_area_fill(
        &mut self,
        padding: f32,
        spacing: f32,
        f: impl FnOnce(&mut Ui<'_, 'w, 'c>),
    ) -> nightshade::ecs::Entity {
        self.container(|tree| tree.add_scroll_area_fill(padding, spacing), f)
    }

    pub fn modal_dialog(
        &mut self,
        title: &str,
        width: f32,
        height: f32,
        f: impl FnOnce(&mut Ui<'_, 'w, 'c>),
    ) -> nightshade::ecs::Entity {
        self.container(|tree| tree.add_modal_dialog(title, width, height), f)
    }

    pub fn tree_view(
        &mut self,
        multi_select: bool,
        f: impl FnOnce(&mut Ui<'_, 'w, 'c>),
    ) -> nightshade::ecs::Entity {
        self.container(|tree| tree.add_tree_view(multi_select), f)
    }

    pub fn tile_container(&mut self, size: Vec2) -> nightshade::ecs::Entity {
        self.scoped(|tree| tree.add_tile_container(size))
    }
}

impl<'w, 'c> UiTreeBuilder<'w, 'c> {
    pub fn build_ui(
        &mut self,
        parent: nightshade::ecs::Entity,
        build: impl FnOnce(&mut Ui<'_, 'w, 'c>),
    ) {
        let needs_push = self.current_parent() != parent;
        if needs_push {
            self.push_parent(parent);
        }
        let mut ui = Ui { tree: self, parent };
        build(&mut ui);
        if needs_push {
            ui.tree.pop_parent();
        }
    }
}
