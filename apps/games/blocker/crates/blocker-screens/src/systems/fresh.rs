use crate::resources::LoadingPanel;
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn forget_the_loading(fresh: EventReader<NewWorld>, mut loading_panel: ResMut<LoadingPanel>) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    loading_panel.welcomed = false;
    loading_panel.enter_shown = false;
    loading_panel.tip_since = 0.0;
    loading_panel.shown_count.clear();
    loading_panel.shown_place.clear();
    loading_panel.shown_tip.clear();
}
