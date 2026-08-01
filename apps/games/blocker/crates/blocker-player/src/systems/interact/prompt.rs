use crate::components::{Boat, Motion, Riding, Velocity, Yaw};
use crate::data::ask::Ask;
use crate::data::offered::Offered;
use crate::data::ripe::Ripe;
use crate::resources::{Aim, Prompt};
use crate::theme;
use blocker_blocks::commands::swing_the_door;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::crops::CropKind;
use blocker_blocks::data::items::Item;
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::queries::{looks_at, read_the_hero};
use blocker_figure::resources::Hero;
use blocker_pocket::commands::inventory_add;
use blocker_pocket::queries::slots_from;
use blocker_pocket::resources::{Chests, Inventory, OpenChest};
use blocker_screen::data::notice::Notice;
use blocker_screen::data::screen::Screen;
use blocker_world::queries::ruin_loot;
use blocker_world::resources::Generation;
use nalgebra_glm::Vec3;
use nightshade::ecs::system_param::{EventReader, EventWriter};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use nightshade_state::prelude::*;

pub fn find_the_prompt(
    aim: Res<Aim>,
    generation: Res<Generation>,
    ripe: EventReader<Ripe>,
    offers: EventReader<Offered>,
    hero: Res<Hero>,
    mut prompt: ResMut<Prompt>,
    mut boats: Query<(&Boat, &LocalTransform)>,
    mut bodies: Query<(&Motion, &Eye, &Gaze, &LocalTransform, Option<&Riding>)>,
) {
    let Some((motion, eye, gaze, place, riding)) = read_the_hero(
        &hero,
        &mut bodies,
        |(motion, eye, gaze, transform, riding)| {
            (
                *motion,
                eye.0,
                gaze.0,
                transform.translation,
                riding.is_some(),
            )
        },
    ) else {
        return;
    };
    if motion == Motion::Watch {
        *prompt = Prompt::None;
        return;
    }
    let mut found = ripe
        .into_iter()
        .next()
        .map(|ripe| Prompt::Harvest(ripe.0))
        .unwrap_or(Prompt::None);
    if riding {
        *prompt = Prompt::Leave;
        return;
    }
    if found == Prompt::None {
        let reach = theme::BOAT_REACH;

        boats.for_each_mut(|entity, (_, transform)| {
            let near = (transform.translation - place).norm() < reach;
            let seen = looks_at(
                eye,
                gaze,
                transform.translation,
                blocker_figure::data::sight::REACH,
                blocker_figure::data::sight::LOOK_COSINE,
            );
            if found == Prompt::None && (near || seen) {
                found = Prompt::Board(entity);
            }
        });
    }
    if found == Prompt::None
        && let Some(hit) = aim.0
    {
        found = match BlockKind::from_index(hit.kind) {
            kind if kind.door_facing().is_some() => Prompt::Door(
                hit.block,
                kind.door_facing().map(|(_, open)| open).unwrap_or_default(),
            ),
            BlockKind::CraftingTable => Prompt::Craft,
            BlockKind::Chest => Prompt::Chest(hit.block),
            BlockKind::Signpost => Prompt::Sign(
                generation
                    .signs
                    .get(&[hit.block.x, hit.block.y, hit.block.z])
                    .cloned()
                    .unwrap_or_else(|| blocker_world::data::world::SIGN_TEXT.to_string()),
            ),
            _ => Prompt::None,
        };
    }
    if found == Prompt::None
        && let Some(offer) = offers.into_iter().next()
    {
        found = Prompt::Talk(offer.0.clone());
    }
    *prompt = found;
}

pub fn open_the_chest(
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    prompt: Res<Prompt>,
    mut chests: ResMut<Chests>,
    mut open: ResMut<OpenChest>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !window.cursor_locked || !actions.just_pressed(Ask::Act) {
        return;
    }
    let Prompt::Chest(block) = *prompt else {
        return;
    };
    let key = [block.x, block.y, block.z];
    chests.0.entry(key).or_insert_with(|| {
        let loot = ruin_loot(block.x, block.z);
        slots_from(&loot, loot.len())
    });
    open.0 = Some(key);
    next.pending = Some(Screen::Chest);
}

pub fn act_on_the_prompt(
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    prompt: Res<Prompt>,
    hero: Res<Hero>,
    mut inventory: ResMut<Inventory>,
    mut world: ResMut<VoxelWorld>,
    mut notices: EventWriter<Notice>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut crops: Query<(&CropKind,)>,
    mut bodies: Query<(&Yaw, &mut Velocity, &mut LocalTransform, Option<&Riding>)>,
) {
    if !window.cursor_locked || !actions.just_pressed(Ask::Act) {
        return;
    }
    let Some(sailor) = hero.0 else {
        return;
    };
    match prompt.clone() {
        Prompt::None => {}
        Prompt::Harvest(entity) => {
            let mut kind = None;
            crops.for_entity(entity, |(crop,)| kind = Some(*crop));
            if let Some(kind) = kind {
                let leftover = inventory_add(&mut inventory, Item::Crop(kind), 1);
                let seeds = inventory_add(&mut inventory, Item::Seed(kind), 1);
                if leftover == 0 {
                    commands.despawn(entity);
                    let message = if seeds == 0 {
                        format!("Picked a {} and kept a seed", kind.info().name)
                    } else {
                        format!("Picked a {}", kind.info().name)
                    };
                    notices.send(Notice(message));
                } else {
                    notices.send(Notice("No room in the hotbar".into()));
                }
            }
        }
        Prompt::Door(block, _) => swing_the_door(&mut world, block),
        Prompt::Board(boat) => commands.set(sailor, Riding(boat)),
        Prompt::Leave => {
            bodies.for_entity(sailor, |(_, velocity, transform, _)| {
                velocity.0 = Vec3::zeros();
                transform.translation.y += 1.0;
            });
            commands.remove::<(Riding,)>(sailor);
        }
        Prompt::Craft => next.pending = Some(Screen::Crafting),
        Prompt::Sign(text) => notices.send(Notice(text)),
        Prompt::Chest(_) | Prompt::Talk(_) => {}
    }
}
