use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::data::state::STATE_COUNT;
use nightshade_text::contract::data::text_data::TextAlignment;

#[derive(Clone, Debug, Default)]
pub enum InputMask {
    #[default]
    None,
    Numeric,
    Decimal,
    Alpha,
    AlphaNumeric,
    Custom(fn(char) -> bool),
}

impl InputMask {
    pub fn accepts(&self, character: char) -> bool {
        match self {
            Self::None => true,
            Self::Numeric => character.is_ascii_digit(),
            Self::Decimal => character.is_ascii_digit() || character == '.' || character == '-',
            Self::Alpha => character.is_alphabetic(),
            Self::AlphaNumeric => character.is_alphanumeric(),
            Self::Custom(predicate) => predicate(character),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeSliderThumb {
    Low,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorPickerMode {
    #[default]
    Rgb,
    Hsv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum UiPanelKind {
    #[default]
    Floating,
    DockedLeft,
    DockedRight,
    DockedTop,
    DockedBottom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum SplitDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct TextSnapshot {
    pub text: String,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct RichTextSnapshot {
    pub text: String,
    pub char_styles: Vec<CharStyle>,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct UndoStack<T> {
    history: Vec<T>,
    position: usize,
    max_entries: usize,
    last_push_time: f64,
}

impl<T> Default for UndoStack<T> {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            position: 0,
            max_entries: 100,
            last_push_time: 0.0,
        }
    }
}

impl<T> UndoStack<T> {
    pub fn new(max_entries: usize) -> Self {
        Self {
            history: Vec::new(),
            position: 0,
            max_entries,
            last_push_time: 0.0,
        }
    }

    pub fn push_initial(&mut self, snapshot: T) {
        if self.history.is_empty() {
            self.history.push(snapshot);
            self.position = 0;
        }
    }

    pub fn push(&mut self, snapshot: T, time: f64) {
        self.history.truncate(self.position + 1);
        if time - self.last_push_time < 0.3 && self.history.len() > 1 {
            *self.history.last_mut().unwrap() = snapshot;
        } else {
            self.history.push(snapshot);
            if self.history.len() > self.max_entries {
                self.history.remove(0);
            }
        }
        self.position = self.history.len() - 1;
        self.last_push_time = time;
    }

    pub fn undo(&mut self) -> Option<&T> {
        if self.position > 0 {
            self.position -= 1;
            Some(&self.history[self.position])
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&T> {
        if self.position + 1 < self.history.len() {
            self.position += 1;
            Some(&self.history[self.position])
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub color: Option<Vec4>,
    pub font_size_override: Option<f32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub font_index: Option<usize>,
}

impl TextSpan {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            color: None,
            font_size_override: None,
            bold: false,
            italic: false,
            underline: false,
            font_index: None,
        }
    }

    pub fn colored(text: &str, color: Vec4) -> Self {
        Self {
            text: text.to_string(),
            color: Some(color),
            font_size_override: None,
            bold: false,
            italic: false,
            underline: false,
            font_index: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_size(mut self, font_size: f32) -> Self {
        self.font_size_override = Some(font_size);
        self
    }

    pub fn sized(text: &str, font_size: f32) -> Self {
        Self {
            text: text.to_string(),
            color: None,
            font_size_override: Some(font_size),
            bold: false,
            italic: false,
            underline: false,
            font_index: None,
        }
    }

    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn with_underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn with_font(mut self, font_index: usize) -> Self {
        self.font_index = Some(font_index);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CharStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<Vec4>,
}

#[derive(Clone, Debug)]
pub struct DataGridColumn {
    pub label: String,
    pub width: f32,
    pub sortable: bool,
    pub alignment: TextAlignment,
    pub editable: bool,
}

impl DataGridColumn {
    pub fn new(label: &str, width: f32) -> Self {
        Self {
            label: label.to_string(),
            width,
            sortable: false,
            alignment: TextAlignment::Left,
            editable: false,
        }
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

#[derive(Clone, Debug)]
pub struct DataGridPoolRow {
    pub row_entity: nightshade::ecs::Entity,
    pub cell_text_slots: Vec<usize>,
    pub cell_entities: Vec<nightshade::ecs::Entity>,
}
#[derive(Clone, Debug)]
pub struct VirtualListPoolItem {
    pub container_entity: nightshade::ecs::Entity,
}

#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub separator: bool,
}

#[derive(Clone, Debug)]
pub enum ContextMenuItemKind {
    Action,
    Checkable {
        checked: bool,
        check_text_slot: usize,
    },
    Separator,
    Submenu {
        children: Vec<ContextMenuItemDef>,
        popup_entity: nightshade::ecs::Entity,
        open: bool,
    },
    Widget {
        content_entity: nightshade::ecs::Entity,
    },
}

#[derive(Clone, Debug)]
pub struct ContextMenuItemDef {
    pub label: String,
    pub shortcut: String,
    pub kind: ContextMenuItemKind,
    pub row_entity: Option<nightshade::ecs::Entity>,
    pub command_id: Option<usize>,
    pub tag: u32,
    pub binding: Option<ShortcutBinding>,
}

#[derive(Clone, Debug)]
pub struct CommandEntry {
    pub label: String,
    pub shortcut: String,
    pub category: String,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub enum DragPayload {
    Index(usize),
    Entity(nightshade::ecs::Entity),
    Text(String),
    Custom(u64),
}

impl Default for DragPayload {
    fn default() -> Self {
        Self::Custom(0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct UiDragSource {
    pub payload: DragPayload,
}

#[derive(Clone, Debug, Default)]
pub enum DragAcceptFilter {
    #[default]
    Any,
    IndexOnly,
    EntityOnly,
    TextOnly,
    CustomOnly,
}

impl DragAcceptFilter {
    pub fn accepts(&self, payload: &DragPayload) -> bool {
        match self {
            Self::Any => true,
            Self::IndexOnly => matches!(payload, DragPayload::Index(_)),
            Self::EntityOnly => matches!(payload, DragPayload::Entity(_)),
            Self::TextOnly => matches!(payload, DragPayload::Text(_)),
            Self::CustomOnly => matches!(payload, DragPayload::Custom(_)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiDropTarget {
    pub filter: DragAcceptFilter,
}

impl Default for UiDropTarget {
    fn default() -> Self {
        Self {
            filter: DragAcceptFilter::Any,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShortcutBinding {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: nightshade_platform::KeyCode,
}

impl ShortcutBinding {
    pub fn parse(shortcut: &str) -> Option<Self> {
        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;
        let mut key_part = "";

        for part in shortcut.split('+') {
            let trimmed = part.trim();
            match trimmed.to_lowercase().as_str() {
                "ctrl" => ctrl = true,
                "shift" => shift = true,
                "alt" => alt = true,
                _ => key_part = trimmed,
            }
        }

        let key = match key_part.to_uppercase().as_str() {
            "A" => nightshade_platform::KeyCode::KeyA,
            "B" => nightshade_platform::KeyCode::KeyB,
            "C" => nightshade_platform::KeyCode::KeyC,
            "D" => nightshade_platform::KeyCode::KeyD,
            "E" => nightshade_platform::KeyCode::KeyE,
            "F" => nightshade_platform::KeyCode::KeyF,
            "G" => nightshade_platform::KeyCode::KeyG,
            "H" => nightshade_platform::KeyCode::KeyH,
            "I" => nightshade_platform::KeyCode::KeyI,
            "J" => nightshade_platform::KeyCode::KeyJ,
            "K" => nightshade_platform::KeyCode::KeyK,
            "L" => nightshade_platform::KeyCode::KeyL,
            "M" => nightshade_platform::KeyCode::KeyM,
            "N" => nightshade_platform::KeyCode::KeyN,
            "O" => nightshade_platform::KeyCode::KeyO,
            "P" => nightshade_platform::KeyCode::KeyP,
            "Q" => nightshade_platform::KeyCode::KeyQ,
            "R" => nightshade_platform::KeyCode::KeyR,
            "S" => nightshade_platform::KeyCode::KeyS,
            "T" => nightshade_platform::KeyCode::KeyT,
            "U" => nightshade_platform::KeyCode::KeyU,
            "V" => nightshade_platform::KeyCode::KeyV,
            "W" => nightshade_platform::KeyCode::KeyW,
            "X" => nightshade_platform::KeyCode::KeyX,
            "Y" => nightshade_platform::KeyCode::KeyY,
            "Z" => nightshade_platform::KeyCode::KeyZ,
            "F1" => nightshade_platform::KeyCode::F1,
            "F2" => nightshade_platform::KeyCode::F2,
            "F3" => nightshade_platform::KeyCode::F3,
            "F4" => nightshade_platform::KeyCode::F4,
            "F5" => nightshade_platform::KeyCode::F5,
            "F6" => nightshade_platform::KeyCode::F6,
            "F7" => nightshade_platform::KeyCode::F7,
            "F8" => nightshade_platform::KeyCode::F8,
            "F9" => nightshade_platform::KeyCode::F9,
            "F10" => nightshade_platform::KeyCode::F10,
            "F11" => nightshade_platform::KeyCode::F11,
            "F12" => nightshade_platform::KeyCode::F12,
            "ESCAPE" | "ESC" => nightshade_platform::KeyCode::Escape,
            "ENTER" | "RETURN" => nightshade_platform::KeyCode::Enter,
            "SPACE" => nightshade_platform::KeyCode::Space,
            "DELETE" | "DEL" => nightshade_platform::KeyCode::Delete,
            "BACKSPACE" => nightshade_platform::KeyCode::Backspace,
            "TAB" => nightshade_platform::KeyCode::Tab,
            _ => return None,
        };

        Some(Self {
            ctrl,
            shift,
            alt,
            key,
        })
    }
}

#[derive(Clone, Debug)]
pub enum CanvasCommand {
    Rect {
        position: Vec2,
        size: Vec2,
        color: Vec4,
        corner_radius: f32,
        id: Option<u32>,
    },
    Circle {
        center: Vec2,
        radius: f32,
        color: Vec4,
        id: Option<u32>,
    },
    Line {
        from: Vec2,
        to: Vec2,
        thickness: f32,
        color: Vec4,
        id: Option<u32>,
    },
    Text {
        text: String,
        position: Vec2,
        font_size: f32,
        color: Vec4,
        id: Option<u32>,
    },
    Polyline {
        points: Vec<Vec2>,
        thickness: f32,
        color: Vec4,
        closed: bool,
        id: Option<u32>,
    },
    RectStroke {
        position: Vec2,
        size: Vec2,
        color: Vec4,
        thickness: f32,
        corner_radius: f32,
        id: Option<u32>,
    },
    CircleStroke {
        center: Vec2,
        radius: f32,
        color: Vec4,
        thickness: f32,
        id: Option<u32>,
    },
    Arc {
        center: Vec2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        thickness: f32,
        color: Vec4,
        id: Option<u32>,
    },
    QuadraticBezier {
        start: Vec2,
        control: Vec2,
        end: Vec2,
        thickness: f32,
        color: Vec4,
        id: Option<u32>,
    },
    CubicBezier {
        start: Vec2,
        control1: Vec2,
        control2: Vec2,
        end: Vec2,
        thickness: f32,
        color: Vec4,
        id: Option<u32>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeColor {
    Text,
    TextDisabled,
    TextAccent,
    Background,
    BackgroundHover,
    BackgroundActive,
    Panel,
    PanelHeader,
    Border,
    BorderFocused,
    Accent,
    AccentHover,
    AccentActive,
    Success,
    Warning,
    Error,
    SliderTrack,
    SliderFill,
    InputBackground,
    InputBackgroundFocused,
    Selection,
    Scrollbar,
    ScrollbarHover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeShadow {
    PanelShadow,
    ButtonShadow,
    ButtonShadowHover,
    ButtonShadowPressed,
    ModalShadow,
    FocusGlow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeEffect {
    PanelEffect,
    ButtonEffect,
    InputEffect,
}

#[derive(Clone, Copy, Debug)]
pub struct UiThemeBinding {
    pub color_roles: [Option<ThemeColor>; STATE_COUNT],
    pub border_color_role: Option<ThemeColor>,
    pub shadow_roles: [Option<ThemeShadow>; STATE_COUNT],
    pub effect_role: Option<ThemeEffect>,
}

impl Default for UiThemeBinding {
    fn default() -> Self {
        Self {
            color_roles: [None; STATE_COUNT],
            border_color_role: None,
            shadow_roles: [None; STATE_COUNT],
            effect_role: None,
        }
    }
}

pub struct DragValueConfig<'a> {
    pub initial: f32,
    pub min: f32,
    pub max: f32,
    pub speed: f32,
    pub precision: usize,
    pub prefix: &'a str,
    pub suffix: &'a str,
    pub show_arrows: bool,
}

impl<'a> DragValueConfig<'a> {
    pub fn new(min: f32, max: f32, initial: f32) -> Self {
        Self {
            initial,
            min,
            max,
            speed: 0.1,
            precision: 2,
            prefix: "",
            suffix: "",
            show_arrows: true,
        }
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }
}

pub struct SliderConfig<'a> {
    pub min: f32,
    pub max: f32,
    pub initial: f32,
    pub logarithmic: bool,
    pub precision: usize,
    pub prefix: &'a str,
    pub suffix: &'a str,
}

impl<'a> SliderConfig<'a> {
    pub fn new(min: f32, max: f32, initial: f32) -> Self {
        Self {
            min,
            max,
            initial,
            logarithmic: false,
            precision: 1,
            prefix: "",
            suffix: "",
        }
    }

    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }

    pub fn logarithmic(mut self) -> Self {
        self.logarithmic = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn text_span_new_defaults() {
        let span = TextSpan::new("hello");
        assert_eq!(span.text, "hello");
        assert!(span.color.is_none());
        assert!(span.font_size_override.is_none());
        assert!(!span.bold);
        assert!(!span.italic);
        assert!(!span.underline);
        assert!(span.font_index.is_none());
    }

    #[test]
    fn text_span_colored() {
        let color = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let span = TextSpan::colored("red", color);
        assert_eq!(span.text, "red");
        assert_eq!(span.color, Some(color));
    }

    #[test]
    fn text_span_builder_chain() {
        let span = TextSpan::new("styled")
            .with_bold()
            .with_italic()
            .with_underline()
            .with_font(2)
            .with_color(Vec4::new(0.0, 1.0, 0.0, 1.0))
            .with_size(24.0);
        assert!(span.bold);
        assert!(span.italic);
        assert!(span.underline);
        assert_eq!(span.font_index, Some(2));
        assert_eq!(span.color, Some(Vec4::new(0.0, 1.0, 0.0, 1.0)));
        assert_eq!(span.font_size_override, Some(24.0));
    }

    #[test]
    fn data_grid_column_defaults() {
        let col = DataGridColumn::new("Name", 100.0);
        assert_eq!(col.label, "Name");
        assert_eq!(col.width, 100.0);
        assert!(!col.sortable);
    }

    #[test]
    fn data_grid_column_sortable() {
        let col = DataGridColumn::new("Name", 100.0).sortable();
        assert!(col.sortable);
    }

    #[test]
    fn data_grid_column_non_sortable() {
        let col = DataGridColumn {
            label: "Actions".to_string(),
            width: 80.0,
            sortable: false,
            alignment: TextAlignment::Left,
            editable: false,
        };
        assert!(!col.sortable);
    }

    #[test]
    fn context_menu_item_separator() {
        let item = ContextMenuItem {
            label: String::new(),
            shortcut: None,
            separator: true,
        };
        assert!(item.separator);
        assert!(item.label.is_empty());
        assert!(item.shortcut.is_none());
    }

    #[test]
    fn selected_rows_hashset_operations() {
        let mut selected: HashSet<usize> = HashSet::new();
        selected.insert(5);
        selected.insert(10);
        selected.insert(5);
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&5));
        assert!(selected.contains(&10));
        selected.remove(&5);
        assert_eq!(selected.len(), 1);
        assert!(!selected.contains(&5));
    }
}
