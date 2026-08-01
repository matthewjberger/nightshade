use crate::components::{Facing, Speed, SwingRest, TouchRest};
use crate::data::Ask;
use crate::queries::walkable;
use crate::resources::Field;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub type Party<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Speed,
        &'item mut Facing,
        &'item mut SwingRest,
        &'item mut TouchRest,
        &'item mut LocalTransform,
        &'item mut Sprite,
    ),
>;

pub fn walk_the_party(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut field: ResMut<Field>,
    mut party: Party<'_, '_>,
) {
    let delta_time = time.delta_time;
    let mut heading = actions.axis2(Ask::Walk);
    let moving = heading.magnitude_squared() > 0.0;
    if moving {
        heading = heading.normalize();
    }

    let ground = &*field;
    let mut walked = 0.0;
    party.for_each_mut(
        |_entity, (speed, facing, swing_rest, touch_rest, transform, sprite)| {
            swing_rest.0 = (swing_rest.0 - delta_time).max(0.0);
            touch_rest.0 = (touch_rest.0 - delta_time).max(0.0);
            if !moving {
                return;
            }
            facing.0 = heading;
            if heading.x != 0.0 {
                sprite.flip_x = heading.x < 0.0;
            }

            let step = heading * speed.0 * delta_time;
            let here = Vec2::new(transform.translation.x, transform.translation.y);
            let mut landed = here;
            let sideways = Vec2::new(here.x + step.x, here.y);
            if walkable(ground, sideways) {
                landed.x = sideways.x;
            }
            let upright = Vec2::new(landed.x, here.y + step.y);
            if walkable(ground, upright) {
                landed.y = upright.y;
            }
            walked += (landed - here).magnitude();
            transform.translation.x = landed.x;
            transform.translation.y = landed.y;
        },
    );
    field.walked += walked;
}
