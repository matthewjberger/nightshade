use crate::commands::{
    add_experience, advance_day, lift_morale, raise_bond, raise_rapport, set_camp_phase, speak,
    spend, story, wake_the_camp,
};
use crate::commands::{edit_fighter, edit_fighters};
use crate::data::{Activity, Bond, CHAPTERS, COMPANIONS, RoomKind};
use crate::queries::Fighters;
use crate::queries::camp::{arc_lines, deploy_limit};
use crate::queries::{mustered, mustering};
use crate::resources::{CampPhase, Campaign};
use crate::theme;
use nightshade::prelude::*;

pub fn enter_the_room(campaign: &mut Campaign, fighters: &mut Fighters<'_, '_>, kind: RoomKind) {
    let squad = mustering(fighters);
    let actions = campaign.actions;
    let supplies = campaign.supplies;
    if kind.actions() > actions {
        story(campaign, "There is no time left today.");
        return;
    }
    if kind.supplies() > supplies {
        story(campaign, "The stores will not cover that.");
        return;
    }
    let mut opened: Vec<(String, String)> = Vec::new();
    let line = match kind {
        RoomKind::Command => {
            campaign.menu_index = 0;
            set_camp_phase(campaign, CampPhase::Muster);
            return;
        }
        RoomKind::Quarters => {
            close_the_day(campaign, fighters);
            return;
        }
        RoomKind::Mess => {
            let names: Vec<Entity> = squad.iter().map(|(entity, _)| *entity).collect();
            for entity in &names {
                let mut reached = Vec::new();
                let mut slot = 0;
                edit_fighter(fighters, *entity, |fighter| {
                    slot = fighter.slot;
                    reached = raise_rapport(fighter, theme::SUPPER_RAPPORT);
                    lift_morale(fighter, theme::MORALE_ACTIVITY);
                });
                let companion = &COMPANIONS[slot.min(COMPANIONS.len() - 1)];
                opened.extend(arc_lines(companion.name, companion.arc, &reached));
                for other in &names {
                    if other == entity {
                        continue;
                    }
                    let mate = *other;
                    edit_fighter(fighters, *entity, |fighter| {
                        raise_bond(fighter, mate, theme::SUPPER_BOND)
                    });
                }
            }
            "The squad eats together and the talk runs late.".to_string()
        }
        RoomKind::Yard => {
            for (entity, fighter) in &squad {
                if !fighter.deployed {
                    continue;
                }
                let entity = *entity;
                edit_fighter(fighters, entity, |fighter| {
                    add_experience(fighter, theme::DRILL_EXPERIENCE);
                    lift_morale(fighter, theme::MORALE_ACTIVITY / 2);
                });
            }
            "The mustered squad drills until dusk.".to_string()
        }
        RoomKind::Workshop => {
            let mut mended = 0;
            for (entity, fighter) in &squad {
                if fighter.wounded == 0 {
                    continue;
                }
                mended += 1;
                let entity = *entity;
                edit_fighter(fighters, entity, |fighter| {
                    fighter.wounded = fighter.wounded.saturating_sub(1);
                    lift_morale(fighter, theme::MORALE_ACTIVITY);
                });
            }
            if mended > 0 {
                format!("The workshop patches up {mended} of the squad.")
            } else {
                "Nobody needs the workshop today.".to_string()
            }
        }
        RoomKind::Gate => {
            campaign.march = true;
            story(campaign, "The squad forms up at the gate and marches out.");
            return;
        }
        RoomKind::Dock => {
            campaign.supplies += theme::STORES_SUPPLIES;
            format!(
                "The boat comes back with {} supplies.",
                theme::STORES_SUPPLIES
            )
        }
    };
    spend(campaign, kind.actions(), kind.supplies());
    story(campaign, line);
    speak(campaign, opened);
}

pub fn close_the_day(campaign: &mut Campaign, fighters: &mut Fighters<'_, '_>) {
    let (day, chapter) = (
        campaign.day,
        CHAPTERS[campaign.chapter.min(CHAPTERS.len() - 1)].days,
    );
    if day >= chapter {
        campaign.march = true;
        story(campaign, "Dawn. The squad forms up and marches out.");
        return;
    }
    edit_fighters(fighters, wake_the_camp);
    advance_day(campaign);
    let day = campaign.day;
    story(campaign, format!("Night passes. Day {day} in camp."));
}

pub fn apply_alone(
    campaign: &mut Campaign,
    fighters: &mut Fighters<'_, '_>,
    entity: Entity,
    slot: usize,
    activity: Activity,
) {
    let companion = &COMPANIONS[slot.min(COMPANIONS.len() - 1)];
    let mut line = String::new();
    let mut opened = Vec::new();
    edit_fighter(fighters, entity, |fighter| match activity {
        Activity::Advance => {}
        Activity::Talk => {
            let rank = Bond::of(fighter.rapport).rank().min(3) as usize;
            line = companion.lines[rank].to_string();
            opened = raise_rapport(fighter, theme::TALK_RAPPORT);
            lift_morale(fighter, theme::MORALE_ACTIVITY);
        }
        Activity::Train => {
            add_experience(fighter, theme::TRAIN_EXPERIENCE);
            fighter.busy = true;
            lift_morale(fighter, theme::MORALE_ACTIVITY);
            line = format!("{} drills on the range.", fighter.name);
        }
        Activity::Gift => {
            opened = raise_rapport(fighter, theme::GIFT_RAPPORT + theme::GIFT_TASTE_RAPPORT);
            lift_morale(fighter, theme::MORALE_ACTIVITY * 2);
            line = format!(
                "{} unwraps {} and keeps it close.",
                fighter.name,
                fighter.taste.gift()
            );
        }
        Activity::Muster => {
            fighter.deployed = !fighter.deployed;
            line = if fighter.deployed {
                format!("{} joins the muster.", fighter.name)
            } else {
                format!("{} stands down.", fighter.name)
            };
        }
        Activity::Spar => {}
    });

    if activity == Activity::Muster {
        let limit = deploy_limit(campaign);
        let squad = mustering(fighters);
        if mustered(&squad) > limit {
            edit_fighter(fighters, entity, |fighter| fighter.deployed = false);
            line = format!("The muster holds only {limit}.");
        }
    } else {
        let spent = activity.supplies();
        spend(campaign, activity.actions(), spent);
    }
    let scenes = arc_lines(companion.name, companion.arc, &opened);
    if matches!(activity, Activity::Talk | Activity::Gift) {
        let mut spoken = vec![(companion.name.to_string(), line)];
        spoken.extend(scenes);
        speak(campaign, spoken);
        return;
    }
    story(campaign, line);
    speak(campaign, scenes);
}

pub fn choose_row(
    campaign: &mut Campaign,
    camp: &crate::resources::Camp,
    frame: &nightshade_ui::RetainedUiFrame,
    buttons: &[Entity],
    ready: &[bool],
) -> Option<usize> {
    let (up, down, confirm, _) = crate::queries::camp::menu_keys(camp);
    let mut picked = None;
    for (item, button) in buttons.iter().enumerate() {
        if crate::queries::menu::clicked(frame, Some(*button))
            && ready.get(item).copied().unwrap_or(false)
        {
            picked = Some(item);
        }
    }
    if campaign.menu_index >= ready.len() {
        campaign.menu_index = 0;
    }
    if up {
        campaign.menu_index = crate::queries::menu::step_index(campaign.menu_index, ready, false);
    }
    if down {
        campaign.menu_index = crate::queries::menu::step_index(campaign.menu_index, ready, true);
    }
    if confirm && picked.is_none() && ready.get(campaign.menu_index) == Some(&true) {
        picked = Some(campaign.menu_index);
    }
    picked
}
