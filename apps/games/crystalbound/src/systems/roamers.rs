use crate::commands::show_toast;
use crate::components::{Home, Hurt, Speed, TouchRest};
use crate::contract::MonsterKind;
use crate::data;
use crate::queries::walkable;
use crate::resources::{Field, Party, Rig, Toast};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub type Hunters<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item MonsterKind,
        &'item Speed,
        &'item mut Hurt,
        &'item Home,
        &'item mut Sprite,
        &'item mut LocalTransform,
    ),
>;
pub type Quarry<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut TouchRest, &'item LocalTransform)>;

pub fn stalk_the_party(
    time: Res<Time>,
    field: Res<Field>,
    mut rig: ResMut<Rig>,
    mut party: ResMut<Party>,
    mut toast: ResMut<Toast>,
    mut hunters: Hunters<'_, '_>,
    mut quarry: Quarry<'_, '_>,
) {
    let delta_time = time.delta_time;
    let mut target = None;
    let mut open = false;
    quarry.for_each_mut(|_entity, (touch_rest, transform)| {
        target = Some(Vec2::new(transform.translation.x, transform.translation.y));
        open = touch_rest.0 <= 0.0;
    });
    let Some(target) = target else {
        return;
    };

    let ground = &*field;
    let mut bite = 0;
    let mut biter = None;
    hunters.for_each_mut(|_entity, (kind, speed, hurt, home, sprite, transform)| {
        hurt.0 = (hurt.0 - delta_time).max(0.0);
        if hurt.0 <= 0.0 && sprite.color != theme::ACTOR_TINT {
            sprite.color = theme::ACTOR_TINT;
        }

        let here = Vec2::new(transform.translation.x, transform.translation.y);
        let toward_party = target - here;
        let distance = toward_party.magnitude();
        let heading = if distance < theme::ROAMER_SIGHT && distance > 0.01 {
            toward_party / distance
        } else {
            let homeward = home.0 - here;
            if homeward.magnitude() > 0.4 {
                homeward.normalize() * 0.45
            } else {
                Vec2::zeros()
            }
        };
        if heading.magnitude_squared() > 0.0 {
            let step = heading * speed.0 * delta_time;
            let sideways = Vec2::new(here.x + step.x, here.y);
            if walkable(ground, sideways) {
                transform.translation.x = sideways.x;
            }
            let upright = Vec2::new(transform.translation.x, here.y + step.y);
            if walkable(ground, upright) {
                transform.translation.y = upright.y;
            }
            if step.x != 0.0 {
                sprite.flip_x = step.x < 0.0;
            }
        }

        if open && distance < theme::TOUCH_RANGE && bite == 0 {
            bite = data::monster_stats(*kind).power;
            biter = Some(kind.name());
        }
    });

    if bite == 0 {
        return;
    }
    quarry.for_each_mut(|_entity, (touch_rest, _)| touch_rest.0 = theme::TOUCH_REST);
    rig.shake = rig.shake.max(0.22);

    let guard = party.guard_level as i32 * 2;
    let hurt = (bite - guard).max(1);
    if let Some(hero) = party.heroes.iter_mut().find(|hero| hero.alive()) {
        hero.health = (hero.health - hurt).max(0);
        let name = hero.name;
        let down = !hero.alive();
        if let Some(biter) = biter {
            if down {
                show_toast(&mut toast, format!("The {biter} fells {name}."));
            } else {
                show_toast(&mut toast, format!("The {biter} hits {name} for {hurt}."));
            }
        }
    }
}
