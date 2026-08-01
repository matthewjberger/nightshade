use crate::theme;
use blocker_beasts::components::{Angry, Beast};
use blocker_beasts::data::beasts::{BEASTS, NEUTRAL_OUTLINE_COLOR, THREAT_OUTLINE_COLOR};
use blocker_beasts::data::settings::Settings;
use blocker_beasts::resources::Quarry;
use blocker_figure::components::{Eye, Gaze, Home};
use blocker_figure::queries::{looks_at, read_the_hero};
use blocker_figure::resources::Hero;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_render::resources::outline::Outline;
use nightshade_scene::prelude::LocalTransform;

pub fn edge_the_aimed(
    quarry: Res<Quarry>,
    config: Res<Config<Settings>>,
    hero: Res<Hero>,
    mut outline: ResMut<Outline>,
    beasts: Query<(&Beast, &Angry, &LocalTransform)>,
    folk: Query<(&LocalTransform,), With<Home>>,
    mut sighted: Query<(&Eye, &Gaze)>,
) {
    let Some((eye, gaze)) = read_the_hero(&hero, &mut sighted, |(eye, gaze)| (eye.0, gaze.0))
    else {
        return;
    };
    outline.active_entity = None;
    let player_melee = config.number(Settings::PlayerMelee);
    let mut quarry_hostile = None;
    let mut threats = Vec::new();
    let mut aimed: Option<(f32, Entity, bool)> = None;
    beasts.for_each(|entity, (beast, angry, transform)| {
        let head = transform.translation + Vec3::new(0.0, BEASTS[beast.0].head, 0.0);
        let hostile = angry.0 > 0.0;
        if quarry.0 == Some(entity) {
            quarry_hostile = Some(hostile);
        }
        if hostile && (head - eye).magnitude() < theme::THREAT_RANGE {
            threats.push(entity);
        }
        if looks_at(
            eye,
            gaze,
            head,
            player_melee,
            blocker_figure::data::sight::LOOK_COSINE,
        ) {
            let distance = (head - eye).magnitude();
            if aimed.is_none_or(|(nearest, _, _)| distance < nearest) {
                aimed = Some((distance, entity, hostile));
            }
        }
    });
    if let Some(target) = quarry.0
        && let Some(hostile) = quarry_hostile
    {
        outline.entities = vec![target];
        outline.color = beast_tint(hostile);
        return;
    }
    if !threats.is_empty() {
        outline.entities = threats;
        outline.color = THREAT_OUTLINE_COLOR;
        return;
    }
    if let Some((_, entity, hostile)) = aimed {
        outline.entities = vec![entity];
        outline.color = beast_tint(hostile);
        return;
    }
    let mut best: Option<(f32, Entity)> = None;
    folk.for_each(|entity, (transform,)| {
        let center = transform.translation + Vec3::new(0.0, theme::FOLK_HEAD_CENTER, 0.0);
        let to = center - eye;
        let along = to.dot(&gaze);
        if along <= 0.0 || along > theme::FOLK_HOVER_REACH {
            return;
        }
        let closest = eye + gaze * along;
        if (center - closest).magnitude() > theme::FOLK_HOVER_RADIUS {
            return;
        }
        if best.is_none_or(|(nearest, _)| along < nearest) {
            best = Some((along, entity));
        }
    });
    match best {
        Some((_, entity)) => {
            outline.entities = vec![entity];
            outline.color = theme::FOLK_OUTLINE_COLOR;
        }
        None => outline.entities.clear(),
    }
}

fn beast_tint(hostile: bool) -> [f32; 4] {
    if hostile {
        THREAT_OUTLINE_COLOR
    } else {
        NEUTRAL_OUTLINE_COLOR
    }
}
