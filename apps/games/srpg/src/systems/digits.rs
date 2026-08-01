use crate::components::{Age, Tint};
use crate::theme;
use nightshade::ecs::HierarchyIndex;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn float_the_digits(
    time: Res<Time>,
    hierarchy: Res<HierarchyIndex>,
    mut commands: Commands,
    holders: Query<(&mut Age, &Tint, &mut LocalTransform)>,
    mut glyphs: Query<(&mut Sprite,)>,
) {
    let delta_time = time.delta_time;
    let mut expired = Vec::new();
    let mut fades = Vec::new();
    holders.for_each(|entity, (age, tint, transform)| {
        age.0 += delta_time;
        let share = (age.0 / theme::DIGIT_SECONDS).clamp(0.0, 1.0);
        transform.translation.y += delta_time * theme::DIGIT_RISE * (1.0 - share);
        let alpha = if share > 0.6 {
            1.0 - (share - 0.6) / 0.4
        } else {
            1.0
        };
        fades.push((entity, [tint.0[0], tint.0[1], tint.0[2], alpha]));
        if age.0 >= theme::DIGIT_SECONDS {
            expired.push(entity);
        }
    });
    for (holder, color) in fades {
        for child in hierarchy.children(holder) {
            glyphs.for_entity(*child, |(sprite,)| sprite.color = color);
        }
    }
    for entity in expired {
        commands.despawn_recursive(entity);
    }
}
