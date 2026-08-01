use crate::contract::{Cue, Glyph};
use crate::data;
use crate::resources::{ArchState, Runner, Story, Walker};

pub fn turn_dial(arch: &mut ArchState, dial: usize) -> Glyph {
    if dial >= data::DIAL_COUNT {
        return arch.dials[0];
    }
    arch.dials[dial] = arch.dials[dial].next();
    arch.dials[dial]
}

pub fn open_arch(arch: &mut ArchState) {
    arch.open = true;
}

pub fn close_arch(arch: &mut ArchState) {
    arch.dials = [Glyph::Bar; data::DIAL_COUNT];
    arch.open = false;
}

pub fn place_in_glass(runner: &mut Runner) {
    runner.position = data::GLASS_START;
    runner.velocity = nightshade::prelude::Vec3::zeros();
    runner.grounded = false;
    runner.since_ground = 0.0;
    runner.armed = false;
    runner.focus = data::GLASS_START;
}

pub fn queue_cue(story: &mut Story, cue: Cue) {
    for (index, line) in data::SCRIPT.iter().enumerate() {
        if story.fired[index] || line.cue != cue {
            continue;
        }
        story.fired[index] = true;
        story.queue.push(index);
    }
}

pub fn dispatch_last(
    viewport: &mut nightshade_render::resources::viewport::Viewport,
    eyes: [nightshade::prelude::Entity; 5],
    active: nightshade::prelude::Entity,
) {
    viewport
        .required_cameras
        .retain(|held| !eyes.contains(held));
    viewport.required_cameras.push(active);
}

pub fn fall_back(runner: &mut Runner) {
    runner.position = data::GLASS_START;
    runner.velocity = nightshade::prelude::Vec3::zeros();
    runner.grounded = false;
    runner.since_ground = 0.0;
    runner.focus = data::GLASS_START;
}

pub fn restart_hub(walker: &mut Walker) {
    walker.position = data::HUB_START;
    walker.velocity = nightshade::prelude::Vec3::zeros();
    walker.grounded = false;
    walker.since_ground = 0.0;
    walker.yaw = 0.0;
    walker.pitch = 0.0;
}

pub fn place_in_hub(walker: &mut Walker) {
    walker.position = nightshade::prelude::Vec3::new(0.0, 0.0, -data::HUB_HALF + 2.4);
    walker.velocity = nightshade::prelude::Vec3::zeros();
    walker.grounded = false;
    walker.since_ground = 0.0;
    walker.yaw = std::f32::consts::PI;
    walker.pitch = 0.0;
}
