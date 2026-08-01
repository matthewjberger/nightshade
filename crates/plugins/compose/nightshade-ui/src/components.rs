pub mod layer;
pub mod layout_types;

pub use layer::*;
pub use layout_types::*;

pub mod data_types;
pub mod interaction;
pub mod node;
pub mod responsive;
pub mod tiles;
pub mod widget_state;

pub use responsive::{ResponsiveOverride, UiBreakpoint, UiResponsive};

pub use data_types::{
    CanvasCommand, CharStyle, ColorPickerMode, CommandEntry, ContextMenuItem, ContextMenuItemDef,
    ContextMenuItemKind, DataGridColumn, DataGridPoolRow, DragAcceptFilter, DragPayload,
    DragValueConfig, InputMask, RangeSliderThumb, ResizeEdge, RichTextSnapshot, ShortcutBinding,
    SliderConfig, SplitDirection, TextSnapshot, TextSpan, ThemeColor, ThemeEffect, ThemeShadow,
    UiDragSource, UiDropTarget, UiPanelKind, UiThemeBinding, UndoStack, VirtualListPoolItem,
};
pub use interaction::{
    AccessibleRole, Spring, StateTransition, UiNodeInteraction, UiStateWeights, ValidationRule,
};
pub use node::{
    UiAnimationPhase, UiAnimationType, UiLayoutNode, UiNodeAnimation, UiNodeBlendedTransform,
    UiNodeColor, UiNodeContent, UiNodeEffect, UiNodeShadow, UiNodeShadowStates,
    UiNodeTransformStates,
};
pub use tiles::{
    DropPreview, DropZone, TileId, TileLayout, TileLayoutNode, TileNode, UiTileContainerData,
};
pub use widget_state::{
    UiBreadcrumbData, UiButtonData, UiCanvasData, UiCheckboxData, UiCollapsingHeaderData,
    UiColorPickerData, UiColorWheelData, UiCommandPaletteData, UiContextMenuData, UiDataGridData,
    UiDatePickerData, UiDragValueData, UiDropdownData, UiMenuData, UiModalDialogData,
    UiMultiSelectData, UiPanelData, UiProgressBarData, UiPropertyGridData, UiRadioData,
    UiRangeSliderData, UiRichTextData, UiRichTextEditorData, UiScrollAreaData,
    UiSelectableLabelData, UiSliderData, UiSpinnerData, UiSplitterData, UiTabBarData,
    UiTextAreaData, UiTextInputData, UiToggleData, UiTreeNodeData, UiTreeViewData,
    UiVirtualListData, WidgetData,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AutoSizeMode {
    #[default]
    None,
    Width,
    Height,
    Both,
}

#[derive(Clone, Debug)]
pub struct UiLayoutRoot {
    pub absolute_scale: f32,
    pub default_font_size: f32,
}

pub const DEFAULT_FONT_SIZE: f32 = 16.0;

impl Default for UiLayoutRoot {
    fn default() -> Self {
        Self {
            absolute_scale: 1.0,
            default_font_size: DEFAULT_FONT_SIZE,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TextOverflow {
    #[default]
    Visible,
    Ellipsis,
    Clip,
    Wrap,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiDepthMode {
    Add(f32),
    Set(f32),
}

impl Default for UiDepthMode {
    fn default() -> Self {
        Self::Add(1.0)
    }
}
