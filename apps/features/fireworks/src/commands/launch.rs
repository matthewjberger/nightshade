use crate::components::{Burst, Fuse, Generation, SecondTint, Tint, Velocity};
use crate::data::{ExplosionType, ShellType};
use crate::queries;
use crate::resources::{Launcher, Show};
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use rand::Rng;

fn spawn_shell(
    commands: &mut Commands<'_>,
    launch_position: Vec3,
    velocity: Vec3,
    fuse_time: f32,
    color: Vec3,
    secondary_color: Option<Vec3>,
    explosion_type: ExplosionType,
) {
    let entity = commands.spawn(());
    let trail = match queries::random_shell_type() {
        ShellType::Standard => ParticleEmitter::firework_shell(launch_position, velocity),
        ShellType::Comet => ParticleEmitter::comet_shell(launch_position, velocity),
    };
    commands.set(entity, trail);
    commands.set(entity, Velocity(velocity));
    commands.set(entity, Fuse(fuse_time));
    commands.set(entity, Tint(color));
    commands.set(entity, SecondTint(secondary_color));
    commands.set(entity, Burst(explosion_type));
    commands.set(entity, Generation(0));
}

pub fn launch_firework(commands: &mut Commands<'_>) {
    let mut generator = rand::rng();

    let spread = 60.0;
    let x_offset: f32 = generator.random_range(-spread..spread);
    let z_offset: f32 = generator.random_range(-20.0..20.0);
    let launch_position = Vec3::new(x_offset, 0.0, z_offset);
    let target_height: f32 = generator.random_range(55.0..95.0);

    let velocity = Vec3::new(
        generator.random_range(-4.0..4.0),
        generator.random_range(50.0..75.0),
        generator.random_range(-3.0..3.0),
    );

    let secondary_color = if generator.random::<f32>() > 0.5 {
        Some(queries::random_color())
    } else {
        None
    };

    spawn_shell(
        commands,
        launch_position,
        velocity,
        target_height / velocity.y,
        queries::random_color(),
        secondary_color,
        queries::random_explosion_type(),
    );
}

pub fn launch_specific_firework(
    commands: &mut Commands<'_>,
    explosion_type: ExplosionType,
    color: Option<Vec3>,
    x_position: Option<f32>,
) {
    let mut generator = rand::rng();

    let x_offset = x_position.unwrap_or_else(|| generator.random_range(-60.0..60.0));
    let z_offset: f32 = generator.random_range(-20.0..20.0);
    let launch_position = Vec3::new(x_offset, 0.0, z_offset);
    let target_height: f32 = generator.random_range(55.0..95.0);

    let velocity = Vec3::new(
        generator.random_range(-2.0..2.0),
        generator.random_range(50.0..75.0),
        generator.random_range(-2.0..2.0),
    );

    let secondary_color = if generator.random::<f32>() > 0.4 {
        Some(queries::random_color())
    } else {
        None
    };

    spawn_shell(
        commands,
        launch_position,
        velocity,
        target_height / velocity.y,
        color.unwrap_or_else(queries::random_color),
        secondary_color,
        explosion_type,
    );
}

pub fn launch_salvo(commands: &mut Commands<'_>, count: u32) {
    for _ in 0..count {
        launch_firework(commands);
    }
}

pub fn launch_finale(commands: &mut Commands<'_>) {
    launch_salvo(commands, 50);
}

pub fn launch_symmetric_fireworks(
    commands: &mut Commands<'_>,
    explosion_type: ExplosionType,
    count: u32,
) {
    let color = queries::random_color();
    let spacing = 100.0 / (count as f32);
    let start_x = -50.0 + spacing / 2.0;

    for index in 0..count {
        let x_position = start_x + (index as f32) * spacing;
        launch_specific_firework(commands, explosion_type, Some(color), Some(x_position));
    }
}

pub fn start_show(show: &mut Show, launcher: &mut Launcher, uptime: f32) {
    show.active = true;
    show.start_time = uptime;
    show.events = crate::data::show_script();
    show.index = 0;
    launcher.auto = false;
}
