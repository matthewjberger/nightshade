use crate::components::Beast;
use crate::data::beasts::BEASTS;
use crate::theme;
use blocker_figure::commands::plate_the_body;
use blocker_figure::components::{Eye, Plate};
use blocker_figure::queries::{plate_properties, read_the_hero};
use blocker_figure::resources::Hero;
use nalgebra_glm::{Vec3, Vec4};
use nightshade::prelude::*;
use nightshade_health::{Health, queries::fallen};
use nightshade_scene::prelude::LocalTransform;
use nightshade_text::prelude::TextState;

pub fn label_the_beasts(
    hero: Res<Hero>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
    beasts: Query<(&Beast, &Health, &LocalTransform, Option<&Plate>)>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(place,)| place.0) else {
        return;
    };
    let mut wanted: Vec<(Entity, usize, Vec3)> = Vec::new();
    beasts.for_each(|entity, (beast, health, transform, plate)| {
        let near = (transform.translation - eye).magnitude() < blocker_figure::theme::PLATE_RANGE;
        if plate.is_none() && near && !fallen(health) {
            wanted.push((entity, beast.0, transform.translation));
        }
    });
    for (entity, kind, position) in wanted {
        let info = &BEASTS[kind];
        let color = if info.aggro > 0.0 {
            crate::data::beasts::THREAT_OUTLINE_COLOR
        } else {
            crate::data::beasts::NEUTRAL_OUTLINE_COLOR
        };
        let mut properties = plate_properties();
        properties.font_size = theme::LABEL_SIZE;
        properties.color = Vec4::new(color[0], color[1], color[2], color[3]);
        plate_the_body(
            &mut text_state,
            &mut commands,
            entity,
            info.name,
            position,
            properties,
        );
    }
}
