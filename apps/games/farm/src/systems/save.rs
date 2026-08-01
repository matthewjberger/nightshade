use crate::commands::save::restore;
use crate::data::Save;
use crate::resources::{
    Diorama, Dungeon, Farmland, Hero, Inventory, Quests, Sheet, Terrain, Vigor,
};
use crate::theme;
use nightshade::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_save::prelude::save_read;

pub fn load_the_farm(
    sheet: Res<Sheet>,
    terrain: Res<Terrain>,
    mut renderer_state: ResMut<RendererState>,
    mut farmland: ResMut<Farmland>,
    mut inventory: ResMut<Inventory>,
    mut quests: ResMut<Quests>,
    mut hero: ResMut<Hero>,
    mut vigor: ResMut<Vigor>,
    mut diorama: ResMut<Diorama>,
    mut dungeon: ResMut<Dungeon>,
    mut commands: Commands,
) {
    let Some(save) = save_read::<Save>(theme::SAVE_SLOT) else {
        return;
    };
    renderer_state.day_night.hour = save.hour;
    diorama.last_hour = save.hour;
    restore(
        &mut commands,
        &sheet,
        &terrain,
        &mut farmland,
        &mut inventory,
        &mut quests,
        &mut hero,
        &mut vigor,
        &mut diorama,
        &mut dungeon,
        &save,
    );
}
