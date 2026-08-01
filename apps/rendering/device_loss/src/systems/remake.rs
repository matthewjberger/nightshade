use crate::theme;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::resources::capture::ScheduledCapture;

pub fn remake_on_key(input: Res<Input>, mut queues: ResMut<CommandQueues>) {
    if input.keyboard.just_pressed(winit::keyboard::KeyCode::KeyR) {
        nightshade_graphics::commands::queues::remake_renderer(&mut queues);
    }
}

pub fn remake_for_the_picture(capture: Res<ScheduledCapture>, mut queues: ResMut<CommandQueues>) {
    if capture.path.is_some() && capture.elapsed == theme::REMADE_AT {
        nightshade_graphics::commands::queues::remake_renderer(&mut queues);
    }
}
