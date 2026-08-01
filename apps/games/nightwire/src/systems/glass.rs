use crate::data::Ask;
use crate::queries::glass_label;
use crate::resources::NightwireResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::Time;
use nightshade_ui::prelude::{ui_button_clicks, ui_set_text};
use nightshade_ui::resources::RetainedUiFrame;

const FADE_PER_SECOND: f32 = 3.0;

pub fn update(
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    frame: Res<RetainedUiFrame>,
    mut resources: ResMut<NightwireResources>,
    mut commands: Commands,
) {
    let button = resources.ui.glass_button;
    let clicked = ui_button_clicks(&frame).any(|entity| entity == button);
    if clicked || actions.just_pressed(Ask::Glass) {
        resources.glass_on = !resources.glass_on;
        ui_set_text(&mut commands, button, glass_label(resources.glass_on));
    }

    let target = if resources.glass_on { 1.0 } else { 0.0 };
    let step = time.delta_time * FADE_PER_SECOND;
    let amount = resources.glass_amount;
    resources.glass_amount = if (target - amount).abs() <= step {
        target
    } else if target > amount {
        amount + step
    } else {
        amount - step
    };
}
