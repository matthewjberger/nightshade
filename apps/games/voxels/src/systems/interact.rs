use crate::commands::{
    dialogue_start, inventory_add, inventory_swap_selected, inventory_take_selected,
    launch_firework, plant, select_next_slot, select_slot, spawn_a_boat, spawn_drop, toast_show,
    update_crack_mesh,
};
use crate::components::{Block, Boat, NpcName, Progress};
use crate::data::{Ask, BlockKind, CropKind, Item, NpcRole, dialogue_for};
use crate::queries::{
    block_at, break_seconds, crack_stage, dungeon_loot, facing_of, looks_at, player_box, raycast,
    raycast_to_water, selected_stack,
};
use crate::resources::{
    Aim, Chests, Dialogue, Fireworks, Framing, Generation, Hand, Inventory, Player, Prompt, Screen,
    Sheet, Toast, VoxelWorld,
};
use crate::theme;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_scene::prelude::{LocalTransform, Visibility};
use nightshade_state::prelude::*;
use nightshade_voxels::prelude::{place_block, set_block, swing_the_block};

pub fn aim(
    player: Res<Player>,
    mut world: ResMut<VoxelWorld>,
    mut chests: ResMut<Chests>,
    mut inventory: ResMut<Inventory>,
    mut aim: ResMut<Aim>,
    mut hand: ResMut<Hand>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    window: Res<Window>,
    mut commands: Commands,
) {
    let delta_time = time.delta_time.min(0.05);
    let locked = window.cursor_locked;

    if player.watching {
        aim.target = None;
        aim.mining = None;
        aim.progress = 0.0;
        if let Some(outline) = aim.outline
            && aim.outlined.is_some()
        {
            aim.outlined = None;
            commands.set(outline, Visibility { visible: false });
        }
        return;
    }

    for (index, ask) in Ask::SLOTS.iter().enumerate() {
        if locked && actions.just_pressed(*ask) {
            select_slot(&mut inventory, index);
        }
    }
    if locked {
        let wheel = input.mouse.wheel_delta.y;
        if wheel.abs() > 0.01 {
            select_next_slot(&mut inventory, if wheel > 0.0 { -1 } else { 1 });
        }
    }
    if actions.just_pressed(Ask::PreviousSlot) {
        select_next_slot(&mut inventory, -1);
    }
    if actions.just_pressed(Ask::NextSlot) {
        select_next_slot(&mut inventory, 1);
    }

    let target = raycast(&world, player.eye, player.forward, theme::REACH);
    aim.target = target;

    if let Some(outline) = aim.outline {
        match target {
            Some(hit) if aim.outlined != Some(hit.block) => {
                aim.outlined = Some(hit.block);
                commands.set(
                    outline,
                    LocalTransform::from_translation(block_corner(hit.block)),
                );
                commands.set(outline, Visibility { visible: true });
            }
            None if aim.outlined.is_some() => {
                aim.outlined = None;
                commands.set(outline, Visibility { visible: false });
            }
            _ => {}
        }
    }

    let held = selected_stack(&inventory).map(|stack| stack.item);
    let mining = locked && actions.pressed(Ask::Mine) && !actions.just_pressed(Ask::Mine);
    if mining {
        hand.swing += delta_time;
    }

    match (mining, target) {
        (true, Some(hit)) => {
            if aim.mining != Some(hit.block) {
                aim.mining = Some(hit.block);
                aim.progress = 0.0;
            }
            let seconds = break_seconds(BlockKind::from_index(hit.kind), held);
            if seconds <= 0.0 {
                aim.progress = 1.0;
            } else if seconds.is_finite() {
                aim.progress += delta_time / seconds;
            }
            if aim.progress >= 1.0 {
                break_block(
                    &mut world,
                    &mut chests,
                    &mut commands,
                    hit.block,
                    BlockKind::from_index(hit.kind),
                    time.uptime_seconds as f32,
                );
                aim.mining = None;
                aim.progress = 0.0;
            }
        }
        _ => {
            aim.mining = None;
            aim.progress = 0.0;
        }
    }
}

pub fn paint_the_crack(
    world: Res<VoxelWorld>,
    mut aim: ResMut<Aim>,
    mut mesh_cache: ResMut<MeshCache>,
    mut mesh_state: ResMut<MeshRenderState>,
    mut commands: Commands,
) {
    let Some(crack) = aim.crack else {
        return;
    };
    match aim.mining {
        Some(block) if aim.progress > 0.0 && aim.progress < 1.0 => {
            let stage = crack_stage(aim.progress);
            if aim.stage != Some(stage) {
                aim.stage = Some(stage);
                update_crack_mesh(&mut mesh_cache, &world.blocks, stage);
            }
            if !aim.crack_shown {
                aim.crack_shown = true;
                commands.set(crack, Visibility { visible: true });
                mesh_state.full_rebuild_needed = true;
            }
            commands.set(crack, LocalTransform::from_translation(block_corner(block)));
        }
        _ if aim.crack_shown => {
            aim.crack_shown = false;
            aim.stage = None;
            commands.set(crack, Visibility { visible: false });
        }
        _ => {}
    }
}

pub fn use_item(
    mut player: ResMut<Player>,
    mut world: ResMut<VoxelWorld>,
    mut inventory: ResMut<Inventory>,
    aim: Res<Aim>,
    mut hand: ResMut<Hand>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    sheet: Res<Sheet>,
    mut fireworks: ResMut<Fireworks>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    mut crops: Query<(&Block,)>,
) {
    if player.watching || !window.cursor_locked {
        return;
    }
    let held = selected_stack(&inventory).map(|stack| stack.item);
    let phase = world.edits as f32 * 0.37;
    let player_food = player.food;
    let mut eaten = None;

    let placing = actions.just_pressed(Ask::Place);
    if placing {
        hand.swing = hand.swing.max(0.01);
        let wet =
            raycast_to_water(&world, player.eye, player.forward, theme::REACH).filter(|hit| {
                world
                    .blocks
                    .fill(hit.kind)
                    .is_some_and(|fill| fill.liquid == theme::WATER)
            });
        match (aim.target, held, wet) {
            (Some(hit), Some(Item::Block(kind)), _) => {
                let position = hit.block + hit.normal;
                let wanted = if kind == BlockKind::Torch {
                    BlockKind::wall_torch(hit.normal)
                } else if world.blocks.tall(kind.index()) {
                    Some(BlockKind::door_kind(facing_of(player.yaw), false))
                } else {
                    Some(kind)
                };
                if let Some(kind) = wanted
                    && place_block(
                        &mut world,
                        position,
                        kind.index(),
                        Some(player_box(player.position)),
                    )
                {
                    inventory_take_selected(&mut inventory, 1);
                }
            }
            (Some(hit), Some(Item::Seed(kind)), _) => {
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
                    toast_show(&mut toast, "Seeds want open dirt or grass");
                }
            }
            (_, Some(Item::Crop(kind)), _) if player_food < theme::FULL_FOOD => {
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
                    spawn_a_boat(&mut commands, place, player.yaw);
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
                toast_show(&mut toast, "The bucket wants water");
            }
            (Some(hit), Some(Item::Firework), _) => {
                let launch = block_corner(hit.block + hit.normal) + Vec3::repeat(0.5);
                if inventory_take_selected(&mut inventory, 1).is_some() {
                    launch_firework(&mut commands, &mut fireworks, launch);
                }
            }
            (None, Some(Item::Firework), _)
                if inventory_take_selected(&mut inventory, 1).is_some() =>
            {
                launch_firework(&mut commands, &mut fireworks, player.eye + player.forward);
            }
            _ => {}
        }
    }
    if let Some(kind) = eaten {
        player.food = (player.food + kind.feeds()).min(theme::FULL_FOOD);
        toast_show(&mut toast, format!("Ate a {}", kind.info().name));
    }

    if actions.just_pressed(Ask::Drop)
        && let Some(stack) = inventory_take_selected(&mut inventory, 1)
    {
        let start = player.eye + player.forward * 0.4;
        let velocity = player.forward * 6.0 + Vec3::new(0.0, 2.0, 0.0);
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

pub fn find_the_prompt(
    player: Res<Player>,
    aim: Res<Aim>,
    generation: Res<Generation>,
    mut prompt: ResMut<Prompt>,
    mut people: Query<(&NpcName, &LocalTransform)>,
    mut crops: Query<(&Block, &Progress)>,
    mut boats: Query<(&Boat, &LocalTransform)>,
) {
    if player.watching {
        *prompt = Prompt::None;
        return;
    }
    let mut found = Prompt::None;
    let mut best = f32::MAX;
    people.for_each_mut(|entity, (name, transform)| {
        let chest = transform.translation + Vec3::new(0.0, 0.8, 0.0);
        let distance = (chest - player.eye).norm();
        if looks_at(
            player.eye,
            player.forward,
            chest,
            theme::TALK_DISTANCE,
            theme::TALK_COSINE,
        ) && distance < best
        {
            best = distance;
            found = Prompt::Talk(entity, name.0);
        }
    });
    if found == Prompt::None {
        crops.for_each_mut(|entity, (block, progress)| {
            let top = Vec3::new(
                block.0.x as f32 + 0.5,
                block.0.y as f32 + 1.4,
                block.0.z as f32 + 0.5,
            );
            if progress.0 >= 1.0
                && looks_at(
                    player.eye,
                    player.forward,
                    top,
                    theme::REACH,
                    theme::TALK_COSINE,
                )
            {
                found = Prompt::Harvest(entity);
            }
        });
    }
    if player.boat.is_some() {
        *prompt = Prompt::Leave;
        return;
    }
    if found == Prompt::None {
        let reach = theme::BOAT_REACH;
        let place = player.position;
        boats.for_each_mut(|entity, (_, transform)| {
            let near = (transform.translation - place).norm() < reach;
            let seen = looks_at(
                player.eye,
                player.forward,
                transform.translation,
                theme::REACH,
                theme::TALK_COSINE,
            );
            if found == Prompt::None && (near || seen) {
                found = Prompt::Board(entity);
            }
        });
    }
    if found == Prompt::None
        && let Some(hit) = aim.target
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
                    .unwrap_or_else(|| crate::data::SIGN_TEXT.to_string()),
            ),
            _ => Prompt::None,
        };
    }
    *prompt = found;
}

pub fn act_on_the_prompt(
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    prompt: Res<Prompt>,
    generation: Res<Generation>,
    mut inventory: ResMut<Inventory>,
    mut world: ResMut<VoxelWorld>,
    mut player: ResMut<Player>,
    mut chests: ResMut<Chests>,
    mut toast: ResMut<Toast>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut crops: Query<(&CropKind,)>,
) {
    if !window.cursor_locked || !actions.just_pressed(Ask::Act) {
        return;
    }
    match prompt.clone() {
        Prompt::None => {}
        Prompt::Talk(entity, name) => {
            let coins = inventory.coins;
            commands.queue(move |storage, resources| {
                let Some(role) = storage.get::<NpcRole>(entity) else {
                    return;
                };
                let (lines, after) = dialogue_for(*role, coins);
                let base = storage
                    .get::<LocalTransform>(entity)
                    .map(|transform| transform.translation)
                    .unwrap_or_default();
                dialogue_start(resources.res_mut::<Dialogue>(), name, lines, after);
                let framing = resources.res_mut::<Framing>();
                framing.subject = Some(entity);
                framing.base = base;
                framing.phase = 0.0;
                framing.talking = true;
            });
            next.pending = Some(Screen::Talking);
        }
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
                    toast_show(&mut toast, message);
                } else {
                    toast_show(&mut toast, "No room in the hotbar");
                }
            }
        }
        Prompt::Door(block, _) => swing_the_block(&mut world, block),
        Prompt::Board(boat) => player.boat = Some(boat),
        Prompt::Leave => {
            player.boat = None;
            player.velocity = Vec3::zeros();
            player.position.y += 1.0;
        }
        Prompt::Craft => next.pending = Some(Screen::Crafting),
        Prompt::Chest(block) => {
            let key = [block.x, block.y, block.z];
            let seed = generation.seed;
            chests.slots.entry(key).or_insert_with(|| {
                dungeon_loot(seed, block).unwrap_or_else(|| vec![None; theme::CHEST_SLOTS])
            });
            chests.open = Some(key);
            next.pending = Some(Screen::Chest);
        }
        Prompt::Sign(text) => toast_show(&mut toast, text),
    }
}

fn block_corner(block: IVec3) -> Vec3 {
    Vec3::new(block.x as f32, block.y as f32, block.z as f32)
}

fn break_block(
    world: &mut VoxelWorld,
    chests: &mut Chests,
    commands: &mut Commands,
    block: IVec3,
    kind: BlockKind,
    phase: f32,
) {
    set_block(world, block, BlockKind::Air.index());
    if kind == BlockKind::Chest
        && let Some(slots) = chests.slots.remove(&[block.x, block.y, block.z])
    {
        let center = block_corner(block) + Vec3::repeat(0.5);
        for stack in slots.into_iter().flatten() {
            spawn_drop(
                commands,
                stack.item,
                stack.count,
                center,
                Vec3::new((phase * 3.7).sin() * 1.5, 3.0, (phase * 2.9).cos() * 1.5),
                phase,
            );
        }
    }
    let above = block + IVec3::new(0, 1, 0);
    if world.blocks.support(block_at(world, above).index()) == Some([0, -1, 0]) {
        set_block(world, above, BlockKind::Air.index());
    }
    if let Some(drop) = kind.drop() {
        let center = block_corner(block) + Vec3::repeat(0.5);
        spawn_drop(
            commands,
            Item::Block(drop),
            1,
            center,
            Vec3::new((phase * 7.3).sin() * 1.5, 3.0, (phase * 5.1).cos() * 1.5),
            phase,
        );
    }
}
