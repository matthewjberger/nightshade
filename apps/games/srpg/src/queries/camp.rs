use crate::data::Fighter;
use crate::data::{ACTIVITIES, Activity, ArcStage, CHAPTERS, UPGRADES, Upgrade};
use crate::queries::menu::activity_items;
use crate::queries::{Squad, mustered, ready_to_deploy};
use crate::resources::{Camp, CampPhase, Campaign};
use crate::theme;

pub fn arc_lines(
    name: &'static str,
    arc: &'static str,
    stages: &[&'static ArcStage],
) -> Vec<(String, String)> {
    let mut lines = Vec::new();
    for stage in stages {
        lines.push((
            format!("{name}  ({arc}: {})", stage.title),
            stage.line.to_string(),
        ));
    }
    lines
}

pub fn menu_keys(camp: &Camp) -> (bool, bool, bool, bool) {
    (camp.step > 0, camp.step < 0, camp.confirm, camp.cancel)
}

pub fn deploy_limit(campaign: &Campaign) -> usize {
    CHAPTERS[campaign.chapter.min(CHAPTERS.len() - 1)].deploy
}

pub fn upgrade_options(fighter: &Fighter) -> Vec<Upgrade> {
    UPGRADES
        .iter()
        .filter(|upgrade| !fighter.taken.contains(upgrade))
        .copied()
        .collect()
}

pub fn menu_rows(
    campaign: &Campaign,
    squad: &Squad,
    holder: Option<usize>,
    deploy: usize,
) -> Vec<(String, bool)> {
    match campaign.phase {
        CampPhase::Menu => {
            let Some(index) = holder else {
                return Vec::new();
            };
            let fighter = &squad[index].1;
            let enabled = activity_items(campaign, fighter);
            ACTIVITIES
                .iter()
                .enumerate()
                .map(|(item, activity)| {
                    let label = match activity {
                        Activity::Muster => {
                            if fighter.deployed {
                                "Stand down".to_string()
                            } else {
                                "Muster".to_string()
                            }
                        }
                        Activity::Gift => format!("Give a gift   {} supplies", theme::GIFT_COST),
                        Activity::Advance => {
                            format!("Advance   {} marks", fighter.marks)
                        }
                        other => other.label().to_string(),
                    };
                    (label, enabled[item])
                })
                .collect()
        }
        CampPhase::Partner => squad
            .iter()
            .map(|(entity, fighter)| {
                let ready = ready_to_deploy(fighter)
                    && !fighter.busy
                    && Some(*entity) != campaign.partner_of;
                (fighter.name.to_string(), ready)
            })
            .collect(),
        CampPhase::Muster => {
            let filled = mustered(squad);
            squad
                .iter()
                .map(|(_, fighter)| {
                    let mark = if fighter.deployed { "*" } else { " " };
                    let room = fighter.deployed || filled < deploy;
                    (
                        format!("{} {}", mark, fighter.name),
                        ready_to_deploy(fighter) && room,
                    )
                })
                .collect()
        }
        CampPhase::Advance => {
            let Some(index) = holder else {
                return Vec::new();
            };
            upgrade_options(&squad[index].1)
                .into_iter()
                .map(|upgrade| (upgrade.label().to_string(), true))
                .collect()
        }
        CampPhase::Roam | CampPhase::Speaking => Vec::new(),
    }
}

pub fn row_flags(
    campaign: &Campaign,
    squad: &Squad,
    holder: Option<usize>,
    deploy: usize,
) -> Vec<bool> {
    menu_rows(campaign, squad, holder, deploy)
        .into_iter()
        .map(|(_, ready)| ready)
        .collect()
}
