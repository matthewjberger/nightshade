use crate::components::{GlowBlock, SignLabel};
use crate::data::world::SIGN_TEXT;
use crate::resources::{Effects, Generation};
use blocker_blocks::data::blocks::{BlockKind, Shape};
use blocker_blocks::queries::column_of;
use blocker_blocks::resources::{VoxelViewer, VoxelWorld};
use nalgebra_glm::{Vec3, vec4};
use nightshade::prelude::*;
use nightshade_graphics::commands::light::spawn_light_entity;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_render::components::light::{Light, LightType};
use nightshade_text::prelude::*;
use std::collections::{HashMap, HashSet};

pub fn follow_the_blocks(
    world: Res<VoxelWorld>,
    viewer: Res<VoxelViewer>,
    mut effects: ResMut<Effects>,
    generation: Res<Generation>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
    mut attached: Query<(&GlowBlock, &BlockKind)>,
    mut labels: Query<(&SignLabel,)>,
) {
    let center = column_of(viewer.eye.x.floor() as i32, viewer.eye.z.floor() as i32);
    if effects.synced_edits == Some(world.edits) && effects.center == Some(center) {
        return;
    }
    effects.synced_edits = Some(world.edits);
    effects.center = Some(center);

    let mut glowing: HashMap<[i32; 3], BlockKind> = HashMap::new();
    let mut posts = HashSet::new();
    for (key, markers) in &world.markers {
        if (key[0] - center[0]).abs() > crate::theme::GLOW_RADIUS
            || (key[2] - center[1]).abs() > crate::theme::GLOW_RADIUS
        {
            continue;
        }
        for (block, held) in markers {
            let kind = BlockKind::from_index(*held);
            if kind == BlockKind::Signpost {
                posts.insert(*block);
            } else {
                glowing.insert(*block, kind);
            }
        }
    }

    let mut gone = Vec::new();
    let mut kept = HashSet::new();
    attached.for_each_mut(|entity, (glow, kind)| {
        if glowing.get(&glow.0) == Some(kind) {
            kept.insert(glow.0);
        } else {
            gone.push(entity);
        }
    });
    let mut labeled = HashSet::new();
    labels.for_each_mut(|entity, (label,)| {
        if posts.contains(&label.0) {
            labeled.insert(label.0);
        } else {
            gone.push(entity);
        }
    });
    for entity in gone {
        commands.despawn(entity);
    }

    for (block, kind) in glowing {
        if !kept.contains(&block) {
            spawn_glow(&mut commands, block, kind);
        }
    }
    for block in posts {
        if labeled.contains(&block) {
            continue;
        }
        let text = generation
            .signs
            .get(&block)
            .cloned()
            .unwrap_or_else(|| SIGN_TEXT.to_string());
        let label = spawn_3d_billboard_text_with_properties(
            &mut text_state,
            &mut commands,
            &text,
            Vec3::new(
                block[0] as f32 + 0.5,
                block[1] as f32 + 1.35,
                block[2] as f32 + 0.5,
            ),
            TextProperties {
                font_size: 20.0,
                color: vec4(1.0, 0.95, 0.8, 0.95),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 1.5,
                outline_color: vec4(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        commands.set(label, SignLabel(block));
    }
}

fn spawn_glow(commands: &mut Commands, block: [i32; 3], kind: BlockKind) {
    let Some((color, intensity, range)) = kind.glow() else {
        return;
    };
    let corner = Vec3::new(block[0] as f32, block[1] as f32, block[2] as f32);
    let torch = kind.info().shape == Shape::Parts;
    let flame_height = if torch {
        blocker_blocks::data::blocks::TORCH_FLAME_HEIGHT
    } else {
        1.0
    };
    let rise = if kind.on_wall() {
        blocker_blocks::data::blocks::TORCH_WALL_RISE
    } else {
        0.0
    };
    let lean = kind.lean();
    let center = corner
        + Vec3::new(0.5, flame_height + rise, 0.5)
        + lean
            * (blocker_blocks::data::blocks::TORCH_WALL_TILT
                - blocker_blocks::data::blocks::TORCH_WALL_SHIFT);

    let flames = commands.spawn(());
    let mut fire = ParticleEmitter::fire(center);
    match kind {
        _ if torch => {
            fire.spawn_rate = 14.0;
            fire.size_start = 0.09;
            fire.size_end = 0.02;
            fire.initial_velocity_min = 0.2;
            fire.initial_velocity_max = 0.5;
            fire.particle_lifetime_min = 0.2;
            fire.particle_lifetime_max = 0.5;
            fire.emissive_strength = 4.0;
        }
        _ => {
            fire.spawn_rate = 45.0;
            fire.size_start = 0.22;
            fire.size_end = 0.05;
            fire.initial_velocity_min = 0.8;
            fire.initial_velocity_max = 1.8;
            fire.particle_lifetime_min = 0.4;
            fire.particle_lifetime_max = 0.9;
            fire.emissive_strength = 5.0;
        }
    }
    commands.set(flames, fire);
    commands.set(flames, GlowBlock(block));
    commands.set(flames, kind);

    if !torch {
        let plume = commands.spawn(());
        let mut smoke = ParticleEmitter::smoke(center + Vec3::new(0.0, 0.6, 0.0));
        smoke.spawn_rate = 5.0;
        smoke.size_start = 0.2;
        smoke.size_end = 0.9;
        smoke.initial_velocity_min = 0.4;
        smoke.initial_velocity_max = 0.8;
        smoke.particle_lifetime_min = 2.5;
        smoke.particle_lifetime_max = 4.5;
        smoke.velocity_spread = 0.25;
        smoke.turbulence_strength = 0.4;
        commands.set(plume, smoke);
        commands.set(plume, GlowBlock(block));
        commands.set(plume, kind);
    }

    let light = spawn_light_entity(
        commands,
        center + Vec3::new(0.0, 0.25, 0.0),
        kind.info().name,
    );
    commands.set(
        light,
        Light {
            light_type: LightType::Point,
            color,
            intensity,
            range,
            ..Default::default()
        },
    );
    commands.set(light, GlowBlock(block));
    commands.set(light, kind);
}
