use crate::data::Bond;
use crate::data::Fighter;
use crate::queries::closest_bond;
use crate::theme;
use nalgebra_glm::Vec4;
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

pub fn set_fill(
    dirty: &mut nightshade_ui::resources::RetainedUiDirty,
    commands: &mut Commands,
    fill: Option<Entity>,
    share: f32,
    color: Vec4,
) {
    let Some(fill) = fill else {
        return;
    };
    ui_set_window_layout(
        dirty,
        commands,
        fill,
        WindowLayout {
            position: Ab(vec2(0.0, 0.0)).into(),
            size: Ab(vec2((theme::BAR_WIDTH * share).max(0.0), theme::BAR_HEIGHT)).into(),
            anchor: Anchor::TopLeft,
        },
    );
    ui_set_color::<UiBase>(commands, fill, color);
}

pub fn set_text(commands: &mut Commands, label: Option<Entity>, text: &str) {
    if let Some(label) = label {
        ui_set_text(commands, label, text);
    }
}

pub fn set_bond_row(
    commands: &mut Commands,
    label: Option<Entity>,
    squad: &[(Entity, Fighter)],
    index: usize,
) {
    let line = match closest_bond(squad, index) {
        Some((other, points)) => format!(
            "Closest to {}  {}",
            squad[other].1.name,
            Bond::of(points).grade()
        ),
        None => "No bonds yet".to_string(),
    };
    set_text(commands, label, &line);
}

pub fn show_menu_rows(
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    buttons: &[Entity],
    rows: usize,
) {
    for (item, button) in buttons.iter().enumerate() {
        ui_set_visible(dirty, commands, *button, item < rows);
    }
}

pub fn set_menu_height(
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    panel: Option<Entity>,
    rows: usize,
) {
    let Some(panel) = panel else {
        return;
    };
    let height = rows as f32 * (theme::CAMP_ROW + 6.0) + theme::CAMP_MENU_PAD;
    ui_set_window_layout(
        dirty,
        commands,
        panel,
        WindowLayout {
            position: Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::SPEECH_LIFT)),
            size: Ab(vec2(theme::CAMP_MENU_WIDTH, height)).into(),
            anchor: Anchor::BottomCenter,
        },
    );
}
