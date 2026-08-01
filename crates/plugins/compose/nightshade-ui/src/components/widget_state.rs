use nalgebra_glm;
use std::collections::{HashMap, HashSet};

use nalgebra_glm::{Vec2, Vec4};

use nightshade_paint::data::Rect;

use super::data_types::{
    CanvasCommand, CharStyle, ColorPickerMode, CommandEntry, ContextMenuItem, ContextMenuItemDef,
    DataGridColumn, DataGridPoolRow, InputMask, RangeSliderThumb, ResizeEdge, RichTextSnapshot,
    SplitDirection, TextSnapshot, TextSpan, UiPanelKind, UndoStack, VirtualListPoolItem,
};

#[derive(Clone, Debug, Default)]
pub struct UiButtonData {
    pub clicked: bool,
    pub text_slot: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct UiSliderData {
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub changed: bool,
    pub fill_entity: nightshade::ecs::Entity,
    pub text_slot: usize,
    pub logarithmic: bool,
    pub precision: usize,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Clone, Debug, Default)]
pub struct UiRangeSliderData {
    pub min: f32,
    pub max: f32,
    pub low_value: f32,
    pub high_value: f32,
    pub changed: bool,
    pub fill_entity: nightshade::ecs::Entity,
    pub low_thumb_entity: nightshade::ecs::Entity,
    pub high_thumb_entity: nightshade::ecs::Entity,
    pub text_slot: usize,
    pub active_thumb: Option<RangeSliderThumb>,
    pub precision: usize,
    pub thumb_half_size: f32,
}

#[derive(Clone, Debug, Default)]
pub struct UiToggleData {
    pub value: bool,
    pub changed: bool,
    pub animated_position: f32,
    pub knob_entity: nightshade::ecs::Entity,
}

#[derive(Clone, Debug, Default)]
pub struct UiCheckboxData {
    pub value: bool,
    pub changed: bool,
    pub inner_entity: nightshade::ecs::Entity,
}

#[derive(Clone, Debug, Default)]
pub struct UiRadioData {
    pub group_id: u32,
    pub option_index: usize,
    pub selected: bool,
    pub changed: bool,
    pub inner_entity: nightshade::ecs::Entity,
}

#[derive(Clone, Debug, Default)]
pub struct UiProgressBarData {
    pub value: f32,
    pub fill_entity: nightshade::ecs::Entity,
}

#[derive(Clone, Debug, Default)]
pub struct UiCollapsingHeaderData {
    pub open: bool,
    pub changed: bool,
    pub content_entity: nightshade::ecs::Entity,
    pub arrow_text_slot: usize,
}

#[derive(Clone, Debug, Default)]
pub struct UiScrollAreaData {
    pub scroll_offset: f32,
    pub content_entity: nightshade::ecs::Entity,
    pub content_height: f32,
    pub visible_height: f32,
    pub thumb_entity: nightshade::ecs::Entity,
    pub track_entity: nightshade::ecs::Entity,
    pub thumb_dragging: bool,
    pub thumb_drag_start_offset: f32,
    pub snap_interval: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct UiTabBarData {
    pub selected_tab: usize,
    pub changed: bool,
    pub tab_entities: Vec<nightshade::ecs::Entity>,
    pub tab_text_slots: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct UiBreadcrumbData {
    pub segments: Vec<String>,
    pub clicked_segment: Option<usize>,
    pub changed: bool,
    pub segment_entities: Vec<nightshade::ecs::Entity>,
    pub segment_text_slots: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct UiSplitterData {
    pub direction: SplitDirection,
    pub ratio: f32,
    pub changed: bool,
    pub first_pane: nightshade::ecs::Entity,
    pub second_pane: nightshade::ecs::Entity,
    pub divider_entity: nightshade::ecs::Entity,
    pub min_ratio: f32,
    pub max_ratio: f32,
}

#[derive(Clone, Debug, Default)]
pub struct UiSpinnerData {
    pub dots: Vec<nightshade::ecs::Entity>,
    pub rotation_speed: f32,
    pub phase: f32,
}

#[derive(Clone, Debug, Default)]
pub struct UiTextInputData {
    pub text: String,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub changed: bool,
    pub text_slot: usize,
    pub cursor_entity: nightshade::ecs::Entity,
    pub selection_entity: nightshade::ecs::Entity,
    pub scroll_offset: f32,
    pub cursor_blink_timer: f64,
    pub placeholder_entity: Option<nightshade::ecs::Entity>,
    pub undo_stack: std::sync::Arc<UndoStack<TextSnapshot>>,
    pub input_mask: InputMask,
    pub max_length: Option<usize>,
    pub password: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UiDropdownData {
    pub options: Vec<String>,
    pub selected_index: usize,
    pub changed: bool,
    pub open: bool,
    pub header_text_slot: usize,
    pub popup_entities: Vec<nightshade::ecs::Entity>,
    pub popup_container_entity: nightshade::ecs::Entity,
    pub hovered_index: Option<usize>,
    pub is_theme_dropdown: bool,
    pub searchable: bool,
    pub filter_text: String,
    pub filter_input_entity: Option<nightshade::ecs::Entity>,
    pub filtered_indices: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct UiMultiSelectData {
    pub options: Vec<String>,
    pub selected_indices: HashSet<usize>,
    pub changed: bool,
    pub open: bool,
    pub header_text_slot: usize,
    pub popup_entities: Vec<nightshade::ecs::Entity>,
    pub popup_container_entity: nightshade::ecs::Entity,
    pub hovered_index: Option<usize>,
    pub check_entities: Vec<nightshade::ecs::Entity>,
}

#[derive(Clone, Debug, Default)]
pub struct UiDatePickerData {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub changed: bool,
    pub open: bool,
    pub header_text_slot: usize,
    pub popup_entity: nightshade::ecs::Entity,
    pub day_entities: Vec<nightshade::ecs::Entity>,
    pub day_text_slots: Vec<usize>,
    pub month_label_slot: usize,
    pub prev_month_entity: nightshade::ecs::Entity,
    pub next_month_entity: nightshade::ecs::Entity,
    pub selected_day_entity: Option<nightshade::ecs::Entity>,
}

#[derive(Clone, Debug, Default)]
pub struct UiMenuData {
    pub items: Vec<String>,
    pub clicked_item: Option<usize>,
    pub open: bool,
    pub label_text_slot: usize,
    pub popup_entities: Vec<nightshade::ecs::Entity>,
    pub popup_container_entity: nightshade::ecs::Entity,
}

#[derive(Clone, Debug, Default)]
pub struct UiPanelData {
    pub id: String,
    pub title: String,
    pub title_text_slot: usize,
    pub content_entity: nightshade::ecs::Entity,
    pub header_entity: nightshade::ecs::Entity,
    pub collapsed: bool,
    pub panel_kind: UiPanelKind,
    pub focus_order: i32,
    pub pinned: bool,
    pub min_size: Vec2,
    pub drag_offset: Option<Vec2>,
    pub resize_edge: Option<ResizeEdge>,
    pub resize_start_rect: Option<Rect>,
    pub resize_start_mouse: Option<Vec2>,
    pub undocked_rect: Option<Rect>,
    pub default_dock_size: f32,
    pub collapse_button_entity: Option<nightshade::ecs::Entity>,
    pub collapse_button_text_slot: Option<usize>,
    pub header_visible: bool,
    pub resizable: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UiColorPickerData {
    pub color: Vec4,
    pub changed: bool,
    pub swatch_entity: nightshade::ecs::Entity,
    pub slider_entities: [nightshade::ecs::Entity; 4],
    pub mode: ColorPickerMode,
    pub popup_entity: nightshade::ecs::Entity,
    pub popup_wheel_entity: nightshade::ecs::Entity,
    pub popup_open: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UiColorWheelData {
    pub color: Vec4,
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub alpha: f32,
    pub changed: bool,
    pub wheel_entity: nightshade::ecs::Entity,
    pub dot_entity: nightshade::ecs::Entity,
    pub swatch_entity: nightshade::ecs::Entity,
    pub value_slider_entity: nightshade::ecs::Entity,
    pub alpha_slider_entity: nightshade::ecs::Entity,
}

#[derive(Clone, Debug, Default)]
pub struct UiSelectableLabelData {
    pub selected: bool,
    pub changed: bool,
    pub text_slot: usize,
    pub group_id: Option<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct UiDragValueData {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub speed: f32,
    pub precision: usize,
    pub prefix: String,
    pub suffix: String,
    pub changed: bool,
    pub text_slot: usize,
    pub editing: bool,
    pub cursor_entity: nightshade::ecs::Entity,
    pub selection_entity: nightshade::ecs::Entity,
    pub edit_text: String,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub cursor_blink_timer: f64,
    pub scroll_offset: f32,
    pub drag_start_value: f32,
    pub undo_stack: UndoStack<TextSnapshot>,
    pub up_entity: Option<nightshade::ecs::Entity>,
    pub down_entity: Option<nightshade::ecs::Entity>,
    pub step: f32,
}

#[derive(Clone, Debug, Default)]
pub struct UiContextMenuData {
    pub items: Vec<ContextMenuItem>,
    pub open: bool,
    pub clicked_item: Option<usize>,
    pub popup_entity: nightshade::ecs::Entity,
    pub item_entities: Vec<Option<nightshade::ecs::Entity>>,
    pub item_defs: Vec<ContextMenuItemDef>,
}

#[derive(Clone, Debug, Default)]
pub struct UiTreeNodeData {
    pub label: String,
    pub text_slot: usize,
    pub depth: usize,
    pub expanded: bool,
    pub selected: bool,
    pub row_entity: nightshade::ecs::Entity,
    pub arrow_entity: nightshade::ecs::Entity,
    pub arrow_text_slot: usize,
    pub children_container: nightshade::ecs::Entity,
    pub user_data: u64,
    pub parent_node: Option<nightshade::ecs::Entity>,
    pub wrapper_entity: nightshade::ecs::Entity,
    pub lazy: bool,
    pub lazy_loaded: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UiTreeViewData {
    pub selected_nodes: Vec<nightshade::ecs::Entity>,
    pub multi_select: bool,
    pub node_entities: Vec<nightshade::ecs::Entity>,
    pub changed: bool,
    pub content_entity: nightshade::ecs::Entity,
    pub context_menu_node: Option<nightshade::ecs::Entity>,
    pub filter_text: String,
    pub filter_active: bool,
    pub pre_filter_expanded: HashMap<nightshade::ecs::Entity, bool>,
    pub scroll_offset: f32,
    pub thumb_entity: nightshade::ecs::Entity,
    pub track_entity: nightshade::ecs::Entity,
    pub thumb_dragging: bool,
    pub thumb_drag_start_offset: f32,
}

#[derive(Clone, Debug, Default)]
pub struct UiModalDialogData {
    pub title_text_slot: usize,
    pub content_entity: nightshade::ecs::Entity,
    pub backdrop_entity: nightshade::ecs::Entity,
    pub ok_button: Option<nightshade::ecs::Entity>,
    pub cancel_button: Option<nightshade::ecs::Entity>,
    pub result: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct UiRichTextData {
    pub span_entities: Vec<nightshade::ecs::Entity>,
    pub span_text_slots: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct UiDataGridData {
    pub columns: Vec<DataGridColumn>,
    pub row_height: f32,
    pub pool_size: usize,
    pub total_rows: usize,
    pub scroll_entity: nightshade::ecs::Entity,
    pub body_entity: nightshade::ecs::Entity,
    pub header_entities: Vec<nightshade::ecs::Entity>,
    pub header_text_slots: Vec<usize>,
    pub top_spacer: nightshade::ecs::Entity,
    pub bottom_spacer: nightshade::ecs::Entity,
    pub pool_rows: Vec<DataGridPoolRow>,
    pub visible_start: usize,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
    pub sort_changed: bool,
    pub selected_rows: HashSet<usize>,
    pub selection_anchor: Option<usize>,
    pub selection_changed: bool,
    pub focused: bool,
    pub resize_column: Option<usize>,
    pub resize_start_x: f32,
    pub resize_start_width: f32,
    pub filtered_indices: Option<Vec<usize>>,
    pub header_divider_entities: Vec<nightshade::ecs::Entity>,
    pub filter_row_entity: Option<nightshade::ecs::Entity>,
    pub filter_input_entities: Vec<nightshade::ecs::Entity>,
    pub filter_texts: Vec<String>,
    pub editing_cell: Option<(usize, usize)>,
    pub editing_input_entity: Option<nightshade::ecs::Entity>,
}

#[derive(Clone, Debug, Default)]
pub struct UiPropertyGridData {
    pub label_width: f32,
    pub row_entities: Vec<nightshade::ecs::Entity>,
    pub label_entities: Vec<nightshade::ecs::Entity>,
    pub resize_active: bool,
    pub resize_start_x: f32,
    pub resize_start_width: f32,
}

#[derive(Clone, Debug, Default)]
pub struct UiCommandPaletteData {
    pub commands: Vec<CommandEntry>,
    pub filter_text: String,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
    pub open: bool,
    pub executed_command: Option<usize>,
    pub text_input_entity: nightshade::ecs::Entity,
    pub result_entities: Vec<nightshade::ecs::Entity>,
    pub result_text_slots: Vec<usize>,
    pub backdrop_entity: nightshade::ecs::Entity,
    pub dialog_entity: nightshade::ecs::Entity,
    pub pool_size: usize,
    pub scroll_entity: nightshade::ecs::Entity,
}

#[derive(Clone, Debug, Default)]
pub struct UiVirtualListData {
    pub item_height: f32,
    pub pool_size: usize,
    pub total_items: usize,
    pub visible_start: usize,
    pub scroll_entity: nightshade::ecs::Entity,
    pub body_entity: nightshade::ecs::Entity,
    pub top_spacer: nightshade::ecs::Entity,
    pub bottom_spacer: nightshade::ecs::Entity,
    pub pool_items: Vec<VirtualListPoolItem>,
    pub selection: Option<usize>,
    pub selection_changed: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UiCanvasData {
    pub commands: Vec<CanvasCommand>,
    pub hit_test_enabled: bool,
    pub command_bounds: Vec<(u32, nightshade_paint::data::Rect)>,
}

#[derive(Clone, Debug, Default)]
pub struct UiTextAreaData {
    pub text: String,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub changed: bool,
    pub text_slot: usize,
    pub cursor_entity: nightshade::ecs::Entity,
    pub selection_pool: Vec<nightshade::ecs::Entity>,
    pub scroll_offset_y: f32,
    pub cursor_blink_timer: f64,
    pub placeholder_entity: Option<nightshade::ecs::Entity>,
    pub line_height: f32,
    pub visible_rows: usize,
    pub undo_stack: UndoStack<TextSnapshot>,
    pub max_length: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct UiRichTextEditorData {
    pub text: String,
    pub char_styles: Vec<CharStyle>,
    pub current_style: CharStyle,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub changed: bool,
    pub text_slot: usize,
    pub cursor_entity: nightshade::ecs::Entity,
    pub selection_pool: Vec<nightshade::ecs::Entity>,
    pub scroll_offset_y: f32,
    pub cursor_blink_timer: f64,
    pub line_height: f32,
    pub visible_rows: usize,
    pub placeholder_entity: Option<nightshade::ecs::Entity>,
    pub undo_stack: UndoStack<RichTextSnapshot>,
}

impl UiRichTextEditorData {
    pub fn spans(&self) -> Vec<TextSpan> {
        if self.text.is_empty() {
            return Vec::new();
        }
        let chars: Vec<char> = self.text.chars().collect();
        let mut spans = Vec::new();
        let mut run_start = 0;
        let default_style = CharStyle::default();
        for index in 1..=chars.len() {
            let current_style = self.char_styles.get(index).unwrap_or(&default_style);
            let prev_style = self.char_styles.get(run_start).unwrap_or(&default_style);
            if index == chars.len() || current_style != prev_style {
                let run_text: String = chars[run_start..index].iter().collect();
                let style = self.char_styles.get(run_start).cloned().unwrap_or_default();
                let mut span = TextSpan::new(&run_text);
                span.bold = style.bold;
                span.italic = style.italic;
                span.underline = style.underline;
                span.color = style.color;
                spans.push(span);
                run_start = index;
            }
        }
        spans
    }
}

pub trait WidgetData: Sized {
    fn get(storage: &nightshade::ecs::Storage, entity: nightshade::ecs::Entity) -> Option<&Self>;
    fn get_mut(
        storage: &mut nightshade::ecs::Storage,
        entity: nightshade::ecs::Entity,
    ) -> Option<&mut Self>;
    fn set(storage: &mut nightshade::ecs::Storage, entity: nightshade::ecs::Entity, data: Self);
}

macro_rules! impl_widget_data {
    ($data:ty) => {
        impl WidgetData for $data {
            fn get(
                storage: &nightshade::ecs::Storage,
                entity: nightshade::ecs::Entity,
            ) -> Option<&Self> {
                storage.get::<Self>(entity)
            }
            fn get_mut(
                storage: &mut nightshade::ecs::Storage,
                entity: nightshade::ecs::Entity,
            ) -> Option<&mut Self> {
                storage.get_mut::<Self>(entity)
            }
            fn set(
                storage: &mut nightshade::ecs::Storage,
                entity: nightshade::ecs::Entity,
                data: Self,
            ) {
                storage.set(entity, data);
            }
        }
    };
}

impl_widget_data!(UiButtonData);
impl_widget_data!(UiSliderData);
impl_widget_data!(UiToggleData);
impl_widget_data!(UiCheckboxData);
impl_widget_data!(UiRadioData);
impl_widget_data!(UiProgressBarData);
impl_widget_data!(UiCollapsingHeaderData);
impl_widget_data!(UiScrollAreaData);
impl_widget_data!(UiTabBarData);
impl_widget_data!(UiTextInputData);
impl_widget_data!(UiDropdownData);
impl_widget_data!(UiMenuData);
impl_widget_data!(UiPanelData);
impl_widget_data!(UiColorPickerData);
impl_widget_data!(UiColorWheelData);
impl_widget_data!(UiSelectableLabelData);
impl_widget_data!(UiDragValueData);
impl_widget_data!(UiContextMenuData);
impl_widget_data!(UiTreeViewData);
impl_widget_data!(UiTreeNodeData);
impl_widget_data!(UiModalDialogData);
impl_widget_data!(UiRichTextData);
impl_widget_data!(UiDataGridData);
impl_widget_data!(UiPropertyGridData);
impl_widget_data!(UiCommandPaletteData);
impl_widget_data!(UiCanvasData);
impl_widget_data!(UiTextAreaData);
impl_widget_data!(super::tiles::UiTileContainerData);
impl_widget_data!(UiVirtualListData);
impl_widget_data!(UiRichTextEditorData);
impl_widget_data!(UiRangeSliderData);
impl_widget_data!(UiBreadcrumbData);
impl_widget_data!(UiSplitterData);
impl_widget_data!(UiMultiSelectData);
impl_widget_data!(UiDatePickerData);
impl_widget_data!(UiSpinnerData);
