use crate::commands::plant;
use crate::components::{Block, Progress};
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::items::Item;
use blocker_blocks::queries::block_at;
use blocker_blocks::resources::{Sheet, VoxelWorld};
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::data::sight::{LOOK_COSINE, REACH};
use blocker_figure::queries::{looks_at, read_the_hero};
use blocker_figure::resources::Hero;
use blocker_player::components::Motion;
use blocker_player::data::ask::Ask;
use blocker_player::data::ripe::Ripe;
use blocker_player::resources::Aim;
use blocker_pocket::commands::inventory_take_selected;
use blocker_pocket::queries::selected_stack;
use blocker_pocket::resources::Inventory;
use blocker_screen::data::notice::Notice;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;

pub fn sow(
    aim: Res<Aim>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    sheet: Res<Sheet>,
    world: Res<VoxelWorld>,
    hero: Res<Hero>,
    mut inventory: ResMut<Inventory>,
    mut notices: EventWriter<Notice>,
    mut commands: Commands,
    mut crops: Query<(&Block,)>,
    mut bodies: Query<(&Motion,)>,
) {
    let Some(motion) = read_the_hero(&hero, &mut bodies, |(motion,)| *motion) else {
        return;
    };
    if motion == Motion::Watch || !window.cursor_locked || !actions.just_pressed(Ask::Place) {
        return;
    }
    let held = selected_stack(&inventory).map(|stack| stack.item);
    let (Some(hit), Some(Item::Seed(kind))) = (aim.0, held) else {
        return;
    };
    let soil = hit.block;
    let above = soil + IVec3::new(0, 1, 0);
    let mut taken = false;
    crops.for_each_mut(|_, (block,)| taken |= block.0 == soil);
    if hit.normal.y > 0
        && BlockKind::from_index(hit.kind).grows_crops()
        && block_at(&world, above) == BlockKind::Air
        && !taken
        && inventory_take_selected(&mut inventory, 1).is_some()
    {
        plant(&mut commands, &world, &sheet, kind, soil);
    } else {
        notices.send(Notice("Seeds want open dirt or grass".into()));
    }
}

pub fn offer_the_harvest(
    hero: Res<Hero>,
    mut ripe: EventWriter<Ripe>,
    mut crops: Query<(&Block, &Progress)>,
    mut bodies: Query<(&Eye, &Gaze)>,
) {
    let Some((eye, gaze)) = read_the_hero(&hero, &mut bodies, |(eye, gaze)| (eye.0, gaze.0)) else {
        return;
    };
    let mut found = None;
    crops.for_each_mut(|entity, (block, progress)| {
        let top = Vec3::new(
            block.0.x as f32 + 0.5,
            block.0.y as f32 + 1.4,
            block.0.z as f32 + 0.5,
        );
        if progress.0 >= 1.0 && looks_at(eye, gaze, top, REACH, LOOK_COSINE) {
            found = Some(entity);
        }
    });
    if let Some(entity) = found {
        ripe.send(Ripe(entity));
    }
}
