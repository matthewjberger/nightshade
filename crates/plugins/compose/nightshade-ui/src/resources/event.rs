use crate::components::{DragPayload, TileId};
use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

#[derive(Clone, Debug)]
pub enum UiEvent {
    ButtonClicked(nightshade::ecs::Entity),
    SliderChanged {
        entity: nightshade::ecs::Entity,
        value: f32,
    },
    ToggleChanged {
        entity: nightshade::ecs::Entity,
        value: bool,
    },
    CheckboxChanged {
        entity: nightshade::ecs::Entity,
        value: bool,
    },
    RadioChanged {
        entity: nightshade::ecs::Entity,
        group_id: u32,
        option_index: usize,
    },
    TabChanged {
        entity: nightshade::ecs::Entity,
        tab_index: usize,
    },
    TextInputChanged {
        entity: nightshade::ecs::Entity,
        text: String,
    },
    TextInputSubmitted {
        entity: nightshade::ecs::Entity,
        text: String,
    },
    DropdownChanged {
        entity: nightshade::ecs::Entity,
        selected_index: usize,
    },
    MenuItemClicked {
        entity: nightshade::ecs::Entity,
        item_index: usize,
    },
    ColorPickerChanged {
        entity: nightshade::ecs::Entity,
        color: Vec4,
    },
    SelectableLabelClicked {
        entity: nightshade::ecs::Entity,
        selected: bool,
    },
    DragValueChanged {
        entity: nightshade::ecs::Entity,
        value: f32,
    },
    ContextMenuItemClicked {
        entity: nightshade::ecs::Entity,
        item_index: usize,
        tag: u32,
    },
    TreeNodeSelected {
        tree: nightshade::ecs::Entity,
        node: nightshade::ecs::Entity,
        selected: bool,
    },
    TreeNodeToggled {
        tree: nightshade::ecs::Entity,
        node: nightshade::ecs::Entity,
        expanded: bool,
    },
    TreeNodeContextMenu {
        tree: nightshade::ecs::Entity,
        node: nightshade::ecs::Entity,
        position: Vec2,
    },
    TreeNodeExpandRequested {
        tree: nightshade::ecs::Entity,
        node: nightshade::ecs::Entity,
        user_data: u64,
    },
    ModalClosed {
        entity: nightshade::ecs::Entity,
        confirmed: bool,
    },
    CommandPaletteExecuted {
        entity: nightshade::ecs::Entity,
        command_index: usize,
    },
    TileTabActivated {
        container: nightshade::ecs::Entity,
        pane_id: TileId,
    },
    TileTabClosed {
        container: nightshade::ecs::Entity,
        pane_id: TileId,
        title: String,
    },
    TileSplitterMoved {
        container: nightshade::ecs::Entity,
        split_id: TileId,
        ratio: f32,
    },
    DragStarted {
        source: nightshade::ecs::Entity,
        payload: DragPayload,
    },
    DragDropped {
        source: nightshade::ecs::Entity,
        target: nightshade::ecs::Entity,
        payload: DragPayload,
    },
    DragCancelled {
        source: nightshade::ecs::Entity,
    },
    DragEnter {
        target: nightshade::ecs::Entity,
        source: nightshade::ecs::Entity,
        payload: DragPayload,
    },
    DragOver {
        target: nightshade::ecs::Entity,
        source: nightshade::ecs::Entity,
        position: Vec2,
    },
    DragLeave {
        target: nightshade::ecs::Entity,
        source: nightshade::ecs::Entity,
    },
    ShortcutTriggered {
        command_index: usize,
    },
    CanvasClicked {
        entity: nightshade::ecs::Entity,
        command_id: u32,
        position: Vec2,
    },
    VirtualListItemClicked {
        entity: nightshade::ecs::Entity,
        item_index: usize,
    },
    VirtualListItemRightClicked {
        entity: nightshade::ecs::Entity,
        item_index: usize,
        screen_position: Vec2,
    },
    TextAreaChanged {
        entity: nightshade::ecs::Entity,
        text: String,
    },
    RichTextEditorChanged {
        entity: nightshade::ecs::Entity,
        text: String,
    },
    DataGridFilterChanged {
        entity: nightshade::ecs::Entity,
        filters: Vec<String>,
    },
    RangeSliderChanged {
        entity: nightshade::ecs::Entity,
        low: f32,
        high: f32,
    },
    DataGridCellEdited {
        entity: nightshade::ecs::Entity,
        row: usize,
        column: usize,
        text: String,
    },
    BreadcrumbClicked {
        entity: nightshade::ecs::Entity,
        segment_index: usize,
    },
    SplitterChanged {
        entity: nightshade::ecs::Entity,
        ratio: f32,
    },
    MultiSelectChanged {
        entity: nightshade::ecs::Entity,
        selected_indices: Vec<usize>,
    },
    DatePickerChanged {
        entity: nightshade::ecs::Entity,
        year: i32,
        month: u32,
        day: u32,
    },
}

impl UiEvent {
    pub fn target_entity(&self) -> nightshade::ecs::Entity {
        match self {
            UiEvent::ButtonClicked(entity) => *entity,
            UiEvent::SliderChanged { entity, .. } => *entity,
            UiEvent::ToggleChanged { entity, .. } => *entity,
            UiEvent::CheckboxChanged { entity, .. } => *entity,
            UiEvent::RadioChanged { entity, .. } => *entity,
            UiEvent::TabChanged { entity, .. } => *entity,
            UiEvent::TextInputChanged { entity, .. } => *entity,
            UiEvent::TextInputSubmitted { entity, .. } => *entity,
            UiEvent::DropdownChanged { entity, .. } => *entity,
            UiEvent::MenuItemClicked { entity, .. } => *entity,
            UiEvent::ColorPickerChanged { entity, .. } => *entity,
            UiEvent::SelectableLabelClicked { entity, .. } => *entity,
            UiEvent::DragValueChanged { entity, .. } => *entity,
            UiEvent::ContextMenuItemClicked { entity, .. } => *entity,
            UiEvent::TreeNodeSelected { node, .. } => *node,
            UiEvent::TreeNodeToggled { node, .. } => *node,
            UiEvent::TreeNodeContextMenu { node, .. } => *node,
            UiEvent::TreeNodeExpandRequested { node, .. } => *node,
            UiEvent::ModalClosed { entity, .. } => *entity,
            UiEvent::CommandPaletteExecuted { entity, .. } => *entity,
            UiEvent::TileTabActivated { container, .. } => *container,
            UiEvent::TileTabClosed { container, .. } => *container,
            UiEvent::TileSplitterMoved { container, .. } => *container,
            UiEvent::DragStarted { source, .. } => *source,
            UiEvent::DragDropped { target, .. } => *target,
            UiEvent::DragCancelled { source, .. } => *source,
            UiEvent::DragEnter { target, .. } => *target,
            UiEvent::DragOver { target, .. } => *target,
            UiEvent::DragLeave { target, .. } => *target,
            UiEvent::ShortcutTriggered { .. } => nightshade::ecs::Entity::default(),
            UiEvent::CanvasClicked { entity, .. } => *entity,
            UiEvent::VirtualListItemClicked { entity, .. } => *entity,
            UiEvent::VirtualListItemRightClicked { entity, .. } => *entity,
            UiEvent::TextAreaChanged { entity, .. } => *entity,
            UiEvent::RichTextEditorChanged { entity, .. } => *entity,
            UiEvent::DataGridFilterChanged { entity, .. } => *entity,
            UiEvent::RangeSliderChanged { entity, .. } => *entity,
            UiEvent::DataGridCellEdited { entity, .. } => *entity,
            UiEvent::BreadcrumbClicked { entity, .. } => *entity,
            UiEvent::SplitterChanged { entity, .. } => *entity,
            UiEvent::MultiSelectChanged { entity, .. } => *entity,
            UiEvent::DatePickerChanged { entity, .. } => *entity,
        }
    }

    pub fn bubbles(&self) -> bool {
        match self {
            UiEvent::ButtonClicked(_)
            | UiEvent::SliderChanged { .. }
            | UiEvent::ToggleChanged { .. }
            | UiEvent::CheckboxChanged { .. }
            | UiEvent::RadioChanged { .. }
            | UiEvent::TabChanged { .. }
            | UiEvent::TextInputChanged { .. }
            | UiEvent::TextInputSubmitted { .. }
            | UiEvent::DropdownChanged { .. }
            | UiEvent::MenuItemClicked { .. }
            | UiEvent::ColorPickerChanged { .. }
            | UiEvent::SelectableLabelClicked { .. }
            | UiEvent::DragValueChanged { .. }
            | UiEvent::ContextMenuItemClicked { .. }
            | UiEvent::TreeNodeSelected { .. }
            | UiEvent::TreeNodeToggled { .. } => true,
            UiEvent::TreeNodeContextMenu { .. }
            | UiEvent::TreeNodeExpandRequested { .. }
            | UiEvent::ModalClosed { .. }
            | UiEvent::CommandPaletteExecuted { .. }
            | UiEvent::TileTabActivated { .. }
            | UiEvent::TileTabClosed { .. }
            | UiEvent::TileSplitterMoved { .. }
            | UiEvent::DragStarted { .. }
            | UiEvent::DragDropped { .. }
            | UiEvent::DragCancelled { .. }
            | UiEvent::DragEnter { .. }
            | UiEvent::DragOver { .. }
            | UiEvent::DragLeave { .. }
            | UiEvent::ShortcutTriggered { .. }
            | UiEvent::CanvasClicked { .. }
            | UiEvent::VirtualListItemClicked { .. }
            | UiEvent::VirtualListItemRightClicked { .. }
            | UiEvent::DataGridFilterChanged { .. }
            | UiEvent::DataGridCellEdited { .. } => false,
            UiEvent::TextAreaChanged { .. }
            | UiEvent::RichTextEditorChanged { .. }
            | UiEvent::RangeSliderChanged { .. }
            | UiEvent::BreadcrumbClicked { .. }
            | UiEvent::SplitterChanged { .. }
            | UiEvent::MultiSelectChanged { .. }
            | UiEvent::DatePickerChanged { .. } => true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BubbledUiEvent {
    pub event: UiEvent,
    pub target: nightshade::ecs::Entity,
    pub ancestor: nightshade::ecs::Entity,
    pub stopped: bool,
}

impl BubbledUiEvent {}
