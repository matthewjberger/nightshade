use crate::contract::Screen;
use crate::data::ask::Ask;
use crate::data::battle::Who;
use crate::data::tutorial::Gate;
use crate::queries::tutorial::gate;
use crate::resources::GatewardenResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_ui::components::UiButtonData;
use nightshade_ui::prelude::ui_clicked;

pub fn update(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<GatewardenResources>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    let resources = &mut *resources;
    resources.tutorial.swallow_pick = false;
    if !resources.tutorial.active || resources.screen != Screen::Battle || resources.paused {
        return;
    }
    let Some(battle) = resources.battle.as_ref() else {
        return;
    };
    let Some(current) = gate(resources.tutorial) else {
        return;
    };
    let theater = &resources.theater;
    let busy = theater.delay > 0.0 || !battle.events.is_empty() || theater.lunge.is_some();
    match current {
        Gate::Next => {
            let clicked = resources
                .ui
                .lesson_next_button
                .is_some_and(|entity| ui_clicked(&mut buttons, entity));
            let advanced = actions.just_pressed(Ask::Advance);
            if clicked || advanced {
                resources.tutorial.step += 1;
                resources.tutorial.swallow_pick = advanced;
            }
        }
        Gate::EnemyTurn if battle.active == Who::Player && !busy => {
            resources.tutorial.step += 1;
        }
        _ => {}
    }
}
