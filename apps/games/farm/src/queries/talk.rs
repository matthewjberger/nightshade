use crate::data::{Errand, Person};
use crate::queries::pocket::{can_deliver, can_temper, errand_progress, is_done, level_of};
use crate::resources::{AfterTalk, Dialogue, Inventory, Quests};

pub fn current(dialogue: &Dialogue) -> &str {
    dialogue.lines.get(dialogue.line).map_or("", String::as_str)
}

pub fn shown(dialogue: &Dialogue) -> String {
    let line = current(dialogue);
    let count = (dialogue.revealed as usize).min(line.chars().count());
    line.chars().take(count).collect()
}

pub fn fully_shown(dialogue: &Dialogue) -> bool {
    dialogue.revealed as usize >= current(dialogue).chars().count()
}

pub fn is_last(dialogue: &Dialogue) -> bool {
    dialogue.line + 1 >= dialogue.lines.len()
}

pub struct Mood {
    pub hour: f32,
    pub raining: bool,
    pub day: u32,
    pub forecast: String,
    pub has_weapon: bool,
}

fn errand_lines(
    errand: Errand,
    inventory: &Inventory,
    quests: &Quests,
    ask: &str,
    thanks: &str,
    after: &str,
) -> (Vec<String>, AfterTalk) {
    if is_done(quests, errand) {
        return (vec![after.to_string()], AfterTalk::Nothing);
    }
    let (item, wanted) = errand.wants();
    if can_deliver(inventory, quests, errand) {
        return (
            vec![
                thanks.to_string(),
                format!("Here, {} coins for your trouble.", errand.reward()),
            ],
            AfterTalk::Deliver(errand),
        );
    }
    let (have, _) = errand_progress(inventory, errand);
    let plural = if wanted == 1 { "" } else { "s" };
    (
        vec![
            ask.to_string(),
            format!(
                "Bring me {} {}{} and I'll pay {} coins. You have {}.",
                wanted,
                item.name().to_lowercase(),
                plural,
                errand.reward(),
                have
            ),
        ],
        AfterTalk::Nothing,
    )
}

pub fn lines_for(
    person: Person,
    mood: &Mood,
    inventory: &Inventory,
    quests: &Quests,
) -> (Vec<String>, AfterTalk) {
    let morning = mood.hour < 11.0;
    let evening = mood.hour >= 17.0;
    match person {
        Person::Bram => {
            let greeting = if mood.raining {
                "Rain's good for the fields and bad for my roof."
            } else if morning {
                "Morning! Fresh seeds, fair prices."
            } else if evening {
                "Evening. I'll take whatever you dug up today."
            } else {
                "Afternoon. The tomatoes are selling well this week."
            };
            (
                vec![
                    greeting.to_string(),
                    "Seeds to buy, and I'll pay coin for crops, eggs, milk, wood, stone, fish."
                        .to_string(),
                ],
                AfterTalk::OpenShop(crate::data::ShopKind::Barn),
            )
        }
        Person::Wren => {
            if !mood.has_weapon {
                return (
                    vec![
                        "Going down the Hollow with bare hands? Not from my forge.".to_string(),
                        "The mouth is north of the square, cut into the cliff. Follow the lanterns."
                            .to_string(),
                        "Take this old dagger. Bring me stone, and I'll make you something better."
                            .to_string(),
                    ],
                    AfterTalk::GiveDagger,
                );
            }
            if let Some(tool) = can_temper(inventory) {
                return (
                    vec![
                        format!(
                            "That {} could take another edge. I have the ore heat for it.",
                            tool.name().to_lowercase()
                        ),
                        format!(
                            "{} ore and {} coins, and it comes back at +{}.",
                            crate::theme::TEMPER_ORE,
                            crate::theme::TEMPER_COINS,
                            level_of(inventory, tool) + 1
                        ),
                    ],
                    AfterTalk::Temper(tool),
                );
            }
            let stone = errand_lines(
                Errand::Stone,
                inventory,
                quests,
                "The forge eats stone faster than I can dig it.",
                "Good stone! That'll line the furnace for a season.",
                "The furnace has never run hotter. Bread's on the bench if you're hungry.",
            );
            if can_deliver(inventory, quests, Errand::Stone) {
                return stone;
            }
            let (mut lines, _) = stone;
            if !is_done(quests, Errand::Stone) {
                lines.push(
                    "Meanwhile, I buy stone and wood, and I keep bread on the bench.".to_string(),
                );
            }
            (lines, AfterTalk::OpenShop(crate::data::ShopKind::Forge))
        }
        Person::Lina => {
            let wheat = errand_lines(
                Errand::Wheat,
                inventory,
                quests,
                "The oven is warm and the flour bin is empty.",
                "Wheat! The whole village will smell the bread by noon.",
                "Bread's fresh. I pay well for wheat, eggs, and milk.",
            );
            if can_deliver(inventory, quests, Errand::Wheat) {
                return wheat;
            }
            let (mut lines, _) = wheat;
            if mood.raining {
                lines.push("Rain outside, bread inside. That's the right way round.".to_string());
            } else if !is_done(quests, Errand::Wheat) {
                lines.push("Fresh bread on the counter, all the same.".to_string());
            }
            (lines, AfterTalk::OpenShop(crate::data::ShopKind::Bakery))
        }
        Person::Mira => errand_lines(
            Errand::Carrots,
            inventory,
            quests,
            "I've been craving carrots for days.",
            "Carrots! You're a marvel. Tonight, watch the sky over the pond.",
            "The sky is yours every night now, from half past eight. Sleep is overrated.",
        ),
        Person::Tam => errand_lines(
            Errand::Fish,
            inventory,
            quests,
            "The river's full and my back is done. You've got the rod for it.",
            "Three fish! You've got the patience for it.",
            "Cast where the water's still. That's all the secret there is.",
        ),
        Person::Nell => errand_lines(
            Errand::Wood,
            inventory,
            quests,
            "I'm building a boat, and a boat wants wood.",
            "That's a hull's worth. She'll float by autumn.",
            "The boat's coming along. Every plank of it was yours.",
        ),
        Person::Pip => {
            if is_done(quests, Errand::Egg) {
                (
                    vec![if mood.raining {
                        "Puddles! Come on, jump in one!".to_string()
                    } else {
                        "Race you to the well! Ready, go!".to_string()
                    }],
                    AfterTalk::Nothing,
                )
            } else {
                errand_lines(
                    Errand::Egg,
                    inventory,
                    quests,
                    "Mum says I can have an egg for breakfast if I find one.",
                    "An egg! I'll tell Mum you found it. Here's my coins.",
                    "",
                )
            }
        }
        Person::Osk => {
            let weather = if mood.raining {
                "My knee said rain, and my knee was right.".to_string()
            } else {
                format!("My knee says: {}", mood.forecast)
            };
            (
                vec![
                    weather,
                    format!(
                        "Day {}. The well's been here longer than any of us.",
                        mood.day + 1
                    ),
                ],
                AfterTalk::Nothing,
            )
        }
        Person::Dov => (
            vec![if morning {
                "The cows give best in the morning. Bring a bucket.".to_string()
            } else if evening {
                "Sheep in, gate shut. That's a day.".to_string()
            } else {
                "The hens lay wherever they please. Keep an eye on the grass.".to_string()
            }],
            AfterTalk::Nothing,
        ),
        Person::Juno => {
            if mood.hour >= 21.0 || mood.hour < 5.5 {
                (
                    vec![
                        "The Kettle keeps a room for me. Sit a while.".to_string(),
                        "I walked from the coast. Your village is the kindest yet.".to_string(),
                    ],
                    AfterTalk::Nothing,
                )
            } else {
                errand_lines(
                    Errand::Bread,
                    inventory,
                    quests,
                    "A traveler lives on bread, and I'm out of it.",
                    "Bread! Lina's, by the smell. I'll be on the road again by spring.",
                    "I'll remember this village. Farewell, friend, until the road turns back.",
                )
            }
        }
    }
}
