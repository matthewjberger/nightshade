mod builder_buttons;
mod builder_containers;
mod builder_context_menu;
mod builder_controls;
mod builder_data;
mod builder_dialogs;
mod builder_drag_value;
mod builder_labels;
mod builder_navigation;
mod builder_panels;
mod builder_stats;
mod builder_text;
mod builder_tree;
pub(crate) mod date_utils;
pub mod world_canvas;
pub mod world_data;
pub mod world_interaction;
pub mod world_layout;
pub mod world_state;
pub mod world_tiles;

pub(crate) use date_utils::{
    day_of_week, days_in_month, format_month_year, populate_calendar_grid,
};
pub(crate) use world_state::masked_input_display;

pub(super) enum ContextMenuBuilderEntry {
    Item {
        label: String,
        shortcut: String,
        tag: u32,
    },
    Checkable {
        label: String,
        shortcut: String,
        checked: bool,
        tag: u32,
    },
    Separator,
    Submenu {
        label: String,
        children: Vec<ContextMenuBuilderEntry>,
    },
    WidgetRow {
        label: String,
    },
}

pub struct ContextMenuBuilder {
    pub(super) entries: Vec<ContextMenuBuilderEntry>,
}

impl ContextMenuBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn item(mut self, label: &str, shortcut: &str) -> Self {
        self.entries.push(ContextMenuBuilderEntry::Item {
            label: label.to_string(),
            shortcut: shortcut.to_string(),
            tag: 0,
        });
        self
    }

    pub fn item_tagged(mut self, label: &str, shortcut: &str, tag: u32) -> Self {
        self.entries.push(ContextMenuBuilderEntry::Item {
            label: label.to_string(),
            shortcut: shortcut.to_string(),
            tag,
        });
        self
    }

    pub fn checkable(mut self, label: &str, shortcut: &str, checked: bool) -> Self {
        self.entries.push(ContextMenuBuilderEntry::Checkable {
            label: label.to_string(),
            shortcut: shortcut.to_string(),
            checked,
            tag: 0,
        });
        self
    }

    pub fn checkable_tagged(
        mut self,
        label: &str,
        shortcut: &str,
        checked: bool,
        tag: u32,
    ) -> Self {
        self.entries.push(ContextMenuBuilderEntry::Checkable {
            label: label.to_string(),
            shortcut: shortcut.to_string(),
            checked,
            tag,
        });
        self
    }

    pub fn separator(mut self) -> Self {
        self.entries.push(ContextMenuBuilderEntry::Separator);
        self
    }

    pub fn submenu(
        mut self,
        label: &str,
        f: impl FnOnce(ContextMenuBuilder) -> ContextMenuBuilder,
    ) -> Self {
        let sub = f(ContextMenuBuilder::new());
        self.entries.push(ContextMenuBuilderEntry::Submenu {
            label: label.to_string(),
            children: sub.entries,
        });
        self
    }

    pub fn widget_row(mut self, label: &str) -> Self {
        self.entries.push(ContextMenuBuilderEntry::WidgetRow {
            label: label.to_string(),
        });
        self
    }
}

impl Default for ContextMenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) struct ContextMenuTheme {
    pub(super) font_size: f32,
    pub(super) corner_radius: f32,
    pub(super) button_height: f32,
}
