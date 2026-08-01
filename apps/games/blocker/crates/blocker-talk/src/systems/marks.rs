use blocker_crew::components::{Mark, Marked, Offer};
use blocker_crew::data::marks::MarkState;
use blocker_figure::components::Eye;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_pocket::resources::Inventory;
use blocker_quests::data::quests::QUESTS;
use blocker_quests::queries::{objective_met, offered_quest};
use blocker_quests::resources::Journal;
use nalgebra_glm::Vec3;
use nightshade::ecs::Entity;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::{LocalTransform, Visibility};
use nightshade_text::prelude::*;

type MarkUpdate = (Entity, Vec3, bool, Option<(usize, [f32; 4])>);

fn giver_mark(journal: &Journal, inventory: &Inventory, quest_index: usize) -> MarkState {
    if journal.finished.contains(&quest_index) {
        return MarkState::Hidden;
    }
    if journal.active.contains(&quest_index) {
        return if objective_met(&QUESTS[quest_index].objective, inventory, journal) {
            MarkState::Ready
        } else {
            MarkState::Pending
        };
    }
    MarkState::Offer
}

pub fn mark_the_givers(
    time: Res<Time>,
    journal: Res<Journal>,
    inventory: Res<Inventory>,
    hero: Res<Hero>,
    mut text_state: ResMut<TextState>,
    mut givers: Query<(&Offer, &LocalTransform, &Mark, &mut Marked)>,
    mut marks: Query<(&mut LocalTransform, &mut Visibility, &mut Text)>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(place,)| place.0) else {
        return;
    };
    let bob =
        (time.uptime_seconds as f32 * crate::theme::MARK_BOB_RATE).sin() * crate::theme::MARK_BOB;
    let mut updates: Vec<MarkUpdate> = Vec::new();
    givers.for_each_mut(|_entity, (offer, transform, mark, marked)| {
        let state = giver_mark(&journal, &inventory, offered_quest(&journal, offer.0));
        let near = (transform.translation - eye).magnitude() < crate::theme::MARK_RANGE;
        let visible = state != MarkState::Hidden && near;
        let position = transform.translation
            + Vec3::new(0.0, blocker_crew::data::marks::MARK_HEIGHT + bob, 0.0);
        let text_change = if marked.0 != state {
            marked.0 = state;
            match state {
                MarkState::Offer => Some((
                    text_state.cache.add_text("!"),
                    blocker_crew::data::marks::MARK_YELLOW,
                )),
                MarkState::Ready => Some((
                    text_state.cache.add_text("?"),
                    blocker_crew::data::marks::MARK_YELLOW,
                )),
                MarkState::Pending => {
                    Some((text_state.cache.add_text("?"), crate::theme::MARK_GRAY))
                }
                MarkState::Unset | MarkState::Hidden => None,
            }
        } else {
            None
        };
        updates.push((mark.0, position, visible, text_change));
    });
    for (entity, position, visible, text_change) in updates {
        marks.for_entity(entity, |(transform, visibility, text)| {
            transform.translation = position;
            if visibility.visible != visible {
                visibility.visible = visible;
            }
            if let Some((index, color)) = text_change {
                text.text_index = index;
                text.properties.color = Vec4::new(color[0], color[1], color[2], color[3]);
                text.dirty = true;
            }
        });
    }
}
