use crate::queries::{built, wash};
use crate::resources::{Flash, FlashHandles, FlashShown};
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;
use nightshade_ui::data::state::UiBase;
use nightshade_ui::prelude::ui_set_color;

pub fn fade_the_flash(
    time: Res<Time>,
    mut flash: ResMut<Flash>,
    handles: Res<FlashHandles>,
    mut shown: ResMut<FlashShown>,
    mut commands: Commands,
) {
    let delta = time.delta_time;
    for held in &mut flash.washes {
        held.amount = (held.amount - held.fade * delta).max(0.0);
    }
    flash.washes.retain(|held| held.amount > 0.0);
    if !built(&handles) {
        return;
    }
    let color = wash(&flash);
    if shown.color == Some(color) {
        return;
    }
    shown.color = Some(color);
    ui_set_color::<UiBase>(&mut commands, handles.panel, color);
}
