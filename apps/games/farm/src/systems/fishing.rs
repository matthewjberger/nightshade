use crate::commands::fishing::reel_in;
use crate::commands::pocket::show_toast;
use crate::resources::{Fishing, FishingState, Toast};
use crate::tags::Bobber;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn wait_for_a_bite(
    time: Res<Time>,
    mut fishing: ResMut<Fishing>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    bobbers: Query<(&mut LocalTransform,), WithTag<Bobber>>,
) {
    let delta_time = time.delta_time;
    let dip = match fishing.state {
        FishingState::Idle => return,
        FishingState::Waiting(remaining) => {
            let remaining = remaining - delta_time;
            if remaining <= 0.0 {
                fishing.state = FishingState::Bite(theme::BITE_WINDOW);
                show_toast(&mut toast, "Something bites! Pull!");
            } else {
                fishing.state = FishingState::Waiting(remaining);
            }
            (remaining * 6.0).sin() * 0.03
        }
        FishingState::Bite(window) => {
            let window = window - delta_time;
            if window <= 0.0 {
                reel_in(&mut commands, &mut fishing);
                show_toast(&mut toast, "It got away.");
                return;
            }
            fishing.state = FishingState::Bite(window);
            -0.14
        }
    };
    bobbers.for_each(|_entity, (transform,)| {
        transform.translation.y = theme::WATER_LEVEL + 0.02 + dip;
    });
}
