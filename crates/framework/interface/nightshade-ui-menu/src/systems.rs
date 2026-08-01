use crate::resources::Menus;
use nightshade::prelude::*;
use nightshade_state::resources::State;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;

pub fn follow_the_screen<S: Copy + PartialEq + Send + Sync + 'static>(
    screen: Res<State<S>>,
    mut menus: ResMut<Menus<S>>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let current = screen.current;
    if menus.shown == Some(current) {
        return;
    }
    menus.shown = Some(current);
    for (owner, panel) in &menus.panels {
        if *panel == Entity::default() {
            continue;
        }
        ui_set_visible(&mut dirty, &mut commands, *panel, *owner == current);
    }
}
