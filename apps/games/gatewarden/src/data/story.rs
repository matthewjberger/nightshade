use crate::data::cards::CardId;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Shard {
    #[default]
    Ember,
    Tide,
    Grove,
    Storm,
    Cinder,
    Deep,
    Elder,
}

impl Shard {
    pub fn name(self) -> &'static str {
        match self {
            Shard::Ember => "Shard of Ember",
            Shard::Tide => "Shard of Tide",
            Shard::Grove => "Shard of Grove",
            Shard::Storm => "Shard of Storm",
            Shard::Cinder => "Shard of Cinder",
            Shard::Deep => "Shard of the Deep",
            Shard::Elder => "Shard of the Elder",
        }
    }
}

impl Shard {
    pub fn text(self) -> &'static str {
        match self {
            Shard::Ember => "Ember minions +1 attack",
            Shard::Tide => "heals 2 each turn",
            Shard::Grove => "Guard minions +1 health",
            Shard::Storm => "spells +1 damage",
            Shard::Cinder => "Rush minions +1 attack",
            Shard::Deep => "+5 hero health",
            Shard::Elder => "one more opening card",
        }
    }
}

pub fn warden_shard(gate: u32) -> Option<Shard> {
    shard_for(gate).or(Some(Shard::Storm))
}

pub fn shard_for(gate: u32) -> Option<Shard> {
    match gate {
        1 => Some(Shard::Ember),
        2 => Some(Shard::Tide),
        3 => Some(Shard::Grove),
        4 => Some(Shard::Storm),
        5 => Some(Shard::Cinder),
        6 => Some(Shard::Deep),
        7 => Some(Shard::Elder),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ForkOutcome {
    Duel,
    Tithe { gold: i64 },
    Borrow,
    Spare,
    Thin,
}

#[derive(Clone, Copy)]
pub struct ForkOption {
    pub label: &'static str,
    pub detail: &'static str,
    pub outcome: ForkOutcome,
}

#[derive(Clone, Copy)]
pub struct Fork {
    pub prompt: &'static str,
    pub first: ForkOption,
    pub second: ForkOption,
}

#[derive(Clone, Copy)]
pub struct SceneText {
    pub speaker: &'static str,
    pub lines: [&'static str; 4],
    pub fork: Option<Fork>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Ending {
    #[default]
    Summit,
    OpenGates,
    Whole,
}

pub const TITHE_GOLD: i64 = 8;
pub const BORROW_WARDEN_HEALTH: i32 = 5;
pub const THIN_WARDEN_HEALTH: i32 = 6;
pub const THIN_COUNT: usize = 3;
pub const SPARE_CARDS: [CardId; 3] = [CardId::Wavesinger, CardId::DeepWard, CardId::TideSurge];
pub const MERCY_FOR_OPEN_GATES: u32 = 2;
const DUEL: ForkOption = ForkOption {
    label: "Draw steel",
    detail: "Fight the warden as it stands.",
    outcome: ForkOutcome::Duel,
};

pub fn prologue() -> SceneText {
    SceneText {
        speaker: "The climb",
        lines: [
            "You were the warden of the summit once. Then the eight gates were sealed,",
            "and each gate keeper took a shard of what you were and settled in to hold it.",
            "You woke at the foot of the mountain with a thin deck and a thinner memory.",
            "Every gate you pass returns a shard. Every shard returns a piece of you.",
        ],
        fork: None,
    }
}

pub fn arrival(gate: u32) -> SceneText {
    match gate {
        1 => SceneText {
            speaker: "Ash Whelp",
            lines: [
                "A small dragon, all teeth and smoke, blocks the first gate.",
                "\"You again? They said you would come crawling back. I hoped it was true.\"",
                "\"I hold the ember you left behind. Come and take it, if you still burn.\"",
                "",
            ],
            fork: None,
        },
        2 => SceneText {
            speaker: "Reef Sentinel",
            lines: [
                "Water runs over the stones of the second gate. A tall figure waits in it.",
                "\"I kept your tide for you. I did not ask to. None of us asked.\"",
                "\"The others will fight you. I would rather be paid and be left alone.\"",
                "",
            ],
            fork: Some(Fork {
                prompt: "The sentinel names a price.",
                first: ForkOption {
                    label: "Pay the tithe",
                    detail: "Give 8 gold. The gate opens, the shard is yours, and no duel is fought. No reward gold.",
                    outcome: ForkOutcome::Tithe { gold: TITHE_GOLD },
                },
                second: DUEL,
            }),
        },
        3 => SceneText {
            speaker: "Bramble Knight",
            lines: [
                "The third gate is grown shut with thorn. A knight in bark armor stands before it.",
                "\"The grove remembers you. It remembers you left.\"",
                "\"Roots hold what you dropped. Roots do not give it back for asking.\"",
                "",
            ],
            fork: None,
        },
        4 => SceneText {
            speaker: "Skyvoice",
            lines: [
                "Wind screams through the fourth gate. A hooded figure leans on the gale.",
                "\"Your storm sits in my hand now. I could lend it to you for one fight.\"",
                "\"A loan has interest. I would fight you at my full strength.\"",
                "",
            ],
            fork: Some(Fork {
                prompt: "The caller offers a loan.",
                first: ForkOption {
                    label: "Borrow the Bolt",
                    detail: "Your hero power becomes Bolt for this duel, needs no target, and the warden gains 5 health.",
                    outcome: ForkOutcome::Borrow,
                },
                second: DUEL,
            }),
        },
        5 => SceneText {
            speaker: "Cinder Baron",
            lines: [
                "The fifth gate glows like a forge. A baron of slag and coal bars the way.",
                "\"The whelp was a spark. I am the furnace it was lit from.\"",
                "\"Your cinder is mine by right of holding. Rights are settled by fire.\"",
                "",
            ],
            fork: None,
        },
        6 => SceneText {
            speaker: "Tide Matriarch",
            lines: [
                "The sixth gate lies under still black water. An old queen rises from it.",
                "\"I am tired, climber. I have held the deep for longer than you were gone.\"",
                "\"Spare me the duel and I will give you what the tide taught me instead.\"",
                "",
            ],
            fork: Some(Fork {
                prompt: "The matriarch asks for mercy.",
                first: ForkOption {
                    label: "Spare her",
                    detail: "No duel and no shard. Three Tide cards join your deck and the gate opens.",
                    outcome: ForkOutcome::Spare,
                },
                second: DUEL,
            }),
        },
        7 => SceneText {
            speaker: "Elder of the Grove",
            lines: [
                "The seventh gate is a single tree older than the mountain. It speaks slowly.",
                "\"You carry too much. Every card you never play is a weight on your climb.\"",
                "\"Let me prune you. I will grow stronger for it, but so will you.\"",
                "",
            ],
            fork: Some(Fork {
                prompt: "The elder offers to prune your deck.",
                first: ForkOption {
                    label: "Be pruned",
                    detail: "Your three cheapest cards leave the deck for good. The elder gains 6 health.",
                    outcome: ForkOutcome::Thin,
                },
                second: DUEL,
            }),
        },
        _ => SceneText {
            speaker: "Storm Dragon",
            lines: [
                "The last gate is the sky itself. A dragon made of lightning coils around it.",
                "\"I know what you are. I was there when they cut you into eight.\"",
                "\"Beat me and the summit is yours again. Or leave the gates open behind you.\"",
                "\"Either way, climber, you will not be what you were.\"",
            ],
            fork: None,
        },
    }
}

pub fn victory(gate: u32, spared: bool) -> SceneText {
    match (gate, spared) {
        (1, _) => SceneText {
            speaker: "Shard of Ember",
            lines: [
                "The whelp curls up in the ash, more sulking than hurt.",
                "A warm shard settles into your chest. You remember a hearth and a name.",
                "Your Ember minions strike harder now.",
                "",
            ],
            fork: None,
        },
        (2, true) => SceneText {
            speaker: "Reef Sentinel",
            lines: [
                "The sentinel counts the gold twice and steps aside without a word.",
                "The shard is cold in your hand, given rather than won. It settles all the same.",
                "Your hero mends a little at the start of every turn.",
                "",
            ],
            fork: None,
        },
        (2, false) => SceneText {
            speaker: "Shard of Tide",
            lines: [
                "The sentinel sinks to a knee in the running water. \"Take it. It was never mine.\"",
                "The shard tastes of salt. You remember standing on a shore you owned.",
                "Your hero mends a little at the start of every turn.",
                "",
            ],
            fork: None,
        },
        (3, _) => SceneText {
            speaker: "Shard of Grove",
            lines: [
                "The thorns withdraw. The knight bows, bark creaking.",
                "A green shard takes root in you. You remember planting the first tree here.",
                "Your Guard minions stand a little taller.",
                "",
            ],
            fork: None,
        },
        (4, _) => SceneText {
            speaker: "Shard of Storm",
            lines: [
                "The wind drops all at once. The caller laughs and hands over the loan and the debt.",
                "A crackling shard settles behind your eyes. You remember the weather obeying.",
                "Your damage spells hit harder.",
                "",
            ],
            fork: None,
        },
        (5, _) => SceneText {
            speaker: "Shard of Cinder",
            lines: [
                "The baron cracks and cools into a statue of himself.",
                "A hot shard finds the place the ember left. You remember running into fire and out.",
                "Your Rush minions hit harder.",
                "",
            ],
            fork: None,
        },
        (6, true) => SceneText {
            speaker: "Tide Matriarch",
            lines: [
                "The old queen sinks back into the black water with something like a smile.",
                "Three cards of the deep drift into your hands. The shard stays with her.",
                "\"Mercy is a strange thing to carry up a mountain. Carry it anyway.\"",
                "",
            ],
            fork: None,
        },
        (6, false) => SceneText {
            speaker: "Shard of the Deep",
            lines: [
                "The matriarch lets the water take her. She does not look unhappy.",
                "A heavy shard settles low in you. You remember how much you could hold.",
                "Your hero has more health.",
                "",
            ],
            fork: None,
        },
        (7, _) => SceneText {
            speaker: "Shard of the Elder",
            lines: [
                "The old tree sheds its leaves all at once, and that is how you know it is done.",
                "A quiet shard settles where the others are. You remember being patient.",
                "You start each battle with an extra card in hand.",
                "",
            ],
            fork: None,
        },
        _ => SceneText {
            speaker: "The summit",
            lines: [
                "The dragon comes apart into weather and is gone.",
                "The last gate stands open. The mountain is quiet for the first time in years.",
                "",
                "",
            ],
            fork: None,
        },
    }
}

pub fn ending(kind: Ending) -> SceneText {
    match kind {
        Ending::Whole => SceneText {
            speaker: "Whole",
            lines: [
                "Seven shards, and the eighth is the mountain itself. They fit together.",
                "You stand at the summit as the warden you were, and you remember all of it.",
                "The gates below seal one by one, waiting for whoever comes next.",
                "Perhaps you will be kinder to them than the keepers were to you.",
            ],
            fork: None,
        },
        Ending::OpenGates => SceneText {
            speaker: "Open gates",
            lines: [
                "You reached the summit with mercy in your pockets instead of shards.",
                "You leave the gates open behind you. The keepers you spared go home.",
                "The mountain will not be yours. It will not be anyone's. That is enough.",
                "",
            ],
            fork: None,
        },
        Ending::Summit => SceneText {
            speaker: "The summit",
            lines: [
                "You take the summit back with most of yourself and all of your deck.",
                "Some shards stayed with their keepers. Some memories will not come back.",
                "The wind is yours again. It is thinner than you remember, and it is enough.",
                "",
            ],
            fork: None,
        },
    }
}

pub fn yard() -> SceneText {
    SceneText {
        speaker: "The Echo",
        lines: [
            "Below the first gate there is a yard, and in the yard there is a scarecrow.",
            "A voice you know says: sit. It is your own voice, older, and it has been waiting.",
            "\"You were the best of us once. It comes back faster than you think.\"",
            "\"Play what I tell you to play. After that, play what you like.\"",
        ],
        fork: None,
    }
}

pub fn yard_end() -> SceneText {
    SceneText {
        speaker: "The Echo",
        lines: [
            "The straw comes apart in the wind. Nobody applauds. The mountain is not that kind of place.",
            "\"That is the whole game: crystals, lines, and a sum. Everything else is nerve.\"",
            "\"The wardens will not tell you what they mean to do. I will not be there to tell you either.\"",
            "\"Go anyway.\"",
        ],
        fork: None,
    }
}

pub fn defeat_line(gate: u32) -> &'static str {
    match gate {
        1 => "\"Crawl back down. Try again when you burn.\"",
        2 => "\"You should have paid. Everyone should have paid.\"",
        3 => "\"The roots keep what falls. They kept you.\"",
        4 => "\"The weather does not owe you anything anymore.\"",
        5 => "\"Ash is what a climber becomes. Welcome to it.\"",
        6 => "\"Rest, climber. The deep is patient and so am I.\"",
        7 => "\"You carried too much. I told you.\"",
        _ => "\"Not what you were. Not yet what you will be. Climb again.\"",
    }
}
