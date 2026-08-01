use crate::components::{Effect, Generation};
use crate::data::{
    EffectKind, ExplosionType, FIREWORK_COLORS, LAUNCH_TYPES, SUB_TYPES, ShellType, Stats,
};
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use rand::Rng;

pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
    let chroma = value * saturation;
    let second = chroma * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let match_value = value - chroma;

    let (red, green, blue) = if hue < 60.0 {
        (chroma, second, 0.0)
    } else if hue < 120.0 {
        (second, chroma, 0.0)
    } else if hue < 180.0 {
        (0.0, chroma, second)
    } else if hue < 240.0 {
        (0.0, second, chroma)
    } else if hue < 300.0 {
        (second, 0.0, chroma)
    } else {
        (chroma, 0.0, second)
    };

    Vec3::new(red + match_value, green + match_value, blue + match_value)
}

pub fn random_color() -> Vec3 {
    let mut generator = rand::rng();

    if generator.random::<f32>() < 0.3 {
        let hue = generator.random::<f32>() * 360.0;
        let saturation = 0.7 + generator.random::<f32>() * 0.3;
        let value = 0.9 + generator.random::<f32>() * 0.1;
        return hsv_to_rgb(hue, saturation, value);
    }

    let channels = FIREWORK_COLORS[generator.random_range(0..FIREWORK_COLORS.len())];
    Vec3::new(channels[0], channels[1], channels[2])
}

pub fn random_explosion_type() -> ExplosionType {
    let mut generator = rand::rng();
    LAUNCH_TYPES[generator.random_range(0..LAUNCH_TYPES.len())]
}

pub fn random_sub_explosion_type() -> ExplosionType {
    let mut generator = rand::rng();
    SUB_TYPES[generator.random_range(0..SUB_TYPES.len())]
}

pub fn random_shell_type() -> ShellType {
    let mut generator = rand::rng();
    if generator.random::<f32>() < 0.3 {
        ShellType::Comet
    } else {
        ShellType::Standard
    }
}

pub fn smiley_pattern() -> Vec<Vec2> {
    let mut points = Vec::new();

    for index in 0..60 {
        let angle = (index as f32) * std::f32::consts::TAU / 60.0;
        points.push(vec2(angle.cos(), angle.sin()));
    }

    for index in 0..16 {
        let angle = (index as f32) * std::f32::consts::TAU / 16.0;
        let radius = 0.15;
        points.push(vec2(
            -0.38 + angle.cos() * radius,
            0.32 + angle.sin() * radius,
        ));
    }

    for index in 0..16 {
        let angle = (index as f32) * std::f32::consts::TAU / 16.0;
        let radius = 0.15;
        points.push(vec2(
            0.38 + angle.cos() * radius,
            0.32 + angle.sin() * radius,
        ));
    }

    for index in 0..30 {
        let step = index as f32 / 29.0;
        let angle = std::f32::consts::PI * 0.2 + step * std::f32::consts::PI * 0.6;
        points.push(vec2(angle.cos() * 0.5, -angle.sin() * 0.5 + 0.05));
    }

    points
}

pub fn heart_pattern() -> Vec<Vec2> {
    let mut points = Vec::new();
    for index in 0..100 {
        let step = (index as f32) * std::f32::consts::TAU / 100.0;
        let x = 16.0 * step.sin().powi(3);
        let y = 13.0 * step.cos()
            - 5.0 * (2.0 * step).cos()
            - 2.0 * (3.0 * step).cos()
            - (4.0 * step).cos();
        points.push(vec2(x / 17.0, y / 17.0));
    }
    points
}

pub fn star_pattern() -> Vec<Vec2> {
    let mut points = Vec::new();
    let outer_radius = 1.0;
    let inner_radius = 0.38;

    for point_index in 0..5 {
        let outer_angle =
            (point_index as f32) * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2;
        let inner_angle = outer_angle + std::f32::consts::TAU / 10.0;

        let outer_point = vec2(
            outer_angle.cos() * outer_radius,
            outer_angle.sin() * outer_radius,
        );
        let inner_point = vec2(
            inner_angle.cos() * inner_radius,
            inner_angle.sin() * inner_radius,
        );

        let next_outer_angle =
            ((point_index + 1) as f32) * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2;
        let next_outer_point = vec2(
            next_outer_angle.cos() * outer_radius,
            next_outer_angle.sin() * outer_radius,
        );

        for step_index in 0..10 {
            let step = step_index as f32 / 10.0;
            points.push(vec2(
                outer_point.x * (1.0 - step) + inner_point.x * step,
                outer_point.y * (1.0 - step) + inner_point.y * step,
            ));
        }
        for step_index in 0..10 {
            let step = step_index as f32 / 10.0;
            points.push(vec2(
                inner_point.x * (1.0 - step) + next_outer_point.x * step,
                inner_point.y * (1.0 - step) + next_outer_point.y * step,
            ));
        }
    }

    points
}

pub fn circle_pattern() -> Vec<Vec2> {
    let mut points = Vec::new();
    for index in 0..60 {
        let angle = (index as f32) * std::f32::consts::TAU / 60.0;
        points.push(vec2(angle.cos(), angle.sin()));
    }
    for index in 0..40 {
        let angle = (index as f32) * std::f32::consts::TAU / 40.0;
        points.push(vec2(angle.cos() * 0.5, angle.sin() * 0.5));
    }
    points
}

pub fn cross_pattern() -> Vec<Vec2> {
    let mut points = Vec::new();
    let arm_length = 1.0;

    for index in 0..25 {
        let step = index as f32 / 24.0;
        let position = -arm_length + step * 2.0 * arm_length;
        points.push(vec2(0.0, position));
    }

    for index in 0..25 {
        let step = index as f32 / 24.0;
        let position = -arm_length + step * 2.0 * arm_length;
        points.push(vec2(position, 0.0));
    }

    points
}

pub fn diamond_pattern() -> Vec<Vec2> {
    let mut points = Vec::new();
    let size = 1.0;

    let corners = [
        vec2(0.0, size),
        vec2(size, 0.0),
        vec2(0.0, -size),
        vec2(-size, 0.0),
    ];

    for edge in 0..4 {
        let start = corners[edge];
        let end = corners[(edge + 1) % 4];
        for step_index in 0..15 {
            let step = step_index as f32 / 15.0;
            points.push(vec2(
                start.x * (1.0 - step) + end.x * step,
                start.y * (1.0 - step) + end.y * step,
            ));
        }
    }

    for index in 0..12 {
        let step = index as f32 / 11.0;
        points.push(vec2(0.0, -size * 0.5 + step * size));
    }
    for index in 0..12 {
        let step = index as f32 / 11.0;
        points.push(vec2(-size * 0.5 + step * size, 0.0));
    }

    points
}

pub fn count_stats(storage: &nightshade::ecs::Storage) -> Stats {
    let mut stats = Stats::default();

    for (_entity, (generation,)) in storage.query_ref::<(&Generation,)>().iter() {
        if generation.0 == 0 {
            stats.shells += 1;
        } else {
            stats.sub_shells += 1;
        }
    }

    for (_entity, (effect, emitter)) in storage.query_ref::<(&Effect, &ParticleEmitter)>().iter() {
        match effect.0 {
            EffectKind::Fire => stats.fire += 1,
            EffectKind::Smoke => stats.smoke += 1,
            EffectKind::Sprite => {
                stats.sprite_total += 1;
                if emitter.enabled {
                    stats.sprite_active += 1;
                }
            }
        }
    }

    stats.emitters = storage.query_ref::<(&ParticleEmitter,)>().iter().count();
    stats
}
