use crate::commands::open_chapter;
use crate::data::CHAPTERS;
use crate::queries::Fighters;
use crate::resources::Campaign;

pub fn take_the_next_chapter(campaign: &mut Campaign, won: bool) -> bool {
    if won && campaign.chapter + 1 >= CHAPTERS.len() {
        return false;
    }
    let next = if won {
        campaign.chapter + 1
    } else {
        campaign.chapter
    };
    open_chapter(campaign, next);
    true
}

pub fn heal_the_camp(fighters: &mut Fighters<'_, '_>) {
    crate::commands::edit_fighters(fighters, |fighter| {
        fighter.wounded = 0;
        fighter.busy = false;
    });
}
