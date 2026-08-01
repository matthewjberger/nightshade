use crate::components::{Burst, Fuse, Generation, SecondTint, Tint, Velocity};
use crate::data::ExplosionType;
use crate::queries;
use nightshade::prelude::*;
use nightshade_particles::contract::*;
use rand::Rng;

pub fn spawn_shaped_explosion(
    commands: &mut Commands<'_>,
    position: Vec3,
    color: Vec3,
    pattern: &[Vec2],
    scale: f32,
    particles_per_point: u32,
) {
    let mut generator = rand::rng();

    let max_emitters = 30;
    let step = (pattern.len() / max_emitters).max(1);

    for (index, point) in pattern.iter().enumerate() {
        if index % step != 0 {
            continue;
        }

        let point_magnitude = (point.x * point.x + point.y * point.y).sqrt();
        if point_magnitude < 0.001 {
            continue;
        }

        let direction = Vec3::new(point.x, point.y, 0.0).normalize();
        let speed = scale * point_magnitude * generator.random_range(0.9..1.1);

        let entity = commands.spawn(());

        let mut emitter = ParticleEmitter::firework_explosion(position, color, particles_per_point);
        emitter.burst_count = particles_per_point;
        emitter.spawn_rate = 0.0;
        emitter.one_shot = true;
        emitter.shape = EmitterShape::Point;
        emitter.direction = direction;
        emitter.initial_velocity_min = speed * 0.85;
        emitter.initial_velocity_max = speed * 1.15;
        emitter.velocity_spread = 0.15;
        emitter.particle_lifetime_min = 2.0;
        emitter.particle_lifetime_max = 3.0;
        emitter.size_start = 0.35;
        emitter.size_end = 0.08;
        emitter.drag = 0.25;
        emitter.gravity = Vec3::new(0.0, -3.0, 0.0);
        emitter.emissive_strength = 12.0;

        commands.set(entity, emitter);
    }
}

pub fn spawn_explosion(
    commands: &mut Commands<'_>,
    position: Vec3,
    color: Vec3,
    secondary_color: Option<Vec3>,
    explosion_type: ExplosionType,
) {
    let mut generator = rand::rng();

    let flash_entity = commands.spawn(());
    commands.set(flash_entity, ParticleEmitter::flash_burst(position));

    match explosion_type {
        ExplosionType::Peony => {
            let particle_count: u32 = generator.random_range(500..900);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_explosion(position, color, particle_count),
            );

            if let Some(secondary) = secondary_color {
                let inner = commands.spawn(());
                let mut emitter =
                    ParticleEmitter::firework_explosion(position, secondary, particle_count / 2);
                emitter.initial_velocity_min *= 0.7;
                emitter.initial_velocity_max *= 0.7;
                emitter.size_start *= 0.8;
                commands.set(inner, emitter);
            }

            let spark_count: u32 = generator.random_range(100..200);
            let spark_entity = commands.spawn(());
            commands.set(
                spark_entity,
                ParticleEmitter::firework_glitter(position, spark_count),
            );
        }
        ExplosionType::Chrysanthemum => {
            let particle_count: u32 = generator.random_range(700..1200);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_chrysanthemum(position, color, particle_count),
            );

            if let Some(secondary) = secondary_color {
                let inner_count: u32 = generator.random_range(300..500);
                let inner = commands.spawn(());
                let mut emitter =
                    ParticleEmitter::firework_explosion(position, secondary, inner_count);
                emitter.initial_velocity_min *= 0.5;
                emitter.initial_velocity_max *= 0.5;
                commands.set(inner, emitter);
            }
        }
        ExplosionType::Willow => {
            let particle_count: u32 = generator.random_range(600..1000);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_willow(position, color, particle_count),
            );

            let tip_count: u32 = generator.random_range(150..300);
            let tip_entity = commands.spawn(());
            commands.set(
                tip_entity,
                ParticleEmitter::firework_glitter(position, tip_count),
            );
        }
        ExplosionType::Ring => {
            let particle_count: u32 = generator.random_range(350..600);

            for angle_index in 0..4 {
                let angle = (angle_index as f32) * std::f32::consts::TAU / 4.0;
                let entity = commands.spawn(());
                let mut emitter = ParticleEmitter::firework_ring(position, color, particle_count);
                emitter.direction = Vec3::new(angle.sin(), 0.0, angle.cos());
                commands.set(entity, emitter);
            }

            if let Some(secondary) = secondary_color {
                let center_count: u32 = generator.random_range(200..400);
                let entity = commands.spawn(());
                commands.set(
                    entity,
                    ParticleEmitter::firework_explosion(position, secondary, center_count),
                );
            }
        }
        ExplosionType::MultiBreak => {
            let main_count: u32 = generator.random_range(400..600);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_explosion(position, color, main_count),
            );

            for sub_index in 0..generator.random_range(4..8) {
                let offset = Vec3::new(
                    generator.random_range(-10.0..10.0),
                    generator.random_range(-6.0..12.0),
                    generator.random_range(-10.0..10.0),
                );
                let sub_count: u32 = generator.random_range(180..350);
                let sub_color = match sub_index % 3 {
                    0 => color,
                    1 => secondary_color.unwrap_or_else(queries::random_color),
                    _ => queries::random_color(),
                };
                let entity = commands.spawn(());
                let mut emitter =
                    ParticleEmitter::firework_explosion(position + offset, sub_color, sub_count);
                emitter.initial_velocity_min *= 0.6;
                emitter.initial_velocity_max *= 0.6;
                emitter.size_start *= 0.7;
                commands.set(entity, emitter);
            }
        }
        ExplosionType::Glitter => {
            let main_count: u32 = generator.random_range(500..800);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_explosion(position, color, main_count),
            );

            let glitter_count: u32 = generator.random_range(300..500);
            let glitter = commands.spawn(());
            commands.set(
                glitter,
                ParticleEmitter::firework_glitter(position, glitter_count),
            );

            if let Some(secondary) = secondary_color {
                let extra_glitter: u32 = generator.random_range(150..300);
                let extra = commands.spawn(());
                let mut emitter = ParticleEmitter::firework_glitter(position, extra_glitter);
                emitter.color_gradient = ColorGradient::firework_explosion(secondary);
                commands.set(extra, emitter);
            }
        }
        ExplosionType::Crackle => {
            let main_count: u32 = generator.random_range(400..700);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_explosion(position, color, main_count),
            );

            let crackle_count: u32 = generator.random_range(200..400);
            let crackle = commands.spawn(());
            commands.set(
                crackle,
                ParticleEmitter::firework_crackle(position, crackle_count),
            );

            let extra_crackle: u32 = generator.random_range(100..200);
            let extra = commands.spawn(());
            let mut emitter = ParticleEmitter::firework_crackle(position, extra_crackle);
            emitter.particle_lifetime_min *= 1.5;
            emitter.particle_lifetime_max *= 1.5;
            commands.set(extra, emitter);
        }
        ExplosionType::Palm => {
            let trunk_count: u32 = generator.random_range(100..180);
            for frond_index in 0..generator.random_range(6..10) {
                let angle = (frond_index as f32) * std::f32::consts::TAU / 8.0;
                let tilt = generator.random_range(0.3..0.6);
                let entity = commands.spawn(());
                let mut emitter = ParticleEmitter::palm_explosion(position, color, trunk_count);
                emitter.direction =
                    Vec3::new(angle.sin() * tilt, 1.0 - tilt * 0.5, angle.cos() * tilt).normalize();
                commands.set(entity, emitter);
            }

            if let Some(secondary) = secondary_color {
                let tip_count: u32 = generator.random_range(200..400);
                let entity = commands.spawn(());
                let mut emitter = ParticleEmitter::firework_glitter(position, tip_count);
                emitter.color_gradient = ColorGradient::firework_explosion(secondary);
                commands.set(entity, emitter);
            }
        }
        ExplosionType::Crossette => {
            let main_count: u32 = generator.random_range(400..700);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_explosion(position, color, main_count),
            );

            let split_count = generator.random_range(8..15);
            for _ in 0..split_count {
                let offset = Vec3::new(
                    generator.random_range(-12.0..12.0),
                    generator.random_range(-8.0..12.0),
                    generator.random_range(-12.0..12.0),
                );
                let burst_count: u32 = generator.random_range(80..150);
                let burst_color = if generator.random::<f32>() > 0.5 {
                    secondary_color.unwrap_or(color)
                } else {
                    color
                };
                let entity = commands.spawn(());
                commands.set(
                    entity,
                    ParticleEmitter::crossette_burst(position + offset, burst_color, burst_count),
                );
            }
        }
        ExplosionType::Strobe => {
            let main_count: u32 = generator.random_range(500..800);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_explosion(position, color, main_count),
            );

            let strobe_count: u32 = generator.random_range(150..300);
            let strobe = commands.spawn(());
            commands.set(
                strobe,
                ParticleEmitter::strobe_effect(position, strobe_count),
            );
        }
        ExplosionType::Kamuro => {
            let particle_count: u32 = generator.random_range(1000..1500);
            let entity = commands.spawn(());
            let mut emitter = ParticleEmitter::firework_willow(position, color, particle_count);
            emitter.initial_velocity_min = 20.0;
            emitter.initial_velocity_max = 35.0;
            emitter.particle_lifetime_min = 4.0;
            emitter.particle_lifetime_max = 7.0;
            emitter.gravity = Vec3::new(0.0, -3.0, 0.0);
            emitter.drag = 0.08;
            emitter.size_start = 0.25;
            emitter.emissive_strength = 10.0;
            commands.set(entity, emitter);

            if let Some(secondary) = secondary_color {
                let inner_count: u32 = generator.random_range(400..600);
                let inner = commands.spawn(());
                let mut emitter =
                    ParticleEmitter::firework_chrysanthemum(position, secondary, inner_count);
                emitter.initial_velocity_min *= 0.5;
                emitter.initial_velocity_max *= 0.5;
                commands.set(inner, emitter);
            }
        }
        ExplosionType::ShapeSmiley => {
            let pattern = queries::smiley_pattern();
            spawn_shaped_explosion(commands, position, color, &pattern, 20.0, 8);
            if let Some(secondary) = secondary_color {
                let glitter_count: u32 = generator.random_range(100..200);
                let entity = commands.spawn(());
                let mut emitter = ParticleEmitter::firework_glitter(position, glitter_count);
                emitter.color_gradient = ColorGradient::firework_explosion(secondary);
                commands.set(entity, emitter);
            }
        }
        ExplosionType::ShapeHeart => {
            let pattern = queries::heart_pattern();
            spawn_shaped_explosion(commands, position, color, &pattern, 22.0, 10);
            if let Some(secondary) = secondary_color {
                let inner = queries::heart_pattern();
                spawn_shaped_explosion(commands, position, secondary, &inner, 12.0, 5);
            }
        }
        ExplosionType::ShapeStar => {
            let pattern = queries::star_pattern();
            spawn_shaped_explosion(commands, position, color, &pattern, 20.0, 12);
            let glitter_count: u32 = generator.random_range(150..250);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_glitter(position, glitter_count),
            );
        }
        ExplosionType::ShapeCircle => {
            let pattern = queries::circle_pattern();
            spawn_shaped_explosion(commands, position, color, &pattern, 22.0, 10);
            if let Some(secondary) = secondary_color {
                let glitter_count: u32 = generator.random_range(150..250);
                let entity = commands.spawn(());
                let mut emitter = ParticleEmitter::firework_glitter(position, glitter_count);
                emitter.color_gradient = ColorGradient::firework_explosion(secondary);
                commands.set(entity, emitter);
            }
        }
        ExplosionType::ShapeCross => {
            let pattern = queries::cross_pattern();
            spawn_shaped_explosion(commands, position, color, &pattern, 20.0, 10);
            if let Some(secondary) = secondary_color {
                let center_count: u32 = generator.random_range(100..200);
                let entity = commands.spawn(());
                commands.set(
                    entity,
                    ParticleEmitter::firework_explosion(position, secondary, center_count),
                );
            }
        }
        ExplosionType::ShapeDiamond => {
            let pattern = queries::diamond_pattern();
            spawn_shaped_explosion(commands, position, color, &pattern, 22.0, 10);
            if let Some(secondary) = secondary_color {
                let inner_pattern = queries::diamond_pattern();
                spawn_shaped_explosion(commands, position, secondary, &inner_pattern, 12.0, 6);
            }
        }
        ExplosionType::SplittingShell => {
            let initial_burst: u32 = generator.random_range(200..400);
            let entity = commands.spawn(());
            commands.set(
                entity,
                ParticleEmitter::firework_explosion(position, color, initial_burst),
            );
        }
    }
}

pub fn spawn_splitting_sub_shells(
    commands: &mut Commands<'_>,
    position: Vec3,
    color: Vec3,
    secondary_color: Option<Vec3>,
) {
    let mut generator = rand::rng();
    let sub_count = generator.random_range(4..8);

    for _ in 0..sub_count {
        let angle = generator.random::<f32>() * std::f32::consts::TAU;
        let elevation = generator.random_range(0.2..0.6);
        let speed = generator.random_range(20.0..35.0);

        let velocity = Vec3::new(
            angle.cos() * (1.0 - elevation) * speed,
            elevation * speed,
            angle.sin() * (1.0 - elevation) * speed,
        );

        let entity = commands.spawn(());
        commands.set(entity, ParticleEmitter::firework_shell(position, velocity));

        let sub_color = if generator.random::<f32>() > 0.5 {
            secondary_color.unwrap_or_else(queries::random_color)
        } else {
            color
        };
        let sub_secondary = if generator.random::<f32>() > 0.6 {
            Some(queries::random_color())
        } else {
            None
        };

        commands.set(entity, Velocity(velocity));
        commands.set(entity, Fuse(generator.random_range(0.8..1.4)));
        commands.set(entity, Tint(sub_color));
        commands.set(entity, SecondTint(sub_secondary));
        commands.set(entity, Burst(queries::random_sub_explosion_type()));
        commands.set(entity, Generation(1));
    }
}
