use crate::commands::chat::system_line;
use crate::commands::{
    inventory_add, inventory_swap_selected, inventory_take_selected, launch_firework, plant,
    record_hit, select_next_slot, spawn_drop, toast_show, update_crack_mesh,
};
use crate::components::{Block, Cooldowns, Giver, Heading, Health, MaxHealth, Progress, Strike};
use crate::data::{Ask, BlockKind, CropKind, GIVERS, Item};
use crate::queries::{
    block_at, break_seconds, chest_loot, crack_stage, facing_of, flat_distance, hero_box, raycast,
    raycast_to_water, selected_stack,
};
use crate::resources::{
    Aim, Chests, Fireworks, Generation, HitSource, Hits, Hover, Inventory, Journal, Player, Prompt,
    Screen, Sheet, Toast, VoxelWorld,
};
use crate::theme;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::prelude::*;
use nightshade_camera::prelude::{ActiveCamera, Camera};
use nightshade_input::Actions;
use nightshade_picking::PickingRay;
use nightshade_picking::queries::{CameraView, ScreenArea};
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::prelude::*;
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Visibility};
use nightshade_state::prelude::*;
use nightshade_voxels::prelude::{place_block, set_block, swing_the_block};

pub fn aim(
    input: Res<Input>,
    capture: Res<InputCapture>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    active_camera: Res<ActiveCamera>,
    hover: Res<Hover>,
    player: Res<Player>,
    world: Res<VoxelWorld>,
    mut aim: ResMut<Aim>,
    mut commands: Commands,
    mut cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let cursor = input.mouse.position;
    let mut view = None;
    if let Some(active) = active_camera.0 {
        cameras.for_entity(active, |(camera, global)| {
            view = Some(CameraView::new(camera, global));
        });
    }
    let ray = if pointer_captured(&capture) || hover.entity.is_some() {
        None
    } else {
        view.and_then(|view| {
            let area = ScreenArea::new(&window, &viewport)?;
            PickingRay::from_screen_position(area, view, cursor)
        })
    };
    let target = ray.and_then(|ray| {
        let to_player = (player.position - ray.origin).magnitude();
        let hit = raycast(&world, ray.origin, ray.direction, to_player + theme::REACH)?;
        let center = Vec3::new(
            hit.block.x as f32 + 0.5,
            hit.block.y as f32 + 0.5,
            hit.block.z as f32 + 0.5,
        );
        ((center - player.position - Vec3::new(0.0, 1.0, 0.0)).magnitude() <= theme::REACH)
            .then_some(hit)
    });
    aim.target = target;
    aim.over_blocks = target.is_some();

    let Some(outline) = aim.outline else {
        return;
    };
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

pub fn mine(
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    player: Res<Player>,
    inventory: Res<Inventory>,
    mut world: ResMut<VoxelWorld>,
    mut chests: ResMut<Chests>,
    mut aim: ResMut<Aim>,
    mut commands: Commands,
    mut heroes: Query<(&mut Heading, &mut Strike), With<Cooldowns>>,
) {
    let delta_time = time.delta_time.min(0.05);
    let mining = actions.pressed(Ask::Mine);
    let Some(hit) = aim.target.filter(|_| mining) else {
        aim.mining = None;
        aim.progress = 0.0;
        return;
    };
    if aim.mining != Some(hit.block) {
        aim.mining = Some(hit.block);
        aim.progress = 0.0;
    }
    let center = Vec3::new(
        hit.block.x as f32 + 0.5,
        hit.block.y as f32 + 0.5,
        hit.block.z as f32 + 0.5,
    );
    heroes.for_each_mut(|_entity, (heading, strike)| {
        heading.0 = (player.position.x - center.x).atan2(player.position.z - center.z);
        strike.0 = strike.0.max(theme::STRIKE_SECONDS * 0.5);
    });
    let held = selected_stack(&inventory).map(|stack| stack.item);
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
    input: Res<Input>,
    capture: Res<InputCapture>,
    actions: Res<Actions<Ask>>,
    player: Res<Player>,
    sheet: Res<Sheet>,
    mut world: ResMut<VoxelWorld>,
    mut inventory: ResMut<Inventory>,
    mut aim: ResMut<Aim>,
    mut hits: ResMut<Hits>,
    mut fireworks: ResMut<Fireworks>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    mut crops: Query<(&Block,)>,
    mut vitals: Query<(&Health, &MaxHealth), With<Cooldowns>>,
) {
    if pointer_captured(&capture) {
        aim.right_press = None;
        return;
    }
    let cursor = input.mouse.position;
    if input.mouse.state.contains(MouseState::RIGHT_JUST_PRESSED) {
        aim.right_press = Some(cursor);
    }
    let placing = input.mouse.state.contains(MouseState::RIGHT_JUST_RELEASED)
        && aim
            .right_press
            .take()
            .is_some_and(|pressed| (pressed - cursor).magnitude() < 6.0);
    let dropping = actions.just_pressed(Ask::Drop);
    if !placing && !dropping {
        return;
    }
    let held = selected_stack(&inventory).map(|stack| stack.item);
    let phase = world.edits as f32 * 0.37;
    let face = facing_of(player.yaw);

    if placing {
        let wet = raycast_to_water(&world, player.eye, player.forward, theme::REACH * 2.0).filter(
            |hit| {
                world
                    .blocks
                    .fill(hit.kind)
                    .is_some_and(|fill| fill.liquid == theme::WATER)
                    && flat_distance(
                        Vec3::new(hit.block.x as f32, 0.0, hit.block.z as f32),
                        player.position,
                    ) <= theme::REACH
            },
        );
        match (aim.target, held, wet) {
            (Some(hit), Some(Item::Block(kind)), _) => {
                let position = hit.block + hit.normal;
                let wanted = if kind == BlockKind::Torch {
                    BlockKind::wall_torch(hit.normal)
                } else if world.blocks.tall(kind.index()) {
                    Some(BlockKind::door_kind(face, false))
                } else {
                    Some(kind)
                };
                if let Some(kind) = wanted
                    && place_block(
                        &mut world,
                        position,
                        kind.index(),
                        Some(hero_box(player.position)),
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
            (_, Some(Item::Crop(kind)), _) => {
                let mut hungry = None;
                vitals.for_each_mut(|entity, (health, max_health)| {
                    if health.0 < max_health.0 {
                        hungry = Some(entity);
                    }
                });
                if let Some(body) = hungry
                    && inventory_take_selected(&mut inventory, 1).is_some()
                {
                    let mending = match kind {
                        CropKind::Carrot => 25.0,
                        CropKind::Wheat => 18.0,
                    };
                    record_hit(
                        &mut hits,
                        body,
                        body,
                        -mending,
                        false,
                        false,
                        HitSource::Heal,
                    );
                    toast_show(&mut toast, format!("Ate a {}", kind.info().name));
                } else if hungry.is_none() {
                    toast_show(&mut toast, "You are not hurt");
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
                launch_firework(
                    &mut commands,
                    &mut fireworks,
                    player.position + Vec3::new(0.0, 1.6, 0.0),
                );
            }
            _ => {}
        }
    }

    if dropping && let Some(stack) = inventory_take_selected(&mut inventory, 1) {
        let forward = Vec3::new(-player.yaw.sin(), 0.0, -player.yaw.cos());
        let start = player.position + Vec3::new(0.0, 1.3, 0.0) + forward * 0.6;
        let velocity = forward * 5.0 + Vec3::new(0.0, 2.5, 0.0);
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

pub fn choose_slot(actions: Res<Actions<Ask>>, mut inventory: ResMut<Inventory>) {
    if actions.just_pressed(Ask::PreviousSlot) {
        select_next_slot(&mut inventory, -1);
    }
    if actions.just_pressed(Ask::NextSlot) {
        select_next_slot(&mut inventory, 1);
    }
}

pub fn find_the_prompt(
    player: Res<Player>,
    aim: Res<Aim>,
    generation: Res<Generation>,
    mut prompt: ResMut<Prompt>,
    mut crops: Query<(&Block, &Progress)>,
    givers: Query<(&Giver, &LocalTransform)>,
) {
    let mut found = Prompt::None;
    let mut best = f32::MAX;
    givers.for_each(|_entity, (giver, transform)| {
        if !GIVERS[giver.0].shop {
            return;
        }
        let distance = flat_distance(transform.translation, player.position);
        if distance < theme::GIVER_REACH && distance < best {
            best = distance;
            found = Prompt::Trade(giver.0);
        }
    });
    if found == Prompt::None {
        crops.for_each_mut(|entity, (block, progress)| {
            let top = Vec3::new(
                block.0.x as f32 + 0.5,
                block.0.y as f32 + 1.0,
                block.0.z as f32 + 0.5,
            );
            let distance = (top - player.position).magnitude();
            if progress.0 >= 1.0 && distance < theme::REACH && distance < best {
                best = distance;
                found = Prompt::Harvest(entity);
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
    prompt: Res<Prompt>,
    generation: Res<Generation>,
    mut journal: ResMut<Journal>,
    mut inventory: ResMut<Inventory>,
    mut world: ResMut<VoxelWorld>,
    mut chests: ResMut<Chests>,
    mut toast: ResMut<Toast>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut crops: Query<(&CropKind,)>,
) {
    if !actions.just_pressed(Ask::Interact) {
        return;
    }
    match prompt.clone() {
        Prompt::None => {}
        Prompt::Trade(giver) => {
            system_line(
                &mut journal.chat,
                format!("{} opens the ledger.", GIVERS[giver].name),
            );
            next.pending = Some(Screen::Shop);
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
                    toast_show(&mut toast, "No room in the pocket");
                }
            }
        }
        Prompt::Door(block, _) => swing_the_block(&mut world, block),
        Prompt::Craft => next.pending = Some(Screen::Crafting),
        Prompt::Chest(block) => {
            let key = [block.x, block.y, block.z];
            chests
                .slots
                .entry(key)
                .or_insert_with(|| chest_loot(&generation, block));
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
