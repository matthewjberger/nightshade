use crate::data::SCRIPT;
use crate::data::board::LEADS;
use crate::data::case::DOSSIERS;
use crate::data::cross::PAIRINGS;
use crate::data::files::LEFTOVERS;
use crate::data::games::{Board, POSTED, solved, turned};
use crate::data::machine::{ACCOUNTS, opens};
use crate::data::office::{PLACES, THINGS, place_at, thing_at};
use crate::data::programs::{PROGRAMS, index_of};
use crate::data::script::{Effect, Fact};
use crate::data::store::LISTINGS;
use crate::data::web::{BOOKMARKS, Block, HOME, PAGES, page_at, tidy};

const TURNS_BEFORE_IT_IS_STUCK: usize = 64;

fn collect(effects: &'static [Effect], facts: &mut Vec<Fact>) {
    for effect in effects {
        if let Effect::Assert(fact) = effect {
            facts.push(fact);
        }
    }
}

fn establishable() -> Vec<Fact> {
    let mut facts: Vec<Fact> = Vec::new();

    for definition in SCRIPT.contacts {
        for node in definition.nodes {
            collect(node.effects, &mut facts);
            for choice in node.choices {
                facts.extend(choice.asserts);
            }
        }
    }
    for night in SCRIPT.nights {
        collect(night.opening, &mut facts);
        facts.push(night.day);
    }
    for definition in SCRIPT.servers {
        for channel in definition.channels {
            collect(channel.on_open, &mut facts);
            for question in channel.questions {
                facts.extend(question.asserts);
                collect(question.effects, &mut facts);
            }
        }
    }
    facts.extend(crate::data::script::established_elsewhere());
    facts.sort_unstable();
    facts.dedup();
    facts
}

#[test]
fn the_script_agrees_with_itself() {
    crate::data::script::validate(&SCRIPT);
}

#[test]
fn every_card_on_the_board_can_be_pinned() {
    let known = establishable();
    for lead in LEADS {
        for note in lead.notes {
            assert!(
                known.contains(&note.fact),
                "the card \"{}\" waits on {}, which nothing establishes",
                note.text,
                note.fact,
            );
        }
    }
}

#[test]
fn every_lead_points_at_a_lead() {
    for lead in LEADS {
        for opens in lead.opens {
            assert!(
                LEADS.iter().any(|other| other.id == *opens),
                "{} points at {opens}, which is not a lead",
                lead.id,
            );
        }
    }
}

#[test]
fn every_note_on_the_desk_can_be_read() {
    let known = establishable();
    for note in crate::data::notes::NOTES {
        for fact in note.requires {
            assert!(
                known.contains(fact),
                "a note by {} waits on {fact}, which nothing establishes",
                note.author,
            );
        }
    }
}

#[test]
fn every_way_out_of_a_room_leads_somewhere() {
    for place in PLACES {
        for way in place.ways {
            assert!(
                place_at(way.goes_to).is_some(),
                "{} offers \"{}\", which goes to {}, which is not a place",
                place.id,
                way.label,
                way.goes_to,
            );
        }
        for thing in place.things {
            assert!(
                thing_at(thing).is_some(),
                "{} lists {thing}, which is not a thing",
                place.id,
            );
        }
    }
}

#[test]
fn every_room_in_the_building_can_be_reached() {
    let mut reached = vec![false; PLACES.len()];
    reached[0] = true;
    let mut walking = vec![0usize];
    while let Some(index) = walking.pop() {
        for way in PLACES[index].ways {
            let Some(target) = place_at(way.goes_to) else {
                continue;
            };
            if !reached[target] {
                reached[target] = true;
                walking.push(target);
            }
        }
    }
    for (index, found) in reached.iter().enumerate() {
        assert!(
            *found,
            "nothing leads to {}, so nobody can walk there",
            PLACES[index].id,
        );
    }
}

#[test]
fn every_thing_in_the_building_is_somewhere() {
    for thing in THINGS {
        assert!(
            PLACES.iter().any(|place| place.things.contains(&thing.id)),
            "{} is not in any room, so nothing can look at it",
            thing.id,
        );
    }
}

#[test]
fn every_room_shows_all_of_its_ways_out() {
    for place in PLACES {
        assert!(
            place.ways.len() <= crate::data::office::MOST_WAYS,
            "{} has more ways out than the walk draws rows for",
            place.id,
        );
        assert!(
            place.things.len() <= crate::data::office::MOST_THINGS,
            "{} has more to look at than the walk draws rows for",
            place.id,
        );
    }
}

#[test]
fn every_posted_game_was_posted_in_a_room() {
    for posted in POSTED {
        assert!(
            SCRIPT
                .servers
                .iter()
                .any(|server| server.channels.iter().any(|room| room.name == posted.room)),
            "{} was posted in {}, which is not a room",
            posted.id,
            posted.room,
        );
    }
}

#[test]
fn every_posted_board_can_be_finished() {
    for posted in POSTED {
        if posted.installs.is_some() {
            assert!(
                posted.start.is_empty() && posted.goal.is_empty(),
                "{} hands over a program, so it has no board",
                posted.id,
            );
            continue;
        }
        assert_eq!(
            posted.start.chars().count(),
            posted.goal.chars().count(),
            "{} starts and finishes at different lengths",
            posted.id,
        );
        let mut squares: Vec<char> = posted.start.chars().collect();
        for (place, wanted) in posted.goal.chars().enumerate() {
            let mut presses = 0;
            while squares[place] != wanted {
                squares[place] = turned(posted.board, squares[place]);
                presses += 1;
                assert!(
                    presses <= TURNS_BEFORE_IT_IS_STUCK,
                    "square {place} of {} never reads {wanted}",
                    posted.id,
                );
            }
        }
        let reads: String = squares.into_iter().collect();
        assert!(solved(posted, &reads), "{} cannot be finished", posted.id);
    }
}

#[test]
fn a_square_always_comes_back_to_itself() {
    for board in [Board::Rotate, Board::Count] {
        for face in ['a', 'z', '0', '9'] {
            let mut turning = face;
            let mut presses = 0;
            loop {
                turning = turned(board, turning);
                presses += 1;
                if turning == face || presses > TURNS_BEFORE_IT_IS_STUCK {
                    break;
                }
            }
            assert_eq!(turning, face, "{face} never comes back on its own board");
        }
    }
}

#[test]
fn every_listing_installs_a_program_that_exists() {
    for listing in LISTINGS {
        let Some(program) = listing.program else {
            continue;
        };
        assert!(
            index_of(program).is_some(),
            "{} installs {program}, which is not a program",
            listing.title,
        );
    }
}

#[test]
fn every_program_from_the_list_is_on_the_list() {
    for program in PROGRAMS {
        if !program.from_store {
            continue;
        }
        assert!(
            LISTINGS
                .iter()
                .any(|listing| listing.program == Some(program.id)),
            "{} is only got from the list, and the list has never heard of it",
            program.id,
        );
    }
}

#[test]
fn the_list_shows_everything_the_network_will_give_you() {
    assert!(
        LISTINGS.len() <= crate::data::store::MOST_LISTINGS,
        "the list holds more than it draws rows for",
    );
}

#[test]
fn every_file_on_the_disk_belongs_to_an_account() {
    for file in LEFTOVERS {
        assert!(
            ACCOUNTS.iter().any(|account| account.id == file.owner),
            "{} sits in {}, which is not an account on this machine",
            file.name,
            file.owner,
        );
    }
}

#[test]
fn every_password_opens_one_account_and_no_other() {
    for account in ACCOUNTS {
        let opened: Vec<&str> = ACCOUNTS
            .iter()
            .filter(|other| opens(other, account.password))
            .map(|other| other.id)
            .collect();
        assert_eq!(
            opened,
            vec![account.id],
            "the password for {} opens {opened:?}",
            account.id,
        );
    }
}

#[test]
fn every_locked_account_says_what_it_wants() {
    let known = establishable();
    for account in ACCOUNTS {
        if account.understands.is_empty() {
            continue;
        }
        assert!(
            !account.locked_note.is_empty(),
            "{} can be worked out and says nothing about how",
            account.id,
        );
        for fact in account.understands {
            assert!(
                known.contains(fact),
                "{} waits on {fact}, which nothing establishes",
                account.id,
            );
        }
    }
}

#[test]
fn every_page_can_be_reached() {
    let mut reached = vec![false; PAGES.len()];
    let home = page_at(HOME).expect("the intranet to be a page");
    reached[home] = true;
    let mut walking = vec![home];
    while let Some(index) = walking.pop() {
        let page = &PAGES[index];
        let going: Vec<&'static str> = page
            .links
            .iter()
            .copied()
            .chain(page.body.iter().filter_map(|block| match block {
                Block::Row { goes_to, .. } => Some(*goes_to),
                _ => None,
            }))
            .collect();
        for url in going {
            let Some(target) = page_at(url) else {
                panic!("{} links to {url}, which is not a page", page.url);
            };
            if !reached[target] {
                reached[target] = true;
                walking.push(target);
            }
        }
    }
    for mark in BOOKMARKS {
        if let Some(index) = page_at(mark.url) {
            reached[index] = true;
        }
    }
    for (index, found) in reached.iter().enumerate() {
        assert!(
            *found,
            "nothing links to {} and it is not bookmarked, so nobody can go there",
            PAGES[index].url,
        );
    }
}

#[test]
fn every_bookmark_points_at_a_page() {
    for mark in BOOKMARKS {
        assert!(
            page_at(mark.url).is_some(),
            "the bookmark \"{}\" points at {}, which is not a page",
            mark.label,
            mark.url,
        );
    }
}

#[test]
fn every_address_is_written_the_way_it_is_typed() {
    for page in PAGES {
        assert_eq!(
            tidy(page.url),
            page.url,
            "{} is not the address the browser would make of it",
            page.url,
        );
    }
}

#[test]
fn every_square_on_tonights_note_has_words_for_it() {
    let known = establishable();
    for night in SCRIPT.nights {
        for wanted in night.condition.requires {
            assert!(
                known.contains(wanted),
                "{} waits on {wanted}, which nothing establishes",
                night.day,
            );
            assert!(
                crate::data::tonight::LINES
                    .iter()
                    .any(|gate| gate.fact == *wanted),
                "{} waits on {wanted}, which the note has no words for",
                night.day,
            );
        }
        assert!(
            night.condition.requires.len() <= crate::data::tonight::ROWS,
            "{} wants more than the note has room for",
            night.day,
        );
    }
}

#[test]
fn every_line_on_the_note_is_wanted_by_a_night() {
    for gate in crate::data::tonight::LINES {
        assert!(
            SCRIPT
                .nights
                .iter()
                .any(|night| night.condition.requires.contains(&gate.fact)),
            "the note has words for {}, which no night waits on",
            gate.fact,
        );
    }
}

#[test]
fn every_pairing_can_be_made() {
    let known = establishable();
    for pairing in PAIRINGS {
        for fact in pairing.requires {
            assert!(
                known.contains(fact),
                "the pairing from {} waits on {fact}, which nothing establishes",
                pairing.source,
            );
        }
    }
}

#[test]
fn every_remark_the_host_keeps_can_be_earned() {
    let known = establishable();
    for (fact, _) in crate::systems::conversation::REMARKED_ON {
        assert!(
            known.contains(fact),
            "the host has a line about {fact}, which nothing establishes",
        );
    }
}

#[test]
fn every_ending_the_demonstration_offers_is_an_ending() {
    let mut seen: Vec<crate::data::endings::Ending> = Vec::new();
    for (ending, label, _) in crate::data::autoplay::OFFERED {
        assert!(!label.is_empty(), "an offered ending has no name");
        assert!(!seen.contains(ending), "{label} is offered twice");
        seen.push(*ending);
    }
}

#[test]
fn everything_the_demonstration_looks_for_can_be_found() {
    let known = establishable();
    for (fact, _) in crate::data::autoplay::KEPT {
        assert!(
            known.contains(fact),
            "the demonstration looks for {fact}, which nothing establishes",
        );
    }
    for ending in [
        crate::data::endings::Ending::HeldOpen,
        crate::data::endings::Ending::Uploaded,
        crate::data::endings::Ending::Filtered,
        crate::data::endings::Ending::Escaped,
    ] {
        for fact in crate::data::autoplay::also_wants(ending) {
            assert!(
                known.contains(fact),
                "playing to {ending:?} wants {fact}, which nothing establishes",
            );
        }
    }
}

#[test]
fn getting_out_waits_on_things_that_can_be_found() {
    let known = establishable();
    for fact in [
        crate::data::endings::KNOWS_WHAT_IT_IS,
        crate::data::endings::KNOWS_THE_COURIER,
        crate::data::endings::FOUND_THE_HOOKUP,
    ] {
        assert!(
            known.contains(&fact),
            "getting out waits on {fact}, which nothing establishes",
        );
    }
}

#[test]
fn the_lamp_program_never_runs_out_of_boards() {
    assert!(
        !crate::data::lights::BOARDS.is_empty(),
        "there are no boards"
    );
    for done in 0..crate::data::lights::BOARDS.len() * 3 {
        let board = crate::data::lights::next_board(done);
        assert_eq!(
            board.chars().count(),
            crate::data::lights::DARK.chars().count(),
            "board {done} is a different width from the one it is finished at",
        );
        assert!(
            board.chars().all(|face| face == '0' || face == '1'),
            "board {done} has something on it that is not a lamp",
        );
        assert!(
            !crate::data::lights::remark(done).is_empty(),
            "the program has nothing to say about board {done}",
        );
    }
}

#[test]
fn the_facility_keeps_exactly_one_file_on_you() {
    let mine: Vec<&str> = DOSSIERS
        .iter()
        .filter(|dossier| dossier.mine)
        .map(|dossier| dossier.handle)
        .collect();
    assert_eq!(mine.len(), 1, "the facility keeps {mine:?} on you");
}

#[test]
fn every_program_the_machine_asks_for_by_name_exists() {
    for id in [
        "iam",
        "board",
        "notes",
        "sessions",
        "browser",
        "files",
        "lights",
        "office",
        "lights_on",
        "case",
        "store",
        "botterfly",
        "cross",
    ] {
        assert!(
            index_of(id).is_some(),
            "nothing on the desktop is called {id}",
        );
    }
}

#[test]
fn every_account_the_machine_asks_for_by_name_exists() {
    for id in ["guest", "r_mears", "d_okonkwo", "j_alderete"] {
        assert!(
            ACCOUNTS.iter().any(|account| account.id == id),
            "no account on this machine is called {id}",
        );
    }
}

#[test]
fn no_beat_offers_more_replies_than_the_composer_draws() {
    for definition in SCRIPT.contacts {
        let room = if definition.id == crate::data::script::HOST {
            crate::resources::choice_capacity(&SCRIPT) - 1
        } else {
            crate::resources::choice_capacity(&SCRIPT)
        };
        for node in definition.nodes {
            assert!(
                node.choices.len() <= room,
                "{}, beat {} offers {} replies and the composer draws {room}",
                definition.id,
                node.id,
                node.choices.len(),
            );
        }
    }
}

#[test]
fn no_room_offers_more_than_the_composer_draws() {
    for definition in SCRIPT.servers {
        for channel in definition.channels {
            let games = POSTED
                .iter()
                .filter(|posted| posted.room == channel.name)
                .count();
            let most = channel.questions.len() + games;
            assert!(
                most <= crate::resources::choice_capacity(&SCRIPT),
                "{}, {} can offer {most} things to say and the composer draws {}",
                definition.id,
                channel.name,
                crate::resources::choice_capacity(&SCRIPT),
            );
        }
    }
}

#[test]
fn the_calendar_reads_every_date_the_week_carries() {
    for night in SCRIPT.nights {
        let read = crate::systems::interface::calendar(night.day, night.date);
        assert!(
            read.contains('/'),
            "{} is dated {}, which the calendar cannot read",
            night.day,
            night.date,
        );
    }
}

#[test]
fn the_clock_reads_every_hour_of_the_day() {
    for minute in 0..24 * 60 {
        let seconds = (minute * 60) as f32;
        let short = crate::systems::interface::wall_clock(seconds);
        assert!(
            short.ends_with(" AM") || short.ends_with(" PM"),
            "{seconds} seconds past midnight reads {short}",
        );
        let hour: u32 = short
            .split(':')
            .next()
            .and_then(|part| part.parse().ok())
            .unwrap_or_default();
        assert!(
            (1..=12).contains(&hour),
            "{seconds} seconds past midnight reads hour {hour}",
        );
    }
    assert_eq!(crate::systems::interface::wall_clock(0.0), "12:00:00 AM");
    assert_eq!(
        crate::systems::interface::wall_clock(12.0 * 3600.0),
        "12:00:00 PM",
    );
    assert_eq!(
        crate::systems::interface::wall_clock(86_400.0 + 3_600.0),
        "1:00:00 AM",
    );
    assert_eq!(crate::systems::interface::wall_clock(-5.0), "12:00:00 AM");
}

#[test]
fn the_walk_and_the_buttons_number_the_ways_the_same() {
    let known: Vec<Fact> = Vec::new();
    for (here, place) in PLACES.iter().enumerate() {
        for target in PLACES {
            let Some(row) = crate::data::office::step_towards(here, target.id, &known) else {
                continue;
            };
            let open = crate::data::office::ways_here(place, &known);
            let way = open
                .get(row)
                .unwrap_or_else(|| panic!("{} has no way {row} open", place.id));
            let goes_to = place.ways[*way].goes_to;
            assert!(
                place_at(goes_to).is_some(),
                "the step from {} towards {} goes nowhere",
                place.id,
                target.id,
            );
            assert_ne!(
                goes_to, place.id,
                "the step from {} towards {} stays where it is",
                place.id, target.id,
            );
        }
    }
}

#[test]
fn every_place_can_be_walked_to_once_its_ways_are_open() {
    let known = establishable();
    for target in PLACES {
        if target.id == crate::data::office::START {
            continue;
        }
        let start = place_at(crate::data::office::START).expect("the walk to start somewhere");
        assert!(
            crate::data::office::step_towards(start, target.id, &known).is_some(),
            "nothing walks from {} to {}",
            crate::data::office::START,
            target.id,
        );
    }
}

#[test]
fn every_link_name_names_a_page() {
    for (url, name) in crate::data::web::LINK_NAMES {
        assert!(
            page_at(url).is_some(),
            "a link is called \"{name}\" and points at {url}, which is not a page",
        );
    }
}

#[test]
fn every_field_answer_can_be_typed() {
    for page in PAGES {
        let Some(prompt) = page.prompt.as_ref() else {
            continue;
        };
        assert!(
            !prompt.answers.is_empty(),
            "{} has a field with nothing it will take",
            page.url
        );
        for answer in prompt.answers {
            let flat: String = answer
                .typed
                .chars()
                .filter(|character| !character.is_whitespace())
                .flat_map(char::to_lowercase)
                .collect();
            assert_eq!(
                flat, answer.typed,
                "the field on {} waits for \"{}\", which it would never see",
                page.url, answer.typed,
            );
        }
        let takes_anything = prompt
            .answers
            .iter()
            .position(|answer| answer.typed.is_empty());
        if let Some(index) = takes_anything {
            assert_eq!(
                index,
                prompt.answers.len() - 1,
                "the field on {} takes anything before it has finished asking",
                page.url,
            );
        }
    }
}

#[test]
fn everything_the_demonstration_types_is_taken() {
    for (_, source) in crate::data::autoplay::KEPT {
        let crate::data::autoplay::Source::Answer(url, typed) = source else {
            continue;
        };
        let index = page_at(url)
            .unwrap_or_else(|| panic!("the demonstration goes to {url}, which is not a page"));
        let prompt = PAGES[index]
            .prompt
            .as_ref()
            .unwrap_or_else(|| panic!("{url} has no field to fill in"));
        assert!(
            crate::data::web::answered(prompt, typed).is_some(),
            "the demonstration types \"{typed}\" into {url} and it is not taken",
        );
    }
}

#[test]
fn the_week_can_be_finished() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    assert_eq!(
        reach.nights,
        SCRIPT.nights.len(),
        "the week stops on {}, which cannot be finished: {}",
        SCRIPT.nights[reach.nights.saturating_sub(1)].day,
        SCRIPT.nights[reach.nights.saturating_sub(1)]
            .condition
            .requires
            .iter()
            .filter(|fact| !reach.facts.contains(fact))
            .copied()
            .collect::<Vec<Fact>>()
            .join(", "),
    );
}

#[test]
fn every_night_can_be_ended() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    for night in SCRIPT.nights {
        for wanted in night.condition.requires {
            assert!(
                crate::data::reachable::reached(&reach, wanted),
                "{} waits on {wanted}, which nothing in the week reaches",
                night.day,
            );
        }
    }
}

#[test]
fn every_way_out_of_the_week_can_be_taken() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    for fact in [
        crate::data::endings::KNOWS_WHAT_IT_IS,
        crate::data::endings::KNOWS_THE_COURIER,
        crate::data::endings::FOUND_THE_HOOKUP,
        "was_given_botterfly",
        "can_go_out",
    ] {
        assert!(
            crate::data::reachable::reached(&reach, fact),
            "getting out waits on {fact}, which nothing in the week reaches",
        );
    }
}

#[test]
fn every_card_on_the_board_can_be_reached_in_play() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    let mut unreachable: Vec<Fact> = Vec::new();
    for lead in LEADS {
        for note in lead.notes {
            if !crate::data::reachable::reached(&reach, note.fact) {
                unreachable.push(note.fact);
            }
        }
    }
    assert!(
        unreachable.is_empty(),
        "these cards can never go up: {}",
        unreachable.join(", "),
    );
}

#[test]
fn nothing_in_the_script_is_written_and_never_read() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    let mut stranded: Vec<String> = Vec::new();
    for (index, definition) in SCRIPT.contacts.iter().enumerate() {
        for (slot, node) in definition.nodes.iter().enumerate() {
            if !reach.beats[index].contains(&slot) {
                stranded.push(format!("{}, beat {}", definition.id, node.id));
            }
        }
    }
    assert!(
        stranded.is_empty(),
        "nothing leads to these, so they can never be read: {}",
        stranded.join("; "),
    );
}

#[test]
fn every_note_on_the_desk_can_be_read_in_play() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    for note in crate::data::notes::NOTES {
        for fact in note.requires {
            assert!(
                crate::data::reachable::reached(&reach, fact),
                "the note by {} waits on {fact}, which nothing in the week reaches",
                note.author,
            );
        }
    }
}

#[test]
fn everything_on_the_network_can_be_offered() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    for listing in LISTINGS {
        for fact in listing.requires {
            assert!(
                crate::data::reachable::reached(&reach, fact),
                "{} waits on {fact}, which nothing in the week reaches",
                listing.title,
            );
        }
    }
    for mark in BOOKMARKS {
        for fact in mark.requires {
            assert!(
                crate::data::reachable::reached(&reach, fact),
                "the bookmark \"{}\" waits on {fact}, which nothing in the week reaches",
                mark.label,
            );
        }
    }
    for program in PROGRAMS {
        for fact in program.requires {
            assert!(
                crate::data::reachable::reached(&reach, fact),
                "{} waits on {fact}, which nothing in the week reaches",
                program.id,
            );
        }
    }
}

#[test]
fn every_file_on_the_disk_can_be_read_in_play() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    for file in LEFTOVERS {
        for fact in file.requires {
            assert!(
                crate::data::reachable::reached(&reach, fact),
                "{} waits on {fact}, which nothing in the week reaches",
                file.name,
            );
        }
    }
    for posted in POSTED {
        for fact in posted.requires {
            assert!(
                crate::data::reachable::reached(&reach, fact),
                "{} waits on {fact}, which nothing in the week reaches",
                posted.id,
            );
        }
    }
}

fn written_twice<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let mut seen: Vec<&str> = Vec::new();
    let mut twice: Vec<&str> = Vec::new();
    for name in names {
        if seen.contains(&name) {
            if !twice.contains(&name) {
                twice.push(name);
            }
        } else {
            seen.push(name);
        }
    }
    twice
}

#[test]
fn no_name_the_machine_looks_things_up_by_is_written_twice() {
    for (what, twice) in [
        (
            "contacts",
            written_twice(SCRIPT.contacts.iter().map(|entry| entry.id)),
        ),
        (
            "handles",
            written_twice(
                SCRIPT
                    .contacts
                    .iter()
                    .map(|entry| entry.handle)
                    .filter(|handle| !handle.is_empty()),
            ),
        ),
        (
            "servers",
            written_twice(SCRIPT.servers.iter().map(|entry| entry.id)),
        ),
        (
            "rooms",
            written_twice(
                SCRIPT
                    .servers
                    .iter()
                    .flat_map(|server| server.channels.iter())
                    .map(|room| room.name),
            ),
        ),
        (
            "nights",
            written_twice(SCRIPT.nights.iter().map(|night| night.day)),
        ),
        (
            "programs",
            written_twice(PROGRAMS.iter().map(|entry| entry.id)),
        ),
        (
            "accounts",
            written_twice(ACCOUNTS.iter().map(|entry| entry.id)),
        ),
        ("places", written_twice(PLACES.iter().map(|entry| entry.id))),
        ("things", written_twice(THINGS.iter().map(|entry| entry.id))),
        ("games", written_twice(POSTED.iter().map(|entry| entry.id))),
        ("pages", written_twice(PAGES.iter().map(|entry| entry.url))),
        ("leads", written_twice(LEADS.iter().map(|lead| lead.id))),
        (
            "cards",
            written_twice(
                LEADS
                    .iter()
                    .flat_map(|lead| lead.notes.iter())
                    .map(|note| note.fact),
            ),
        ),
        (
            "listings",
            written_twice(LISTINGS.iter().filter_map(|listing| listing.program)),
        ),
    ] {
        assert!(
            twice.is_empty(),
            "two of these {what} share a name: {twice:?}"
        );
    }
}

#[test]
fn every_post_in_a_room_lands_in_the_week() {
    let reach = crate::data::reachable::walk(&SCRIPT);
    for server in SCRIPT.servers {
        for channel in server.channels {
            for post in channel.posts {
                assert!(
                    post.night < SCRIPT.nights.len(),
                    "{}, {}: a post by {} is written for night {} and the week has {}",
                    server.id,
                    channel.name,
                    post.author,
                    post.night,
                    SCRIPT.nights.len(),
                );
                for fact in post.condition.requires {
                    assert!(
                        crate::data::reachable::reached(&reach, fact),
                        "{}, {}: a post by {} waits on {fact}, which nothing in the week reaches",
                        server.id,
                        channel.name,
                        post.author,
                    );
                }
                assert!(
                    post.delay_seconds >= 0.0,
                    "{}, {}: a post by {} lands before it is written",
                    server.id,
                    channel.name,
                    post.author,
                );
            }
        }
    }
}

#[test]
fn every_line_is_typed_before_it_lands() {
    for definition in SCRIPT.contacts {
        for node in definition.nodes {
            for line in node.lines {
                assert!(
                    !line.rhythm.is_empty(),
                    "{}, beat {}: a line lands with no rhythm at all",
                    definition.id,
                    node.id,
                );
                assert!(
                    line.rhythm.iter().all(|segment| *segment > 0.0),
                    "{}, beat {}: a line has a part of its rhythm that takes no time",
                    definition.id,
                    node.id,
                );
                assert!(
                    !line.text.is_empty() || crate::data::script::sent_program(line.text).is_some(),
                    "{}, beat {}: a line says nothing",
                    definition.id,
                    node.id,
                );
            }
        }
    }
}

#[test]
fn every_program_somebody_sends_is_a_program() {
    for definition in SCRIPT.contacts {
        for node in definition.nodes {
            for line in node.lines {
                let Some(name) = line.text.strip_prefix("store:") else {
                    continue;
                };
                assert!(
                    crate::data::script::sent_program(line.text).is_some(),
                    "{}, beat {} sends {}, which is not a program",
                    definition.id,
                    node.id,
                    name.trim(),
                );
                assert!(
                    crate::data::store::for_program(name.trim()).is_some(),
                    "{}, beat {} sends {}, which the software list cannot open",
                    definition.id,
                    node.id,
                    name.trim(),
                );
            }
        }
    }
}

const SOURCES: &[(&str, &str)] = &[
    ("autoplay.rs", include_str!("data/autoplay.rs")),
    ("board.rs", include_str!("data/board.rs")),
    ("boot.rs", include_str!("data/boot.rs")),
    ("case.rs", include_str!("data/case.rs")),
    ("config.rs", include_str!("data/config.rs")),
    ("cross.rs", include_str!("data/cross.rs")),
    ("data.rs", include_str!("data.rs")),
    ("endings.rs", include_str!("data/endings.rs")),
    ("files.rs", include_str!("data/files.rs")),
    ("games.rs", include_str!("data/games.rs")),
    ("intro.rs", include_str!("data/intro.rs")),
    ("lights.rs", include_str!("data/lights.rs")),
    ("machine.rs", include_str!("data/machine.rs")),
    ("main.rs", include_str!("main.rs")),
    ("notes.rs", include_str!("data/notes.rs")),
    ("office.rs", include_str!("data/office.rs")),
    ("phosphor.rs", include_str!("data/phosphor.rs")),
    ("pipes.rs", include_str!("data/pipes.rs")),
    ("plugin.rs", include_str!("plugin.rs")),
    ("programs.rs", include_str!("data/programs.rs")),
    ("resources.rs", include_str!("resources.rs")),
    ("screen.rs", include_str!("data/screen.rs")),
    ("script.rs", include_str!("data/script.rs")),
    ("store.rs", include_str!("data/store.rs")),
    ("tonight.rs", include_str!("data/tonight.rs")),
    ("web.rs", include_str!("data/web.rs")),
    (
        "components/desktop.rs",
        include_str!("components/desktop.rs"),
    ),
    (
        "components/messenger.rs",
        include_str!("components/messenger.rs"),
    ),
    ("components/panels.rs", include_str!("components/panels.rs")),
    ("components/slots.rs", include_str!("components/slots.rs")),
    ("systems/ambience.rs", include_str!("systems/ambience.rs")),
    ("systems/autoplay.rs", include_str!("systems/autoplay.rs")),
    ("systems/browser.rs", include_str!("systems/browser.rs")),
    (
        "systems/conversation.rs",
        include_str!("systems/conversation.rs"),
    ),
    ("systems/desk.rs", include_str!("systems/desk.rs")),
    ("systems/feeds.rs", include_str!("systems/feeds.rs")),
    ("systems/input.rs", include_str!("systems/input.rs")),
    ("systems/interface.rs", include_str!("systems/interface.rs")),
    ("systems/intro.rs", include_str!("systems/intro.rs")),
    (
        "systems/screensaver.rs",
        include_str!("systems/screensaver.rs"),
    ),
    ("systems/setup.rs", include_str!("systems/setup.rs")),
];

#[test]
fn nothing_carries_its_own_behaviour() {
    for (name, text) in SOURCES {
        for (number, line) in text.lines().enumerate() {
            let line = line.trim_end();
            if let Some(rest) = line.strip_prefix("impl ") {
                let allowed = rest.starts_with("Default for")
                    || rest.starts_with("Plugin for")
                    || rest.starts_with("PassNode<");
                assert!(
                    allowed,
                    "{name}:{}: {line}\nbehaviour goes in a system, not on the type",
                    number + 1,
                );
            }
            assert!(
                !line.starts_with("trait ") && !line.starts_with("pub trait "),
                "{name}:{}: {line}\nnothing here needs a trait of its own",
                number + 1,
            );
        }
    }
}

#[test]
fn every_system_says_what_it_reads() {
    for (name, text) in SOURCES {
        for (number, line) in text.lines().enumerate() {
            if line.contains("app.world.") {
                continue;
            }
            let reaches = line.contains("world.res::<")
                || line.contains("world.res_mut::<")
                || line.contains("world.plugin_resource")
                || line.contains(".res::<")
                    && !line.contains("resources.")
                    && !line.trim_start().starts_with("//");
            assert!(
                !reaches,
                "{name}:{}: {}\nask for it as Res or ResMut instead",
                number + 1,
                line.trim(),
            );
        }
    }
}

#[test]
fn every_file_the_game_is_written_in_is_checked() {
    let listed: Vec<&str> = SOURCES.iter().map(|(name, _)| *name).collect();
    assert!(
        listed.contains(&"main.rs"),
        "main.rs is not on the list the architecture checks read",
    );
    let modules = include_str!("main.rs");
    for line in modules.lines() {
        let Some(name) = line
            .strip_prefix("mod ")
            .and_then(|rest| rest.strip_suffix(';'))
        else {
            continue;
        };
        if matches!(name, "components" | "reachable" | "systems" | "tests") {
            continue;
        }
        let file = format!("{name}.rs");
        assert!(
            listed.contains(&file.as_str()),
            "{file} is not on the list the architecture checks read",
        );
    }
    for line in include_str!("systems.rs").lines() {
        let Some(name) = line
            .strip_prefix("pub mod ")
            .and_then(|rest| rest.strip_suffix(';'))
        else {
            continue;
        };
        let file = format!("systems/{name}.rs");
        assert!(
            listed.contains(&file.as_str()),
            "{file} is not on the list the architecture checks read",
        );
    }
}

#[test]
fn every_pane_that_is_built_is_shown() {
    let built = SOURCES
        .iter()
        .find(|(name, _)| *name == "systems/setup.rs")
        .expect("setup.rs on the list")
        .1;
    let everywhere: String = SOURCES.iter().map(|(_, text)| *text).collect();
    for line in built.lines() {
        let Some(pane) = line
            .trim()
            .strip_prefix("resources.interface.")
            .and_then(|rest| rest.strip_suffix(" = root;"))
        else {
            continue;
        };
        assert!(
            everywhere.contains(&format!("resources.interface.{pane},")),
            "the {pane} pane is built and never shown, so the program opens on nothing",
        );
    }
}
