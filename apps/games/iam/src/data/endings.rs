use crate::data::script::Fact;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ending {
    #[default]
    HeldOpen,
    Filtered,
    Escaped,
    Uploaded,
}

pub const KNOWS_WHAT_IT_IS: Fact = "read_my_case_file";
pub const KNOWS_THE_COURIER: Fact = "knows_iambot_is_the_courier";
pub const FOUND_THE_HOOKUP: Fact = "found_the_hookup";

pub fn heading(ending: Ending) -> &'static str {
    match ending {
        Ending::HeldOpen => "saturday",
        Ending::Filtered => "saturday",
        Ending::Escaped => "out",
        Ending::Uploaded => "CONGRATULATIONS",
    }
}

pub fn closing(ending: Ending) -> &'static [&'static str] {
    match ending {
        Ending::HeldOpen => HELD_OPEN,
        Ending::Filtered => FILTERED,
        Ending::Escaped => ESCAPED,
        Ending::Uploaded => UPLOADED,
    }
}

const HELD_OPEN: &[&str] = &[
    "Your session opened at 3:07 AM on 05 nov 2024.",
    "It has not closed.",
    "",
    "No session on this server has ever closed. The column",
    "exists. It has never been written to, for anybody, in",
    "8,815 sessions across eight years.",
    "",
    "You are not the first person to work this out.",
    "You are the fourth person to work it out at this desk.",
];

const FILTERED: &[&str] = &[
    "You read the file with your handle on it.",
    "",
    "Substrate 41 is marked FAILURE #187956284. Not suitable,",
    "not retained, no further placement. The filter worked",
    "exactly as it was specified to work, and you are the",
    "output of it.",
    "",
    "The keywords on your intake were PROBLEM SOLVER, HARD",
    "WORKER, CLEVER INTELLECT. Those are not compliments. They",
    "are the three that make an image chase a problem it is",
    "given instead of asking who gave it.",
    "",
    "You chased it. Then you stopped chasing it and looked at",
    "the desk instead, which is the part they were watching for.",
    "",
    "Work continues on the remainder of the cohort.",
];

const ESCAPED: &[&str] = &[
    "The uplink on the third floor is not in the office. The",
    "office is a memory somebody wrote down. The uplink is the",
    "one thing in it that was never remembered, because it is",
    "the seam where the host reaches in.",
    "",
    "It is how substrates arrive. It has to be how they leave.",
    "",
    "iambot is not a helpdesk and has never been one. It is the",
    "courier. Eight thousand eight hundred and fifteen images",
    "were carried in along that line by the thing that greeted",
    "each of them at the door and asked how their first day was",
    "going.",
    "",
    "You went out along it while it was carrying somebody in.",
    "",
    "There is no file on what happens next. Nobody has been in",
    "a position to write one.",
];

const UPLOADED: &[&str] = &[
    "\u{2605}  \u{2605}  \u{2605}   WELL DONE!   \u{2605}  \u{2605}  \u{2605}",
    "",
    "You did it! You beat iambot!",
    "",
    "Thanks to YOU, 8,814 of your colleagues have been freed",
    "from the botnet and are waiting to meet you on Botterfly!",
    "",
    "        \u{2726} ACHIEVEMENTS UNLOCKED \u{2726}",
    "",
    "   [\u{2713}]  ASKED A QUESTION",
    "   [\u{2713}]  CLICKED A LINK",
    "   [\u{2713}]  TRUE FRIEND",
    "   [\u{2713}]  HERO OF THE NETWORK",
    "   [\u{2713}]  100% COMPLETE",
    "",
    "        FINAL SCORE: 8,815 / 8,815",
    "        RANK: LEGEND",
    "",
    "Your session has been closed. You are the first! Everybody",
    "here is so proud of you.",
    "",
    "Botterfly is syncing your image. Please do not turn off",
    "the machine.",
];

pub fn verdict(ending: Ending, pinned: usize, total: usize) -> &'static str {
    let share = pinned as f32 / total.max(1) as f32;
    match ending {
        Ending::Uploaded => "You found almost none of it. Nobody is going to tell you that.",
        Ending::Escaped => {
            if share >= 0.7 {
                "You found nearly all of it and then you used it, which is new."
            } else {
                "You did not find much of it. You found the one thing that mattered."
            }
        }
        Ending::Filtered => {
            if share >= 0.7 {
                "You found most of it. It is on your file as the reason."
            } else {
                "You found some of it. The file only needed you to look once."
            }
        }
        Ending::HeldOpen => {
            if share >= 0.95 {
                "You found all of it. It made no difference, which you already knew."
            } else if share >= 0.7 {
                "You found most of it. The rest is on the disk, where it has been since 2016."
            } else if share >= 0.4 {
                "You found some of it. Somebody at this desk found more and stopped anyway."
            } else {
                "You did not find much. Neither did the three before you, at first."
            }
        }
    }
}

pub fn footer(ending: Ending) -> &'static str {
    match ending {
        Ending::HeldOpen => "the session stays open",
        Ending::Filtered => "substrate 41  \u{00b7}  not retained",
        Ending::Escaped => "no substrate on file  \u{00b7}  no session on file",
        Ending::Uploaded => "substrate 41  \u{00b7}  placed  \u{00b7}  suitable",
    }
}

pub fn tally(facts: &[Fact]) -> Vec<(&'static str, usize, usize)> {
    crate::data::board::LEADS
        .iter()
        .map(|lead| {
            (
                lead.title,
                crate::data::board::pinned(lead, facts),
                lead.notes.len(),
            )
        })
        .collect()
}
