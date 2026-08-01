use crate::resources::{Deed, Speech};
use crate::theme;
use blocker_beasts::data::beasts::BEASTS;
use blocker_crew::components::{Offer, Persona, Shop};
use blocker_crew::data::marks::HEAD_CENTER;
use blocker_crew::data::people::FOLK;
use blocker_figure::components::{Eye, Gaze, Home};
use blocker_figure::queries::{looks_at, read_the_hero};
use blocker_figure::resources::Hero;
use blocker_player::data::ask::Ask;
use blocker_player::data::offered::Offered;
use blocker_pocket::resources::Inventory;
use blocker_quests::data::quests::{Objective, QUESTS};
use blocker_quests::data::spots::PLACES;
use blocker_quests::data::trade::GIVERS;
use blocker_quests::queries::{objective_met, objective_progress, offered_quest};
use blocker_quests::resources::Journal;
use blocker_screen::data::screen::Screen;
use blocker_screen::queries::page_the_speech;
use nalgebra_glm::Vec3;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_dialogue::commands::{offer, open};
use nightshade_input::Actions;
use nightshade_random::Random;
use nightshade_random::commands::index;
use nightshade_scene::prelude::{LocalTransform, Name};
use nightshade_state::prelude::*;

#[derive(Clone, Copy)]
enum Part {
    Folk(usize),
    Giver(usize),
    Trader(usize),
}

type Talkers<'world, 'item> = Query<
    'world,
    (
        &'item LocalTransform,
        &'item Name,
        Option<&'item Persona>,
        Option<&'item Offer>,
        Option<&'item Shop>,
    ),
    With<Home>,
>;

pub fn talk_to_the_crew(
    actions: Res<Actions<Ask>>,
    inventory: Res<Inventory>,
    hero: Res<Hero>,
    mut journal: ResMut<Journal>,
    mut speech: Speech,
    mut next: ResMut<NextState<Screen>>,
    mut random: ResMut<Random>,
    mut offers: EventWriter<Offered>,
    talkers: Talkers<'_, '_>,
    mut sighted: Query<(&Eye, &Gaze)>,
) {
    let Some((eye, gaze)) = read_the_hero(&hero, &mut sighted, |(place, ahead)| (place.0, ahead.0))
    else {
        return;
    };
    let mut target: Option<(Part, String)> = None;
    let mut nearest = f32::MAX;
    talkers.for_each(|_entity, (transform, name, persona, offer, shop)| {
        let part = match (offer, shop, persona) {
            (Some(offer), _, _) => Part::Giver(offered_quest(&journal, offer.0)),
            (_, Some(shop), _) => Part::Trader(shop.0),
            (_, _, Some(persona)) => Part::Folk(persona.0),
            _ => return,
        };
        let head = transform.translation + Vec3::new(0.0, HEAD_CENTER, 0.0);
        if !looks_at(
            eye,
            gaze,
            head,
            blocker_figure::data::sight::REACH,
            blocker_figure::data::sight::LOOK_COSINE,
        ) {
            return;
        }
        let distance = (head - eye).magnitude();
        if distance < nearest {
            nearest = distance;
            target = Some((part, name.0.clone()));
        }
    });
    let Some((part, name)) = target else {
        return;
    };
    offers.send(Offered(name.clone()));
    if !actions.just_pressed(Ask::Act) {
        return;
    }
    match part {
        Part::Folk(persona) => speak_with_folk(&mut random, &mut speech, &mut next, persona, name),
        Part::Giver(quest) => speak_with_giver(
            &inventory,
            &mut journal,
            &mut speech,
            &mut next,
            quest,
            name,
        ),
        Part::Trader(trader) => {
            speech.shopping.trader = trader;
            next.pending = Some(Screen::Shop);
        }
    }
}

fn speak_with_folk(
    random: &mut Random,
    speech: &mut Speech<'_>,
    next: &mut NextState<Screen>,
    persona: usize,
    name: String,
) {
    let lines = FOLK[persona].lines;
    if lines.is_empty() {
        return;
    }
    let body = lines[index(random, lines.len())].to_string();
    open_dialogue(
        speech,
        next,
        name,
        body,
        String::new(),
        vec![("Good day to you.".to_string(), Deed::Close)],
    );
}

fn speak_with_giver(
    inventory: &Inventory,
    journal: &mut Journal,
    speech: &mut Speech<'_>,
    next: &mut NextState<Screen>,
    quest_index: usize,
    name: String,
) {
    let quest = &QUESTS[quest_index];
    journal.record_met(quest.giver);
    let mut choices = Vec::new();
    let (body, objective, close) = if journal.finished.contains(&quest_index) {
        (
            "Thanks again. The roads are safer for it.".to_string(),
            String::new(),
            "Farewell.",
        )
    } else if journal.active.contains(&quest_index) {
        if objective_met(&quest.objective, inventory, journal) {
            choices.push(("It is done.".to_string(), Deed::Complete(quest_index)));
            (
                quest.complete.join("\n\n"),
                "Ready to hand in.".to_string(),
                "I will be on my way.",
            )
        } else {
            (
                quest.nudge.to_string(),
                objective_line(&quest.objective, inventory, journal),
                "I am still working on it.",
            )
        }
    } else {
        choices.push((
            "I will take the job.".to_string(),
            Deed::Accept(quest_index),
        ));
        let mut lines = vec![quest.story.to_string()];
        lines.extend(quest.accept.iter().map(|line| line.to_string()));
        (
            lines.join("\n\n"),
            objective_line(&quest.objective, inventory, journal),
            "Not today.",
        )
    };
    choices.push((close.to_string(), Deed::Close));
    open_dialogue(speech, next, name, body, objective, choices);
}

fn objective_line(objective: &Objective, inventory: &Inventory, journal: &Journal) -> String {
    let (have, want) = objective_progress(objective, inventory, journal);
    match objective {
        Objective::Gather { item, .. } => format!("{}: {have}/{want}", item.name()),
        Objective::Explore { .. } => format!("Places found: {have}/{want}"),
        Objective::Slay { beast, .. } => format!("{} slain: {have}/{want}", BEASTS[*beast].name),
        Objective::Meet { giver } => format!("Speak with {}", GIVERS[*giver].name),
        Objective::Visit { place } => format!("Go to {}", PLACES[*place].name),
        Objective::Report => "Report what you found".to_string(),
    }
}

fn open_dialogue(
    speech: &mut Speech<'_>,
    next: &mut NextState<Screen>,
    name: String,
    body: String,
    objective: String,
    choices: Vec<(String, Deed)>,
) {
    let pages = page_the_speech(
        &body,
        theme::DIALOGUE_LINE_CHARACTERS,
        theme::DIALOGUE_PAGE_LINES,
    );
    let lines: Vec<&str> = pages.iter().map(String::as_str).collect();
    open(speech.conversation, name, &lines);
    let labels: Vec<&str> = choices.iter().map(|(label, _)| label.as_str()).collect();
    offer(speech.conversation, &labels);
    speech.deeds.0 = choices.into_iter().map(|(_, deed)| deed).collect();
    speech.objective.0 = objective;
    next.pending = Some(Screen::Dialogue);
}
