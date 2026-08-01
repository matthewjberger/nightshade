use super::block_corner;
use crate::commands::{launch_firework, spawn_a_boat, spawn_drop};
use crate::components::{Food, Motion, Yaw};
use crate::data::ask::Ask;
use crate::queries::player_box;
use crate::resources::{Aim, Fireworks, Hand};
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::items::Item;
use blocker_blocks::queries::{block_at, facing_of, raycast_to_water};
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_pocket::commands::{inventory_swap_selected, inventory_take_selected};
use blocker_pocket::queries::selected_stack;
use blocker_pocket::resources::Inventory;
use blocker_screen::data::notice::Notice;
use nalgebra_glm::Vec3;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_random::Random;
use nightshade_scene::prelude::LocalTransform;
use nightshade_voxels::prelude::{place_block, set_block};

pub fn use_item(
    aim: Res<Aim>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    mut hand: ResMut<Hand>,
    mut fireworks: ResMut<Fireworks>,
    mut world: ResMut<VoxelWorld>,
    mut inventory: ResMut<Inventory>,
    hero: Res<Hero>,
    mut random: ResMut<Random>,
    mut notices: EventWriter<Notice>,
    mut commands: Commands,
    mut bodies: Query<(&mut Food, &Motion, &Eye, &Gaze, &Yaw, &LocalTransform)>,
) {
    let Some(body) = hero.0 else {
        return;
    };
    let Some((player_food, motion, eye, gaze, yaw, stood)) = read_the_hero(
        &hero,
        &mut bodies,
        |(food, motion, eye, gaze, yaw, transform)| {
            (food.0, *motion, eye.0, gaze.0, yaw.0, transform.translation)
        },
    ) else {
        return;
    };
    if motion == Motion::Watch || !window.cursor_locked {
        return;
    }
    let held = selected_stack(&inventory).map(|stack| stack.item);
    let phase = world.edits as f32 * 0.37;
    let mut eaten = None;

    let placing = actions.just_pressed(Ask::Place);
    if placing {
        hand.swing = hand.swing.max(0.01);
        let wet =
            raycast_to_water(&world, eye, gaze, blocker_figure::data::sight::REACH).filter(|hit| {
                world
                    .blocks
                    .fill(hit.kind)
                    .is_some_and(|fill| fill.liquid == blocker_blocks::data::blocks::WATER)
            });
        match (aim.0, held, wet) {
            (Some(hit), Some(Item::Block(kind)), _) => {
                let position = hit.block + hit.normal;
                let wanted = if kind == BlockKind::Torch {
                    BlockKind::wall_torch(hit.normal)
                } else if world.blocks.tall(kind.index()) {
                    Some(BlockKind::door_kind(facing_of(yaw), false))
                } else {
                    Some(kind)
                };
                if let Some(kind) = wanted
                    && place_block(&mut world, position, kind.index(), Some(player_box(stood)))
                {
                    inventory_take_selected(&mut inventory, 1);
                }
            }
            (_, Some(Item::Crop(kind)), _) if player_food < crate::data::body::FULL_FOOD => {
                if inventory_take_selected(&mut inventory, 1).is_some() {
                    eaten = Some(kind);
                }
            }
            (_, Some(Item::Boat), Some(hit)) => {
                let place = Vec3::new(
                    hit.block.x as f32 + 0.5,
                    hit.block.y as f32 + 1.0,
                    hit.block.z as f32 + 0.5,
                );
                if inventory_take_selected(&mut inventory, 1).is_some() {
                    spawn_a_boat(&mut commands, place, yaw);
                }
            }
            (_, Some(Item::Bucket), Some(hit)) => {
                set_block(&mut world, hit.block, BlockKind::Air.index());
                inventory_swap_selected(&mut inventory, Item::WaterBucket);
            }
            (Some(hit), Some(Item::WaterBucket), _) => {
                let position = hit.block + hit.normal;
                if world.blocks.replaceable(block_at(&world, position).index()) {
                    set_block(&mut world, position, BlockKind::Water.index());
                    inventory_swap_selected(&mut inventory, Item::Bucket);
                }
            }
            (Some(_), Some(Item::Bucket), None) => {
                notices.send(Notice("The bucket wants water".into()));
            }
            (Some(hit), Some(Item::Firework), _) => {
                let launch = block_corner(hit.block + hit.normal) + Vec3::repeat(0.5);
                if inventory_take_selected(&mut inventory, 1).is_some() {
                    launch_firework(&mut commands, &mut fireworks, &mut random, launch);
                }
            }
            (None, Some(Item::Firework), _)
                if inventory_take_selected(&mut inventory, 1).is_some() =>
            {
                launch_firework(&mut commands, &mut fireworks, &mut random, eye + gaze);
            }
            _ => {}
        }
    }
    if let Some(kind) = eaten {
        bodies.for_entity(body, |(food, _, _, _, _, _)| {
            food.0 = (food.0 + kind.feeds()).min(crate::data::body::FULL_FOOD);
        });
        notices.send(Notice(format!("Ate a {}", kind.info().name)));
    }

    if actions.just_pressed(Ask::Drop)
        && let Some(stack) = inventory_take_selected(&mut inventory, 1)
    {
        let start = eye + gaze * 0.4;
        let velocity = gaze * 6.0 + Vec3::new(0.0, 2.0, 0.0);
        spawn_drop(
            &mut commands,
            stack.item,
            stack.count,
            start,
            velocity,
            phase,
        );
    }
}
