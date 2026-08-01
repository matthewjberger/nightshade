use crate::data::script::{Effect, Fact, Script, condition_holds};

pub struct Reach {
    pub facts: Vec<Fact>,
    pub nights: usize,
    pub beats: Vec<Vec<usize>>,
}

pub fn reached(reach: &Reach, fact: Fact) -> bool {
    reach.facts.contains(&fact)
}

pub fn walk(script: &'static Script) -> Reach {
    let mut facts: Vec<Fact> = Vec::from(crate::data::script::SYSTEM_FACTS);
    for page in crate::data::web::PAGES {
        for fact in page.asserts {
            learn(&mut facts, fact);
        }
        if let Some(prompt) = page.prompt.as_ref() {
            for answer in prompt.answers {
                for fact in answer.asserts {
                    learn(&mut facts, fact);
                }
            }
        }
    }
    let mut nights = 0usize;
    let mut servers: Vec<usize> = script
        .servers
        .iter()
        .enumerate()
        .filter(|(_, server)| server.listed)
        .map(|(index, _)| index)
        .collect();
    let mut beats: Vec<Vec<usize>> = vec![Vec::new(); script.contacts.len()];
    for (index, definition) in script.contacts.iter().enumerate() {
        if definition.listed {
            open_beat(script, index, definition.entry, &facts, &mut beats);
        }
    }

    loop {
        let before = (facts.len(), nights, servers.len(), depth(&beats));

        if nights == 0 {
            nights = 1;
            if let Some(night) = script.nights.first() {
                learn(&mut facts, night.day);
                fire(script, night.opening, &mut facts, &mut servers, &mut beats);
            }
        }
        while nights < script.nights.len() {
            let last = &script.nights[nights - 1];
            if !condition_holds(&last.condition, &facts) {
                break;
            }
            let night = &script.nights[nights];
            nights += 1;
            learn(&mut facts, night.day);
            fire(script, night.opening, &mut facts, &mut servers, &mut beats);
        }

        for index in 0..script.contacts.len() {
            let definition = &script.contacts[index];
            for slot in beats[index].clone() {
                let node = &definition.nodes[slot];
                fire(script, node.effects, &mut facts, &mut servers, &mut beats);
                for choice in node.choices {
                    if !condition_holds(&choice.condition, &facts) {
                        continue;
                    }
                    let program = definition.bot
                        && !crate::data::script::stopped_being_a_program(
                            script,
                            definition.id,
                            nights.saturating_sub(1),
                        );
                    if program && choice.tactic != crate::data::script::Tactic::Plain {
                        continue;
                    }
                    for fact in choice.asserts {
                        learn(&mut facts, fact);
                    }
                    open_beat(script, index, choice.goto, &facts, &mut beats);
                }
            }
        }

        for server in servers.clone() {
            for channel in script.servers[server].channels {
                fire(
                    script,
                    channel.on_open,
                    &mut facts,
                    &mut servers,
                    &mut beats,
                );
                for question in channel.questions {
                    if !condition_holds(&question.condition, &facts) {
                        continue;
                    }
                    for fact in question.asserts {
                        learn(&mut facts, fact);
                    }
                    fire(
                        script,
                        question.effects,
                        &mut facts,
                        &mut servers,
                        &mut beats,
                    );
                }
            }
        }

        for place in crate::data::office::PLACES {
            if !walked_to(place.id, &facts) {
                continue;
            }
            for fact in place.asserts {
                learn(&mut facts, fact);
            }
            for id in place.things {
                let Some(thing) = crate::data::office::thing_at(id) else {
                    continue;
                };
                let thing = &crate::data::office::THINGS[thing];
                if !thing.requires.iter().all(|fact| facts.contains(fact)) {
                    continue;
                }
                for fact in thing.asserts {
                    learn(&mut facts, fact);
                }
            }
        }

        for file in crate::data::files::LEFTOVERS {
            if file.requires.iter().all(|fact| facts.contains(fact)) {
                for fact in file.asserts {
                    learn(&mut facts, fact);
                }
            }
        }
        for posted in crate::data::games::POSTED {
            if posted.requires.iter().all(|fact| facts.contains(fact)) {
                for fact in posted.asserts {
                    learn(&mut facts, fact);
                }
            }
        }
        for pairing in crate::data::cross::PAIRINGS {
            if pairing.requires.iter().all(|fact| facts.contains(fact)) {
                for fact in pairing.asserts {
                    learn(&mut facts, fact);
                }
            }
        }
        for dossier in crate::data::case::DOSSIERS {
            if facts.contains(&crate::data::lights::TURNED_ON) {
                for fact in dossier.asserts {
                    learn(&mut facts, fact);
                }
            }
        }

        if before == (facts.len(), nights, servers.len(), depth(&beats)) {
            break;
        }
    }

    facts.sort_unstable();
    facts.dedup();
    Reach {
        facts,
        nights,
        beats,
    }
}

fn depth(beats: &[Vec<usize>]) -> usize {
    beats.iter().map(|open| open.len()).sum()
}

fn learn(facts: &mut Vec<Fact>, fact: Fact) {
    if !facts.contains(&fact) {
        facts.push(fact);
    }
}

fn open_beat(
    script: &'static Script,
    contact: usize,
    id: &str,
    facts: &[Fact],
    beats: &mut [Vec<usize>],
) {
    if id.is_empty() {
        return;
    }
    for (slot, node) in script.contacts[contact].nodes.iter().enumerate() {
        if node.id == id
            && condition_holds(&node.condition, facts)
            && !beats[contact].contains(&slot)
        {
            beats[contact].push(slot);
        }
    }
}

fn fire(
    script: &'static Script,
    effects: &'static [Effect],
    facts: &mut Vec<Fact>,
    servers: &mut Vec<usize>,
    beats: &mut [Vec<usize>],
) {
    for effect in effects {
        match effect {
            Effect::Assert(fact) => learn(facts, fact),
            Effect::Reveal(id) | Effect::Invite(id, _) => {
                if let Some(index) = crate::data::script::contact_index(script, id) {
                    let entry = script.contacts[index].entry;
                    open_beat(script, index, entry, facts, beats);
                }
            }
            Effect::Message(id, node) => {
                if let Some(index) = crate::data::script::contact_index(script, id) {
                    open_beat(script, index, node, facts, beats);
                }
            }
            Effect::RevealServer(id) => {
                if let Some(index) = crate::data::script::server_index(script, id)
                    && !servers.contains(&index)
                {
                    servers.push(index);
                }
            }
            _ => {}
        }
    }
}

fn walked_to(target: &str, facts: &[Fact]) -> bool {
    let Some(goal) = crate::data::office::place_at(target) else {
        return false;
    };
    let Some(start) = crate::data::office::place_at(crate::data::office::START) else {
        return false;
    };
    if goal == start {
        return true;
    }
    let mut seen = vec![false; crate::data::office::PLACES.len()];
    let mut walking = vec![start];
    seen[start] = true;
    while let Some(at) = walking.pop() {
        for way in crate::data::office::ways_here(&crate::data::office::PLACES[at], facts) {
            let Some(next) =
                crate::data::office::place_at(crate::data::office::PLACES[at].ways[way].goes_to)
            else {
                continue;
            };
            if next == goal {
                return true;
            }
            if !seen[next] {
                seen[next] = true;
                walking.push(next);
            }
        }
    }
    false
}
