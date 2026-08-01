use crate::commands::spawn_drop;
use crate::components::Motion;
use crate::data::ask::Ask;
use crate::resources::{Aim, Markers, Mining};
use blocker_blocks::commands::update_crack_mesh;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::items::Item;
use blocker_blocks::queries::{block_at, break_seconds, crack_stage, raycast};
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_pocket::queries::selected_stack;
use blocker_pocket::queries::stacks_of;
use blocker_pocket::resources::{Chests, Inventory};
use blocker_world::queries::ruin_loot;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_scene::prelude::{LocalTransform, Visibility};
use nightshade_voxels::prelude::set_block;

pub mod prompt;
pub mod slots;
pub mod tools;

pub fn aim(
    mut world: ResMut<VoxelWorld>,
    mut chests: ResMut<Chests>,
    inventory: Res<Inventory>,
    mut aim: ResMut<Aim>,
    mut mining: ResMut<Mining>,
    mut markers: ResMut<Markers>,
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    window: Res<Window>,
    hero: Res<Hero>,
    mut commands: Commands,
    mut bodies: Query<(&Motion, &Eye, &Gaze)>,
) {
    let Some((motion, eye, gaze)) = read_the_hero(&hero, &mut bodies, |(motion, eye, gaze)| {
        (*motion, eye.0, gaze.0)
    }) else {
        return;
    };
    let delta_time = time.delta_time.min(0.05);
    let locked = window.cursor_locked;

    if motion == Motion::Watch {
        aim.0 = None;
        mining.block = None;
        mining.progress = 0.0;
        if let Some(outline) = markers.outline
            && markers.outlined.is_some()
        {
            markers.outlined = None;
            commands.set(outline, Visibility { visible: false });
        }
        return;
    }

    let target = raycast(&world, eye, gaze, blocker_figure::data::sight::REACH);
    aim.0 = target;

    if let Some(outline) = markers.outline {
        match target {
            Some(hit) if markers.outlined != Some(hit.block) => {
                markers.outlined = Some(hit.block);
                commands.set(
                    outline,
                    LocalTransform::from_translation(block_corner(hit.block)),
                );
                commands.set(outline, Visibility { visible: true });
            }
            None if markers.outlined.is_some() => {
                markers.outlined = None;
                commands.set(outline, Visibility { visible: false });
            }
            _ => {}
        }
    }

    let held = selected_stack(&inventory).map(|stack| stack.item);
    let digging = locked && actions.pressed(Ask::Mine) && !actions.just_pressed(Ask::Mine);

    match (digging, target) {
        (true, Some(hit)) => {
            if mining.block != Some(hit.block) {
                mining.block = Some(hit.block);
                mining.progress = 0.0;
            }
            let seconds = break_seconds(BlockKind::from_index(hit.kind), held);
            if seconds <= 0.0 {
                mining.progress = 1.0;
            } else if seconds.is_finite() {
                mining.progress += delta_time / seconds;
            }
            if mining.progress >= 1.0 {
                break_block(
                    &mut world,
                    &mut chests,
                    &mut commands,
                    hit.block,
                    BlockKind::from_index(hit.kind),
                    time.uptime_seconds as f32,
                );
                mining.block = None;
                mining.progress = 0.0;
            }
        }
        _ => {
            mining.block = None;
            mining.progress = 0.0;
        }
    }
}

pub fn paint_the_crack(
    world: Res<VoxelWorld>,
    mut mining: ResMut<Mining>,
    mut markers: ResMut<Markers>,
    mut mesh_cache: ResMut<MeshCache>,
    mut mesh_state: ResMut<MeshRenderState>,
    mut commands: Commands,
) {
    let Some(crack) = markers.crack else {
        return;
    };
    match mining.block {
        Some(block) if mining.progress > 0.0 && mining.progress < 1.0 => {
            let stage = crack_stage(mining.progress);
            if mining.stage != Some(stage) {
                mining.stage = Some(stage);
                update_crack_mesh(&mut mesh_cache, &world.blocks, stage);
            }
            if !markers.crack_shown {
                markers.crack_shown = true;
                commands.set(crack, Visibility { visible: true });
                mesh_state.full_rebuild_needed = true;
            }
            commands.set(crack, LocalTransform::from_translation(block_corner(block)));
        }
        _ if markers.crack_shown => {
            markers.crack_shown = false;
            mining.stage = None;
            commands.set(crack, Visibility { visible: false });
        }
        _ => {}
    }
}

pub(super) fn block_corner(block: IVec3) -> Vec3 {
    Vec3::new(block.x as f32, block.y as f32, block.z as f32)
}

pub(super) fn break_block(
    world: &mut VoxelWorld,
    chests: &mut Chests,
    commands: &mut Commands,
    block: IVec3,
    kind: BlockKind,
    phase: f32,
) {
    set_block(world, block, BlockKind::Air.index());
    if kind == BlockKind::Chest {
        let slots = chests
            .0
            .remove(&[block.x, block.y, block.z])
            .map(|slots| stacks_of(&slots))
            .unwrap_or_else(|| ruin_loot(block.x, block.z));
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
