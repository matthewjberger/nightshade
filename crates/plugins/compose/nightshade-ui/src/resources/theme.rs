use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::components::UiNodeShadow;

#[derive(Copy, Clone, Debug, Default)]
pub enum RectEffect {
    #[default]
    Solid,
    VerticalGradient {
        secondary: Vec4,
    },
    HorizontalGradient {
        secondary: Vec4,
    },
    RadialGlow {
        glow_color: Vec4,
        intensity: f32,
    },
    FrostedGlass {
        tint: Vec4,
        blur_amount: f32,
    },
    InsetEmboss {
        strength: f32,
        edge_softness: f32,
    },
    OutsetEmboss {
        strength: f32,
        edge_softness: f32,
    },
    SaturationValue {
        hue: f32,
    },
    ColorWheel {
        value: f32,
    },
}

impl RectEffect {
    pub fn encode(&self) -> (u32, [f32; 4]) {
        match *self {
            Self::Solid => (0, [0.0; 4]),
            Self::VerticalGradient { secondary } => {
                (1, [secondary.x, secondary.y, secondary.z, secondary.w])
            }
            Self::HorizontalGradient { secondary } => {
                (2, [secondary.x, secondary.y, secondary.z, secondary.w])
            }
            Self::RadialGlow {
                glow_color,
                intensity,
            } => (3, [glow_color.x, glow_color.y, glow_color.z, intensity]),
            Self::InsetEmboss {
                strength,
                edge_softness,
            } => (4, [strength, edge_softness, 0.0, 0.0]),
            Self::OutsetEmboss {
                strength,
                edge_softness,
            } => (5, [strength, edge_softness, 0.0, 0.0]),
            Self::FrostedGlass { tint, blur_amount } => (6, [tint.x, tint.y, tint.z, blur_amount]),
            Self::SaturationValue { hue } => (7, [hue, 0.0, 0.0, 0.0]),
            Self::ColorWheel { value } => (9, [value, 0.0, 0.0, 0.0]),
        }
    }
}

fn default_panel_shadow() -> UiNodeShadow {
    UiNodeShadow {
        color: Vec4::new(0.0, 0.0, 0.0, 0.38),
        offset: Vec2::new(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
    }
}

fn default_button_shadow() -> UiNodeShadow {
    UiNodeShadow {
        color: Vec4::new(0.0, 0.0, 0.0, 0.0),
        offset: Vec2::new(0.0, 0.0),
        blur: 0.0,
        spread: 0.0,
    }
}

fn default_button_shadow_hover() -> UiNodeShadow {
    UiNodeShadow {
        color: Vec4::new(0.0, 0.0, 0.0, 0.22),
        offset: Vec2::new(0.0, 2.0),
        blur: 6.0,
        spread: 0.0,
    }
}

fn default_button_shadow_pressed() -> UiNodeShadow {
    UiNodeShadow {
        color: Vec4::new(0.0, 0.0, 0.0, 0.0),
        offset: Vec2::new(0.0, 0.0),
        blur: 0.0,
        spread: 0.0,
    }
}

fn default_modal_shadow() -> UiNodeShadow {
    UiNodeShadow {
        color: Vec4::new(0.0, 0.0, 0.0, 0.55),
        offset: Vec2::new(0.0, 12.0),
        blur: 48.0,
        spread: 2.0,
    }
}

#[derive(Clone, Debug)]
pub struct UiTheme {
    pub name: String,
    pub dark_mode: bool,

    pub text_color: Vec4,
    pub text_color_disabled: Vec4,
    pub text_color_accent: Vec4,

    pub background_color: Vec4,
    pub background_color_hovered: Vec4,
    pub background_color_active: Vec4,

    pub panel_color: Vec4,
    pub panel_header_color: Vec4,

    pub border_color: Vec4,
    pub border_color_focused: Vec4,

    pub accent_color: Vec4,
    pub accent_color_hovered: Vec4,
    pub accent_color_active: Vec4,

    pub success_color: Vec4,
    pub warning_color: Vec4,
    pub error_color: Vec4,

    pub slider_track_color: Vec4,
    pub slider_fill_color: Vec4,

    pub input_background_color: Vec4,
    pub input_background_focused: Vec4,

    pub selection_color: Vec4,

    pub scrollbar_color: Vec4,
    pub scrollbar_color_hovered: Vec4,

    pub border_width: f32,
    pub corner_radius: f32,
    pub padding: f32,
    pub spacing: f32,
    pub font_size: f32,
    pub button_height: f32,
    pub slider_height: f32,
    pub checkbox_size: f32,
    pub toggle_width: f32,
    pub toggle_height: f32,
    pub panel_header_height: f32,

    pub heading_font_size: f32,
    pub body_font_size: f32,
    pub caption_font_size: f32,
    pub monospace_font_size: f32,

    pub panel_shadow: UiNodeShadow,
    pub button_shadow: UiNodeShadow,
    pub button_shadow_hover: UiNodeShadow,
    pub button_shadow_pressed: UiNodeShadow,
    pub modal_shadow: UiNodeShadow,
    pub focus_glow: Option<UiNodeShadow>,

    pub panel_effect: RectEffect,
    pub button_effect: RectEffect,
    pub input_effect: RectEffect,

    pub transition_speed_enter: f32,
    pub transition_speed_exit: f32,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::dark()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ThemePalette {
    pub name: &'static str,
    pub dark_mode: bool,
    pub text_color: Vec4,
    pub text_color_disabled: Vec4,
    pub text_color_accent: Vec4,
    pub background_color: Vec4,
    pub background_color_hovered: Vec4,
    pub background_color_active: Vec4,
    pub panel_color: Vec4,
    pub panel_header_color: Vec4,
    pub border_color: Vec4,
    pub border_color_focused: Vec4,
    pub accent_color: Vec4,
    pub accent_color_hovered: Vec4,
    pub accent_color_active: Vec4,
    pub success_color: Vec4,
    pub warning_color: Vec4,
    pub error_color: Vec4,
    pub slider_track_color: Vec4,
    pub slider_fill_color: Vec4,
    pub input_background_color: Vec4,
    pub input_background_focused: Vec4,
    pub selection_color: Vec4,
    pub scrollbar_color: Vec4,
    pub scrollbar_color_hovered: Vec4,
}

impl UiTheme {
    pub fn from_palette(palette: ThemePalette) -> Self {
        let focus_color = Vec4::new(
            palette.border_color_focused.x,
            palette.border_color_focused.y,
            palette.border_color_focused.z,
            0.7,
        );
        let focus_glow = Some(UiNodeShadow {
            color: focus_color,
            offset: Vec2::new(0.0, 0.0),
            blur: 5.0,
            spread: 1.0,
        });
        Self {
            name: palette.name.to_string(),
            dark_mode: palette.dark_mode,
            text_color: palette.text_color,
            text_color_disabled: palette.text_color_disabled,
            text_color_accent: palette.text_color_accent,
            background_color: palette.background_color,
            background_color_hovered: palette.background_color_hovered,
            background_color_active: palette.background_color_active,
            panel_color: palette.panel_color,
            panel_header_color: palette.panel_header_color,
            border_color: palette.border_color,
            border_color_focused: palette.border_color_focused,
            accent_color: palette.accent_color,
            accent_color_hovered: palette.accent_color_hovered,
            accent_color_active: palette.accent_color_active,
            success_color: palette.success_color,
            warning_color: palette.warning_color,
            error_color: palette.error_color,
            slider_track_color: palette.slider_track_color,
            slider_fill_color: palette.slider_fill_color,
            input_background_color: palette.input_background_color,
            input_background_focused: palette.input_background_focused,
            selection_color: palette.selection_color,
            scrollbar_color: palette.scrollbar_color,
            scrollbar_color_hovered: palette.scrollbar_color_hovered,

            border_width: 1.0,
            corner_radius: 6.0,
            padding: 8.0,
            spacing: 8.0,
            font_size: 14.0,
            button_height: 28.0,
            slider_height: 20.0,
            checkbox_size: 16.0,
            toggle_width: 36.0,
            toggle_height: 20.0,
            panel_header_height: 30.0,

            heading_font_size: 17.0,
            body_font_size: 14.0,
            caption_font_size: 12.0,
            monospace_font_size: 13.0,

            panel_shadow: default_panel_shadow(),
            button_shadow: default_button_shadow(),
            button_shadow_hover: default_button_shadow_hover(),
            button_shadow_pressed: default_button_shadow_pressed(),
            modal_shadow: default_modal_shadow(),
            focus_glow,

            panel_effect: RectEffect::Solid,
            button_effect: RectEffect::Solid,
            input_effect: RectEffect::Solid,

            transition_speed_enter: 8.0,
            transition_speed_exit: 6.0,
        }
    }
}

impl UiTheme {
    pub fn dark() -> Self {
        Self::from_palette(ThemePalette {
            name: "Dark",
            dark_mode: true,
            text_color: Vec4::new(0.867, 0.878, 0.898, 1.0),
            text_color_disabled: Vec4::new(0.404, 0.427, 0.463, 1.0),
            text_color_accent: Vec4::new(0.424, 0.714, 1.0, 1.0),
            background_color: Vec4::new(0.118, 0.125, 0.141, 1.0),
            background_color_hovered: Vec4::new(0.157, 0.169, 0.192, 1.0),
            background_color_active: Vec4::new(0.196, 0.212, 0.243, 1.0),
            panel_color: Vec4::new(0.086, 0.094, 0.106, 1.0),
            panel_header_color: Vec4::new(0.125, 0.133, 0.153, 1.0),
            border_color: Vec4::new(0.184, 0.196, 0.224, 1.0),
            border_color_focused: Vec4::new(0.231, 0.510, 0.965, 1.0),
            accent_color: Vec4::new(0.231, 0.510, 0.965, 1.0),
            accent_color_hovered: Vec4::new(0.322, 0.588, 1.0, 1.0),
            accent_color_active: Vec4::new(0.161, 0.408, 0.827, 1.0),
            success_color: Vec4::new(0.294, 0.769, 0.463, 1.0),
            warning_color: Vec4::new(0.961, 0.702, 0.259, 1.0),
            error_color: Vec4::new(0.937, 0.384, 0.384, 1.0),
            slider_track_color: Vec4::new(0.157, 0.169, 0.192, 1.0),
            slider_fill_color: Vec4::new(0.231, 0.510, 0.965, 1.0),
            input_background_color: Vec4::new(0.063, 0.071, 0.082, 1.0),
            input_background_focused: Vec4::new(0.078, 0.086, 0.102, 1.0),
            selection_color: Vec4::new(0.231, 0.510, 0.965, 0.35),
            scrollbar_color: Vec4::new(0.322, 0.345, 0.396, 0.55),
            scrollbar_color_hovered: Vec4::new(0.443, 0.475, 0.541, 0.8),
        })
    }

    pub fn dracula() -> Self {
        Self::from_palette(ThemePalette {
            name: "Dracula",
            dark_mode: true,
            text_color: Vec4::new(0.973, 0.973, 0.949, 1.0),
            text_color_disabled: Vec4::new(0.384, 0.447, 0.643, 1.0),
            text_color_accent: Vec4::new(0.545, 0.914, 0.992, 1.0),
            background_color: Vec4::new(0.157, 0.165, 0.212, 1.0),
            background_color_hovered: Vec4::new(0.267, 0.278, 0.353, 1.0),
            background_color_active: Vec4::new(0.337, 0.349, 0.424, 1.0),
            panel_color: Vec4::new(0.157, 0.165, 0.212, 1.0),
            panel_header_color: Vec4::new(0.267, 0.278, 0.353, 1.0),
            border_color: Vec4::new(0.384, 0.447, 0.643, 1.0),
            border_color_focused: Vec4::new(0.741, 0.576, 0.976, 1.0),
            accent_color: Vec4::new(0.741, 0.576, 0.976, 1.0),
            accent_color_hovered: Vec4::new(0.827, 0.686, 1.0, 1.0),
            accent_color_active: Vec4::new(0.639, 0.475, 0.855, 1.0),
            success_color: Vec4::new(0.314, 0.980, 0.482, 1.0),
            warning_color: Vec4::new(1.0, 0.722, 0.424, 1.0),
            error_color: Vec4::new(1.0, 0.333, 0.333, 1.0),
            slider_track_color: Vec4::new(0.267, 0.278, 0.353, 1.0),
            slider_fill_color: Vec4::new(0.741, 0.576, 0.976, 1.0),
            input_background_color: Vec4::new(0.118, 0.125, 0.165, 1.0),
            input_background_focused: Vec4::new(0.157, 0.165, 0.212, 1.0),
            selection_color: Vec4::new(0.384, 0.447, 0.643, 0.5),
            scrollbar_color: Vec4::new(0.384, 0.447, 0.643, 0.4),
            scrollbar_color_hovered: Vec4::new(0.384, 0.447, 0.643, 0.7),
        })
    }

    pub fn nord() -> Self {
        Self::from_palette(ThemePalette {
            name: "Nord",
            dark_mode: true,
            text_color: Vec4::new(0.847, 0.871, 0.914, 1.0),
            text_color_disabled: Vec4::new(0.369, 0.420, 0.506, 1.0),
            text_color_accent: Vec4::new(0.533, 0.753, 0.816, 1.0),
            background_color: Vec4::new(0.180, 0.204, 0.251, 1.0),
            background_color_hovered: Vec4::new(0.231, 0.259, 0.322, 1.0),
            background_color_active: Vec4::new(0.263, 0.298, 0.369, 1.0),
            panel_color: Vec4::new(0.180, 0.204, 0.251, 1.0),
            panel_header_color: Vec4::new(0.231, 0.259, 0.322, 1.0),
            border_color: Vec4::new(0.369, 0.420, 0.506, 1.0),
            border_color_focused: Vec4::new(0.533, 0.753, 0.816, 1.0),
            accent_color: Vec4::new(0.533, 0.753, 0.816, 1.0),
            accent_color_hovered: Vec4::new(0.565, 0.792, 0.863, 1.0),
            accent_color_active: Vec4::new(0.427, 0.671, 0.757, 1.0),
            success_color: Vec4::new(0.639, 0.745, 0.549, 1.0),
            warning_color: Vec4::new(0.922, 0.796, 0.545, 1.0),
            error_color: Vec4::new(0.749, 0.380, 0.416, 1.0),
            slider_track_color: Vec4::new(0.231, 0.259, 0.322, 1.0),
            slider_fill_color: Vec4::new(0.533, 0.753, 0.816, 1.0),
            input_background_color: Vec4::new(0.141, 0.165, 0.204, 1.0),
            input_background_focused: Vec4::new(0.180, 0.204, 0.251, 1.0),
            selection_color: Vec4::new(0.533, 0.753, 0.816, 0.4),
            scrollbar_color: Vec4::new(0.369, 0.420, 0.506, 0.4),
            scrollbar_color_hovered: Vec4::new(0.369, 0.420, 0.506, 0.7),
        })
    }

    pub fn gruvbox_dark() -> Self {
        Self::from_palette(ThemePalette {
            name: "Gruvbox Dark",
            dark_mode: true,
            text_color: Vec4::new(0.922, 0.859, 0.698, 1.0),
            text_color_disabled: Vec4::new(0.502, 0.502, 0.502, 1.0),
            text_color_accent: Vec4::new(0.514, 0.647, 0.596, 1.0),
            background_color: Vec4::new(0.157, 0.157, 0.157, 1.0),
            background_color_hovered: Vec4::new(0.235, 0.220, 0.212, 1.0),
            background_color_active: Vec4::new(0.314, 0.294, 0.278, 1.0),
            panel_color: Vec4::new(0.157, 0.157, 0.157, 1.0),
            panel_header_color: Vec4::new(0.235, 0.220, 0.212, 1.0),
            border_color: Vec4::new(0.400, 0.361, 0.329, 1.0),
            border_color_focused: Vec4::new(0.514, 0.647, 0.596, 1.0),
            accent_color: Vec4::new(0.514, 0.647, 0.596, 1.0),
            accent_color_hovered: Vec4::new(0.557, 0.694, 0.639, 1.0),
            accent_color_active: Vec4::new(0.427, 0.561, 0.518, 1.0),
            success_color: Vec4::new(0.722, 0.733, 0.149, 1.0),
            warning_color: Vec4::new(0.980, 0.741, 0.184, 1.0),
            error_color: Vec4::new(0.984, 0.286, 0.204, 1.0),
            slider_track_color: Vec4::new(0.235, 0.220, 0.212, 1.0),
            slider_fill_color: Vec4::new(0.514, 0.647, 0.596, 1.0),
            input_background_color: Vec4::new(0.114, 0.125, 0.129, 1.0),
            input_background_focused: Vec4::new(0.157, 0.157, 0.157, 1.0),
            selection_color: Vec4::new(0.400, 0.361, 0.329, 0.5),
            scrollbar_color: Vec4::new(0.400, 0.361, 0.329, 0.4),
            scrollbar_color_hovered: Vec4::new(0.400, 0.361, 0.329, 0.7),
        })
    }

    pub fn catppuccin_mocha() -> Self {
        Self::from_palette(ThemePalette {
            name: "Catppuccin Mocha",
            dark_mode: true,
            text_color: Vec4::new(0.804, 0.839, 0.957, 1.0),
            text_color_disabled: Vec4::new(0.345, 0.357, 0.439, 1.0),
            text_color_accent: Vec4::new(0.537, 0.706, 0.980, 1.0),
            background_color: Vec4::new(0.118, 0.118, 0.180, 1.0),
            background_color_hovered: Vec4::new(0.192, 0.196, 0.267, 1.0),
            background_color_active: Vec4::new(0.271, 0.278, 0.353, 1.0),
            panel_color: Vec4::new(0.118, 0.118, 0.180, 1.0),
            panel_header_color: Vec4::new(0.192, 0.196, 0.267, 1.0),
            border_color: Vec4::new(0.345, 0.357, 0.439, 1.0),
            border_color_focused: Vec4::new(0.537, 0.706, 0.980, 1.0),
            accent_color: Vec4::new(0.537, 0.706, 0.980, 1.0),
            accent_color_hovered: Vec4::new(0.608, 0.765, 1.0, 1.0),
            accent_color_active: Vec4::new(0.431, 0.612, 0.878, 1.0),
            success_color: Vec4::new(0.651, 0.890, 0.631, 1.0),
            warning_color: Vec4::new(0.976, 0.886, 0.686, 1.0),
            error_color: Vec4::new(0.953, 0.545, 0.659, 1.0),
            slider_track_color: Vec4::new(0.192, 0.196, 0.267, 1.0),
            slider_fill_color: Vec4::new(0.537, 0.706, 0.980, 1.0),
            input_background_color: Vec4::new(0.067, 0.067, 0.106, 1.0),
            input_background_focused: Vec4::new(0.118, 0.118, 0.180, 1.0),
            selection_color: Vec4::new(0.345, 0.357, 0.439, 0.5),
            scrollbar_color: Vec4::new(0.345, 0.357, 0.439, 0.4),
            scrollbar_color_hovered: Vec4::new(0.345, 0.357, 0.439, 0.7),
        })
    }

    pub fn all_presets() -> Vec<Self> {
        vec![
            Self::dark(),
            Self::dracula(),
            Self::nord(),
            Self::gruvbox_dark(),
            Self::catppuccin_mocha(),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct ThemeState {
    pub current_theme: UiTheme,
    pub presets: Vec<UiTheme>,
    pub selected_preset_index: Option<usize>,
    pub preview_theme_index: Option<usize>,
    pub generation: u64,
    pub last_applied_generation: u64,
    pub transition_from: Option<UiTheme>,
    pub transition_progress: f32,
    pub transition_duration: f32,
    pub transition_theme: Option<UiTheme>,
    pub type_scale: f32,
    pub type_scale_chosen: f32,
    pub type_scale_target: f32,
    pub type_scale_from: f32,
    pub type_scale_progress: f32,
    pub resolved_theme: Option<UiTheme>,
}

impl Default for ThemeState {
    fn default() -> Self {
        let presets = UiTheme::all_presets();
        Self {
            current_theme: presets[0].clone(),
            presets,
            selected_preset_index: Some(0),
            preview_theme_index: None,
            generation: 0,
            last_applied_generation: 0,
            transition_from: None,
            transition_progress: 0.0,
            transition_duration: 0.25,
            transition_theme: None,
            type_scale: 1.0,
            type_scale_chosen: 1.0,
            type_scale_target: 1.0,
            type_scale_from: 1.0,
            type_scale_progress: 1.0,
            resolved_theme: None,
        }
    }
}

impl ThemeState {
    pub fn active_theme(&self) -> &UiTheme {
        if let Some(resolved) = self.resolved_theme.as_ref() {
            return resolved;
        }
        if let Some(transition) = self.transition_theme.as_ref() {
            return transition;
        }
        self.target_theme()
    }

    fn target_theme(&self) -> &UiTheme {
        self.preview_theme_index
            .and_then(|index| self.presets.get(index))
            .unwrap_or(&self.current_theme)
    }

    pub fn select_theme(&mut self, index: usize) {
        if let Some(preset) = self.presets.get(index) {
            self.current_theme = preset.clone();
            self.selected_preset_index = Some(index);
            self.preview_theme_index = None;
            self.transition_from = None;
            self.transition_theme = None;
            self.transition_progress = 0.0;
            self.generation += 1;
            self.resolve();
        }
    }

    pub fn set_preview(&mut self, index: Option<usize>) {
        if self.preview_theme_index == index {
            return;
        }
        let from = self.active_theme().clone();
        self.preview_theme_index = index;
        self.start_transition(from);
    }

    pub fn clear_preview(&mut self) {
        if self.preview_theme_index.is_none() {
            return;
        }
        let from = self.active_theme().clone();
        self.preview_theme_index = None;
        self.start_transition(from);
    }

    fn start_transition(&mut self, from: UiTheme) {
        self.transition_from = Some(from);
        self.transition_progress = 0.0;
        self.transition_theme = None;
        self.generation += 1;
        self.resolve();
    }

    pub fn set_type_scale_preview(&mut self, scale: f32) {
        self.start_type_scale(scale.max(0.1));
    }

    pub fn clear_type_scale_preview(&mut self) {
        self.start_type_scale(self.type_scale_chosen);
    }

    pub fn select_type_scale(&mut self, scale: f32) {
        let scale = scale.max(0.1);
        self.type_scale_chosen = scale;
        self.type_scale = scale;
        self.type_scale_from = scale;
        self.type_scale_target = scale;
        self.type_scale_progress = 1.0;
        self.generation += 1;
        self.resolve();
    }

    fn start_type_scale(&mut self, target: f32) {
        if (self.type_scale_target - target).abs() < 0.0005 {
            return;
        }
        self.type_scale_from = self.type_scale;
        self.type_scale_target = target;
        self.type_scale_progress = 0.0;
        self.generation += 1;
    }

    fn resolve(&mut self) {
        if (self.type_scale - 1.0).abs() < 0.0005 {
            self.resolved_theme = None;
            return;
        }
        let base = match self.transition_theme.as_ref() {
            Some(theme) => theme.clone(),
            None => self.target_theme().clone(),
        };
        self.resolved_theme = Some(scale_type(&base, self.type_scale));
    }

    pub fn tick_transition(&mut self, delta_time: f32) {
        self.tick_type_scale(delta_time);
        let Some(from) = self.transition_from.as_ref() else {
            return;
        };
        let duration = self.transition_duration.max(0.0001);
        self.transition_progress = (self.transition_progress + delta_time / duration).min(1.0);
        if self.transition_progress >= 1.0 {
            self.transition_from = None;
            self.transition_theme = None;
            self.generation += 1;
            return;
        }
        let eased_progress = ease_in_out_cubic(self.transition_progress);
        let to = self.target_theme().clone();
        let from = from.clone();
        self.transition_theme = Some(lerp_theme_visual(&from, &to, eased_progress));
        self.generation += 1;
        self.resolve();
    }

    fn tick_type_scale(&mut self, delta_time: f32) {
        if self.type_scale_progress >= 1.0 {
            return;
        }
        let duration = self.transition_duration.max(0.0001);
        self.type_scale_progress = (self.type_scale_progress + delta_time / duration).min(1.0);
        let eased = ease_in_out_cubic(self.type_scale_progress);
        self.type_scale =
            self.type_scale_from + (self.type_scale_target - self.type_scale_from) * eased;
        self.generation += 1;
        self.resolve();
    }
}

fn scale_type(theme: &UiTheme, scale: f32) -> UiTheme {
    UiTheme {
        font_size: theme.font_size * scale,
        heading_font_size: theme.heading_font_size * scale,
        body_font_size: theme.body_font_size * scale,
        caption_font_size: theme.caption_font_size * scale,
        monospace_font_size: theme.monospace_font_size * scale,
        ..theme.clone()
    }
}

fn ease_in_out_cubic(progress: f32) -> f32 {
    if progress < 0.5 {
        4.0 * progress * progress * progress
    } else {
        1.0 - (-2.0 * progress + 2.0).powi(3) / 2.0
    }
}

fn lerp_vec4(from: Vec4, to: Vec4, factor: f32) -> Vec4 {
    from + (to - from) * factor
}

fn lerp_vec2(from: Vec2, to: Vec2, factor: f32) -> Vec2 {
    from + (to - from) * factor
}

fn lerp_shadow(from: UiNodeShadow, to: UiNodeShadow, factor: f32) -> UiNodeShadow {
    UiNodeShadow {
        color: lerp_vec4(from.color, to.color, factor),
        offset: lerp_vec2(from.offset, to.offset, factor),
        blur: from.blur + (to.blur - from.blur) * factor,
        spread: from.spread + (to.spread - from.spread) * factor,
    }
}

fn lerp_optional_shadow(
    from: Option<UiNodeShadow>,
    to: Option<UiNodeShadow>,
    factor: f32,
) -> Option<UiNodeShadow> {
    match (from, to) {
        (Some(from), Some(to)) => Some(lerp_shadow(from, to, factor)),
        (Some(from), None) => {
            let mut out = from;
            out.color.w *= 1.0 - factor;
            Some(out)
        }
        (None, Some(to)) => {
            let mut out = to;
            out.color.w *= factor;
            Some(out)
        }
        (None, None) => None,
    }
}

fn lerp_theme_visual(from: &UiTheme, to: &UiTheme, factor: f32) -> UiTheme {
    let mut out = to.clone();
    out.text_color = lerp_vec4(from.text_color, to.text_color, factor);
    out.text_color_disabled = lerp_vec4(from.text_color_disabled, to.text_color_disabled, factor);
    out.text_color_accent = lerp_vec4(from.text_color_accent, to.text_color_accent, factor);
    out.background_color = lerp_vec4(from.background_color, to.background_color, factor);
    out.background_color_hovered = lerp_vec4(
        from.background_color_hovered,
        to.background_color_hovered,
        factor,
    );
    out.background_color_active = lerp_vec4(
        from.background_color_active,
        to.background_color_active,
        factor,
    );
    out.panel_color = lerp_vec4(from.panel_color, to.panel_color, factor);
    out.panel_header_color = lerp_vec4(from.panel_header_color, to.panel_header_color, factor);
    out.border_color = lerp_vec4(from.border_color, to.border_color, factor);
    out.border_color_focused =
        lerp_vec4(from.border_color_focused, to.border_color_focused, factor);
    out.accent_color = lerp_vec4(from.accent_color, to.accent_color, factor);
    out.accent_color_hovered =
        lerp_vec4(from.accent_color_hovered, to.accent_color_hovered, factor);
    out.accent_color_active = lerp_vec4(from.accent_color_active, to.accent_color_active, factor);
    out.success_color = lerp_vec4(from.success_color, to.success_color, factor);
    out.warning_color = lerp_vec4(from.warning_color, to.warning_color, factor);
    out.error_color = lerp_vec4(from.error_color, to.error_color, factor);
    out.slider_track_color = lerp_vec4(from.slider_track_color, to.slider_track_color, factor);
    out.slider_fill_color = lerp_vec4(from.slider_fill_color, to.slider_fill_color, factor);
    out.input_background_color = lerp_vec4(
        from.input_background_color,
        to.input_background_color,
        factor,
    );
    out.input_background_focused = lerp_vec4(
        from.input_background_focused,
        to.input_background_focused,
        factor,
    );
    out.selection_color = lerp_vec4(from.selection_color, to.selection_color, factor);
    out.scrollbar_color = lerp_vec4(from.scrollbar_color, to.scrollbar_color, factor);
    out.scrollbar_color_hovered = lerp_vec4(
        from.scrollbar_color_hovered,
        to.scrollbar_color_hovered,
        factor,
    );
    out.panel_shadow = lerp_shadow(from.panel_shadow, to.panel_shadow, factor);
    out.button_shadow = lerp_shadow(from.button_shadow, to.button_shadow, factor);
    out.button_shadow_hover = lerp_shadow(from.button_shadow_hover, to.button_shadow_hover, factor);
    out.button_shadow_pressed =
        lerp_shadow(from.button_shadow_pressed, to.button_shadow_pressed, factor);
    out.modal_shadow = lerp_shadow(from.modal_shadow, to.modal_shadow, factor);
    out.focus_glow = lerp_optional_shadow(from.focus_glow, to.focus_glow, factor);
    out
}

impl crate::components::ThemeColor {
    pub fn resolve(self, theme: &UiTheme) -> Vec4 {
        match self {
            Self::Text => theme.text_color,
            Self::TextDisabled => theme.text_color_disabled,
            Self::TextAccent => theme.text_color_accent,
            Self::Background => theme.background_color,
            Self::BackgroundHover => theme.background_color_hovered,
            Self::BackgroundActive => theme.background_color_active,
            Self::Panel => theme.panel_color,
            Self::PanelHeader => theme.panel_header_color,
            Self::Border => theme.border_color,
            Self::BorderFocused => theme.border_color_focused,
            Self::Accent => theme.accent_color,
            Self::AccentHover => theme.accent_color_hovered,
            Self::AccentActive => theme.accent_color_active,
            Self::Success => theme.success_color,
            Self::Warning => theme.warning_color,
            Self::Error => theme.error_color,
            Self::SliderTrack => theme.slider_track_color,
            Self::SliderFill => theme.slider_fill_color,
            Self::InputBackground => theme.input_background_color,
            Self::InputBackgroundFocused => theme.input_background_focused,
            Self::Selection => theme.selection_color,
            Self::Scrollbar => theme.scrollbar_color,
            Self::ScrollbarHover => theme.scrollbar_color_hovered,
        }
    }
}
