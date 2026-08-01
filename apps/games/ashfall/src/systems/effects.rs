use crate::data::theme;
use crate::resources::{AshfallResources, Senses, WorldPhase};
use crate::tags::{AshCloud, LevelPiece, Player};
use nightshade::prelude::*;
use nightshade_decals::contract::Decal;
use nightshade_particles::contract::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter, ParticleTextureUpload,
    ParticleTextures,
};
use nightshade_scene::contract::Lifetime;
use nightshade_scene::prelude::*;

const SPRITE_SIZE: u32 = 512;

pub fn upload_the_ash_sprite(mut textures: ResMut<ParticleTextures>) {
    let middle = SPRITE_SIZE as f32 * 0.5;
    let mut pixels = Vec::with_capacity((SPRITE_SIZE * SPRITE_SIZE * 4) as usize);
    for row in 0..SPRITE_SIZE {
        for col in 0..SPRITE_SIZE {
            let reach = ((col as f32 - middle).hypot(row as f32 - middle)) / middle;
            let alpha = (1.0 - reach).clamp(0.0, 1.0).powf(1.6);
            pixels.extend_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }
    textures.pending.push(ParticleTextureUpload {
        slot: 0,
        rgba_data: pixels,
        width: SPRITE_SIZE,
        height: SPRITE_SIZE,
    });
}

pub fn spawn_ash_cloud(commands: &mut Commands, at: Vec3, phase: WorldPhase) {
    let entity = commands.spawn((Name("Ashfall".to_string()), ash_emitter(at, phase)));
    commands.add_tag_type::<AshCloud>(entity);
    commands.add_tag_type::<LevelPiece>(entity);
}

fn ash_emitter(at: Vec3, phase: WorldPhase) -> ParticleEmitter {
    ParticleEmitter {
        emitter_type: EmitterType::Smoke,
        shape: EmitterShape::Box {
            half_extents: theme::ASH_HALF_EXTENTS,
        },
        position: at + Vec3::new(0.0, theme::ASH_HEIGHT, 0.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
        spawn_rate: theme::ASH_RATE,
        burst_count: 0,
        particle_lifetime_min: theme::ASH_LIFE.0,
        particle_lifetime_max: theme::ASH_LIFE.1,
        initial_velocity_min: 0.1,
        initial_velocity_max: 0.4,
        velocity_spread: 0.6,
        gravity: Vec3::new(0.0, theme::ASH_FALL, 0.0),
        drag: 0.5,
        size_start: theme::ASH_SIZE_START,
        size_end: theme::ASH_SIZE_END,
        color_gradient: ash_gradient(phase),
        emissive_strength: if phase == WorldPhase::Otherworld {
            2.5
        } else {
            0.0
        },
        turbulence_strength: 0.5,
        turbulence_frequency: 0.4,
        ..Default::default()
    }
}

fn ash_gradient(phase: WorldPhase) -> ColorGradient {
    let color = match phase {
        WorldPhase::Fog => theme::ASH_COLOR,
        WorldPhase::Otherworld => theme::EMBER_COLOR,
    };
    ColorGradient {
        colors: vec![
            (0.0, Vec4::new(color.x, color.y, color.z, 0.0)),
            (0.25, color),
            (0.85, color),
            (1.0, Vec4::new(color.x, color.y, color.z, 0.0)),
        ],
    }
}

pub fn drift(
    resources: Res<AshfallResources>,
    mut players: Query<(&LocalTransform,), WithTag<Player>>,
    clouds: Query<(&mut ParticleEmitter,), WithTag<AshCloud>>,
) {
    let Some(body) = resources.player.body else {
        return;
    };
    let mut here = None;
    players.for_entity(body, |(transform,)| here = Some(transform.translation));
    let Some(here) = here else {
        return;
    };
    let phase = resources.run.phase;
    let falling = resources.screen.is_in_town();
    clouds.for_each(|_entity, (emitter,)| {
        emitter.position = here + Vec3::new(0.0, theme::ASH_HEIGHT, 0.0);
        emitter.enabled = falling;
        emitter.color_gradient = ash_gradient(phase);
        emitter.emissive_strength = if phase == WorldPhase::Otherworld {
            2.5
        } else {
            0.0
        };
    });
}

pub fn shatter(commands: &mut Commands, at: Vec3) {
    let entity = commands.spawn((
        Name("Broken Glass".to_string()),
        Lifetime {
            remaining: theme::PUFF_LIFE,
        },
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.12 },
            position: at,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: theme::SHARD_COUNT,
            one_shot: true,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.4,
            initial_velocity_min: 1.2,
            initial_velocity_max: 4.0,
            velocity_spread: std::f32::consts::PI,
            gravity: Vec3::new(0.0, -9.0, 0.0),
            drag: 0.4,
            size_start: 0.045,
            size_end: 0.02,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, theme::SHARD_COLOR),
                    (0.7, theme::SHARD_COLOR),
                    (
                        1.0,
                        Vec4::new(
                            theme::SHARD_COLOR.x,
                            theme::SHARD_COLOR.y,
                            theme::SHARD_COLOR.z,
                            0.0,
                        ),
                    ),
                ],
            },
            emissive_strength: 1.6,
            ..Default::default()
        },
    ));
    commands.add_tag_type::<LevelPiece>(entity);
}

pub fn stain_the_ground(mut senses: ResMut<Senses>, mut commands: Commands) {
    for at in std::mem::take(&mut senses.stains) {
        spawn_stain(&mut commands, at, theme::STAIN_SIZE, theme::STAIN_COLOR);
    }
}

pub fn spawn_stain(commands: &mut Commands, at: Vec3, size: f32, color: [f32; 4]) {
    let floor = Vec3::new(at.x, (at.y - 1.0).max(0.05), at.z);
    let entity = commands.spawn((
        Name("Stain".to_string()),
        LocalTransform {
            translation: floor,
            rotation: facing_up(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Decal::new(theme::STAIN_TEXTURE)
            .with_size(size, size)
            .with_color(color)
            .with_depth(size)
            .with_normal_threshold(0.2),
    ));
    commands.add_tag_type::<LevelPiece>(entity);
}

fn facing_up() -> Quat {
    nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x())
}
