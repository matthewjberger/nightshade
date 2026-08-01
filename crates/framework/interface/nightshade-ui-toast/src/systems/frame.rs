use crate::resources::ToastCard;
use nightshade::prelude::*;
use nightshade_toast::Toast;
use nightshade_ui::commands::widgets::world_layout::{ui_set_text, ui_set_visible};
use nightshade_ui::resources::RetainedUiDirty;

pub fn show_the_toast(
    toast: Res<Toast>,
    mut card: ResMut<ToastCard>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let showing = toast.remaining > 0.0;
    ui_set_visible(&mut dirty, &mut commands, card.card, showing);
    if !showing || card.shown == toast.line {
        return;
    }
    if let Some(line) = card.line {
        ui_set_text(&mut commands, line, &toast.line);
    }
    card.shown = toast.line.clone();
}
