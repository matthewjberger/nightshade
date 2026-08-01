use crate::contract::{Sprite, SpriteAnimator};
use nightshade::ecs::system_param::{Query, Res};
use nightshade_platform::contract::time::Time;

pub fn advance_sprite_animators(
    time: Res<Time>,
    animators: Query<(&mut SpriteAnimator, &mut Sprite)>,
) {
    let delta_time = time.delta_time;
    animators.for_each(|_entity, (animator, sprite)| {
        if animator.advance(delta_time)
            && let Some(region) = animator.current_region()
        {
            sprite.region = region;
        }
    });
}
