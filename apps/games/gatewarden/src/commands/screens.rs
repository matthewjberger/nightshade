use crate::contract::Screen;
use crate::resources::GatewardenResources;
use nightshade::prelude::*;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;

pub fn set_screen(
    resources: &mut GatewardenResources,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    screen: Screen,
) {
    resources.screen = screen;
    let handles = resources.ui;
    let in_battle = screen == Screen::Battle;
    let in_shop = screen == Screen::Shop;
    ui_set_visible(
        dirty,
        commands,
        handles.title_panel,
        screen == Screen::Title,
    );
    let in_scene = screen == Screen::Scene;
    let in_map = screen == Screen::Map;
    ui_set_visible(dirty, commands, handles.scene_panel, in_scene);
    ui_set_visible(dirty, commands, handles.map_panel, in_map);
    ui_set_visible(
        dirty,
        commands,
        handles.strip,
        in_battle || in_shop || in_scene || in_map,
    );
    ui_set_visible(dirty, commands, handles.relic_row, in_battle);
    ui_set_visible(dirty, commands, handles.table_surface, in_battle);
    ui_set_visible(dirty, commands, handles.hand_surface, in_battle);
    ui_set_visible(dirty, commands, handles.divider, in_battle);
    ui_set_visible(dirty, commands, handles.arena, in_battle);
    ui_set_visible(dirty, commands, handles.enemy_hero, in_battle);
    ui_set_visible(dirty, commands, handles.player_hero, in_battle);
    ui_set_visible(dirty, commands, handles.controls, in_battle);
    ui_set_visible(dirty, commands, handles.log_panel, in_battle);
    ui_set_visible(dirty, commands, handles.shop_panel, in_shop);
    ui_set_visible(dirty, commands, handles.over_panel, screen == Screen::Over);
}
