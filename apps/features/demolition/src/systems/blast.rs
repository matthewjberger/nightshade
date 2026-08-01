use crate::data::Block;
use crate::resources::{Aim, Digging, VoxelSettings, VoxelViewer, VoxelWorld};
use crate::theme;
use nalgebra_glm::{IVec3, Quat, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::{LocalTransform, Visibility};
use nightshade_voxels::prelude::{AIR, reset_the_world, set_block};
use nightshade_voxels::queries::{block_at, in_blocks, in_world, raycast};

pub fn aim(
    input: Res<Input>,
    time: Res<Time>,
    window: Res<Window>,
    settings: Res<VoxelSettings>,
    viewer: Res<VoxelViewer>,
    world: Res<VoxelWorld>,
    mut aim: ResMut<Aim>,
    mut commands: Commands,
) {
    if window.cursor_locked {
        let wheel = input.mouse.wheel_delta.y;
        if wheel.abs() > 0.01 {
            aim.radius = (aim.radius + wheel.signum() as i32)
                .clamp(theme::BLAST_RADIUS_MIN, theme::BLAST_RADIUS_MAX);
        }
    }
    let origin = in_blocks(&settings, viewer.eye);
    let reach = theme::REACH / settings.block_size;
    aim.hit = raycast(&world, origin, viewer.forward, reach).map(|hit| hit.block);
    aim.pulse += time.delta_time * theme::AIM_PULSE_RATE;

    let Some(marker) = aim.marker else {
        return;
    };
    let shown = window.cursor_locked && aim.hit.is_some();
    if aim.marked != shown {
        aim.marked = shown;
        commands.set(marker, Visibility { visible: shown });
    }
    let Some(block) = aim.hit else {
        return;
    };
    let center = in_world(&settings, block.cast::<f32>() + Vec3::repeat(0.5));
    let pulse = 1.0 + theme::AIM_PULSE * aim.pulse.sin();
    let scale = (aim.radius as f32 + 0.5) * settings.block_size * pulse;
    commands.set(
        marker,
        LocalTransform {
            translation: center,
            rotation: Quat::identity(),
            scale: Vec3::repeat(scale),
        },
    );
}

pub fn blast(
    input: Res<Input>,
    window: Res<Window>,
    mut aim: ResMut<Aim>,
    mut world: ResMut<VoxelWorld>,
) {
    if !window.cursor_locked || !input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        return;
    }
    let Some(center) = aim.hit else {
        return;
    };
    let radius = aim.radius;
    let ground = Block::Ground.index();
    let mut taken = 0;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                if dx * dx + dy * dy + dz * dz > radius * radius {
                    continue;
                }
                let position = center + IVec3::new(dx, dy, dz);
                let block = block_at(&world, position);
                if block == AIR || block == ground {
                    continue;
                }
                set_block(&mut world, position, AIR);
                taken += 1;
            }
        }
    }
    aim.blasted += taken;
}

pub fn rebuild(
    input: Res<Input>,
    window: Res<Window>,
    mut aim: ResMut<Aim>,
    mut commands: Commands,
) {
    if !window.cursor_locked || !input.keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }
    aim.blasted = 0;
    aim.hit = None;
    commands.queue(|storage, resources| {
        reset_the_world::<Digging>(storage, resources);
    });
}
