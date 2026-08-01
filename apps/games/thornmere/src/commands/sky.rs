use crate::commands::scenery::prop;
use crate::components::{Drift, Orbit, Phase, Spin};
use crate::data::{
    BIRD_FLOCKS, BIRD_HEIGHT, BIRD_RADIUS, BIRD_SPEED, BIRDS_PER_FLOCK, CLOUD_CEILING, CLOUD_COUNT,
    CLOUD_DRIFT, CLOUD_FLOOR, CLOUD_REACH,
};
use crate::resources::Palette;
use nightshade::prelude::*;
use nightshade_random::prelude::*;
use nightshade_scene::prelude::{ChildOf, GlobalTransform, LocalTransform, Name};

const TAU: f32 = std::f32::consts::TAU;

pub fn spawn_clouds(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    for index in 0..CLOUD_COUNT {
        let center = Vec3::new(
            between(random, -CLOUD_REACH, CLOUD_REACH),
            between(random, CLOUD_FLOOR, CLOUD_CEILING),
            between(random, -CLOUD_REACH, CLOUD_REACH),
        );
        let root = commands.spawn((
            Name(format!("Cloud {index}")),
            LocalTransform {
                translation: center,
                rotation: nalgebra_glm::Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            Drift(Vec3::new(
                CLOUD_DRIFT * between(random, 0.7, 1.3),
                0.0,
                CLOUD_DRIFT * between(random, 0.2, 0.5),
            )),
        ));
        let puffs = random.usize(4..8);
        let spread = between(random, 9.0, 16.0);
        for puff in 0..puffs {
            let along = (puff as f32 / (puffs - 1) as f32 - 0.5) * spread * 2.0;
            let size = between(random, 7.0, 13.0) * (1.0 - 0.35 * (along / spread).abs());
            let offset = Vec3::new(
                along,
                between(random, -1.5, 2.5),
                between(random, -4.0, 4.0),
            );
            let ball = prop(
                commands,
                "Sphere",
                offset,
                Vec3::new(size, size * 0.45, size * 0.8),
                palette.cloud,
            );
            commands.set(ball, ChildOf(root));
        }
    }
}

pub fn spawn_birds(commands: &mut Commands, palette: &Palette, random: &mut Random) {
    for flock in 0..BIRD_FLOCKS {
        let center = Vec3::new(
            between(random, -CLOUD_REACH * 0.6, CLOUD_REACH * 0.6),
            BIRD_HEIGHT + between(random, -6.0, 10.0),
            between(random, -CLOUD_REACH * 0.6, CLOUD_REACH * 0.6),
        );
        let radius = BIRD_RADIUS * between(random, 0.7, 1.4);
        let speed = BIRD_SPEED / radius * if flock % 2 == 0 { 1.0 } else { -1.0 };
        for bird in 0..BIRDS_PER_FLOCK {
            let phase = bird as f32 / BIRDS_PER_FLOCK as f32 * 0.9 + between(random, 0.0, 0.1);
            let body = prop(
                commands,
                "Cube",
                center,
                Vec3::new(1.4, 0.12, 0.5),
                palette.feather,
            );
            commands.set(body, Orbit(Vec3::new(center.x, center.y, center.z)));
            commands.set(body, Spin(speed));
            commands.set(body, Phase(phase * TAU));
            commands.set(body, Drift(Vec3::new(radius, 0.0, radius * 0.7)));
        }
    }
}
