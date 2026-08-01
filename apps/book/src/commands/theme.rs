use nightshade::prelude::{Vec2, Vec4, vec4};
use nightshade_ui::components::UiNodeShadow;
use nightshade_ui::resources::ThemeState;
use nightshade_ui::resources::theme::{RectEffect, ThemePalette, UiTheme};

use crate::data::style::{BODY_SIZE, CAPTION_SIZE, CODE_SIZE, LABEL_SIZE};

pub const BOOK_DARK: &str = "Book Dark";

pub fn install_book_themes(theme: &mut ThemeState) {
    let rest: Vec<UiTheme> = theme
        .presets
        .drain(..)
        .filter(|preset| preset.name != BOOK_DARK && preset.dark_mode)
        .collect();

    let mut presets = vec![book_dark()];
    presets.extend(rest);

    theme.presets = presets;
    theme.select_theme(0);
}

fn book_dark() -> UiTheme {
    let accent = vec4(0.365, 0.616, 0.933, 1.0);
    with_book_metrics(
        UiTheme::from_palette(ThemePalette {
            name: BOOK_DARK,
            dark_mode: true,
            text_color: vec4(0.878, 0.890, 0.906, 1.0),
            text_color_disabled: vec4(0.451, 0.475, 0.510, 1.0),
            text_color_accent: vec4(0.545, 0.729, 0.976, 1.0),
            background_color: vec4(0.070, 0.075, 0.086, 1.0),
            background_color_hovered: vec4(0.106, 0.114, 0.129, 1.0),
            background_color_active: vec4(0.137, 0.149, 0.169, 1.0),
            panel_color: vec4(0.098, 0.106, 0.122, 1.0),
            panel_header_color: vec4(0.122, 0.133, 0.153, 1.0),
            border_color: vec4(0.180, 0.196, 0.224, 1.0),
            border_color_focused: vec4(0.400, 0.620, 0.925, 1.0),
            accent_color: accent,
            accent_color_hovered: vec4(0.463, 0.694, 0.976, 1.0),
            accent_color_active: vec4(0.286, 0.510, 0.812, 1.0),
            success_color: vec4(0.400, 0.769, 0.518, 1.0),
            warning_color: vec4(0.925, 0.706, 0.310, 1.0),
            error_color: vec4(0.910, 0.400, 0.380, 1.0),
            slider_track_color: vec4(0.180, 0.196, 0.224, 1.0),
            slider_fill_color: accent,
            input_background_color: vec4(0.078, 0.086, 0.098, 1.0),
            input_background_focused: vec4(0.106, 0.118, 0.137, 1.0),
            selection_color: vec4(accent.x, accent.y, accent.z, 0.30),
            scrollbar_color: vec4(0.235, 0.255, 0.290, 1.0),
            scrollbar_color_hovered: vec4(0.325, 0.353, 0.400, 1.0),
        }),
        vec4(0.0, 0.0, 0.0, 0.45),
    )
}

fn with_book_metrics(theme: UiTheme, shadow_color: Vec4) -> UiTheme {
    UiTheme {
        border_width: 1.0,
        corner_radius: 8.0,
        padding: 10.0,
        spacing: 8.0,
        font_size: LABEL_SIZE + 2.0,
        button_height: 30.0,
        panel_header_height: 34.0,
        heading_font_size: 20.5,
        body_font_size: BODY_SIZE,
        caption_font_size: CAPTION_SIZE,
        monospace_font_size: CODE_SIZE,
        panel_shadow: UiNodeShadow {
            color: shadow_color,
            offset: Vec2::new(0.0, 2.0),
            blur: 14.0,
            spread: 0.0,
        },
        button_shadow: UiNodeShadow {
            color: vec4(shadow_color.x, shadow_color.y, shadow_color.z, 0.0),
            offset: Vec2::new(0.0, 0.0),
            blur: 0.0,
            spread: 0.0,
        },
        button_shadow_hover: UiNodeShadow {
            color: shadow_color,
            offset: Vec2::new(0.0, 1.0),
            blur: 6.0,
            spread: 0.0,
        },
        button_shadow_pressed: UiNodeShadow {
            color: shadow_color,
            offset: Vec2::new(0.0, 0.0),
            blur: 2.0,
            spread: 0.0,
        },
        modal_shadow: UiNodeShadow {
            color: vec4(shadow_color.x, shadow_color.y, shadow_color.z, 0.32),
            offset: Vec2::new(0.0, 8.0),
            blur: 40.0,
            spread: 0.0,
        },
        panel_effect: RectEffect::Solid,
        button_effect: RectEffect::Solid,
        input_effect: RectEffect::Solid,
        transition_speed_enter: 10.0,
        transition_speed_exit: 8.0,
        ..theme
    }
}
