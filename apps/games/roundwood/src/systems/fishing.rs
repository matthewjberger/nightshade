use crate::commands::pocket::show_toast;
use crate::data;
use crate::resources::{Fishing, FishingState, Toast};
use crate::tags::Bobber;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_sprite::prelude::*;

pub fn wait_for_a_bite(
    time: Res<Time>,
    mut fishing: ResMut<Fishing>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    bobbers: Query<(&mut Sprite,), WithTag<Bobber>>,
) {
    if fishing.state == FishingState::Idle {
        return;
    }
    fishing.timer -= time.delta_time;
    if fishing.timer <= 0.0 {
        match fishing.state {
            FishingState::Waiting => {
                fishing.state = FishingState::Bite;
                fishing.timer = data::BITE_WINDOW;
            }
            _ => {
                fishing.state = FishingState::Idle;
                if let Some(bobber) = fishing.bobber.take() {
                    commands.despawn(bobber);
                }
                show_toast(&mut toast, "The fish slipped away.");
            }
        }
    }
    let biting = fishing.state == FishingState::Bite;
    let dip = if biting {
        0.55 + 0.25 * (fishing.timer * 26.0).sin()
    } else {
        0.4
    };
    bobbers.for_each(|_entity, (sprite,)| {
        sprite.size = Vec2::new(dip, dip);
        sprite.color = if biting {
            [1.0, 0.7, 0.5, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };
    });
}
