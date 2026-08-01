use crate::data::cards::CardId;

pub const ECHO: &str = "The Echo";
pub const STRAW_WARDEN: &str = "Straw Warden";
pub const STRAW_TITLE: &str = "A practice target";
pub const STRAW_HEALTH: i32 = 12;
pub const PLAYER_HAND: [CardId; 4] = [
    CardId::Spearhand,
    CardId::Shieldbearer,
    CardId::EmberBolt,
    CardId::Knight,
];

pub const PLAYER_DECK: [CardId; 5] = [
    CardId::Wildfire,
    CardId::DragonBreath,
    CardId::Wavesinger,
    CardId::Growth,
    CardId::Lightning,
];

pub const STRAW_HAND: [CardId; 4] = [
    CardId::EmberMote,
    CardId::Wavesinger,
    CardId::Mercenary,
    CardId::Knight,
];

pub const STRAW_DECK: [CardId; 6] = [
    CardId::Giant,
    CardId::BlazeDrake,
    CardId::ElderBough,
    CardId::Kraken,
    CardId::Giant,
    CardId::Champion,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Aim {
    Warden,
    Minion(CardId),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    None,
    Hand(CardId),
    Mine(CardId),
    Foe(CardId),
    Warden,
    You,
    HandSurface,
    EndTurn,
    Power,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gate {
    Next,
    Play(CardId),
    Attack(CardId, Aim),
    Cast(CardId, Aim),
    Power(Aim),
    EndTurn,
    EnemyTurn,
    Win,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Deed {
    Played(CardId),
    Attacked(CardId, Aim),
    Cast(CardId, Aim),
    Powered(Aim),
    Ended,
}

pub struct Lesson {
    pub text: &'static str,
    pub focus: Focus,
    pub gate: Gate,
}

pub const WATCH: &str = "Watch. The straw does not think, but it does move.";
pub const LESSONS: &[Lesson] = &[
    Lesson {
        text: "Easy. It is only straw. I am the part of you that stayed behind when the gates took the rest, and I remember how the table works. By the end of this, so will you.",
        focus: Focus::None,
        gate: Gate::Next,
    },
    Lesson {
        text: "The Straw Warden. The gate keepers stuff it and stand it up to practice on. Twelve health. Bring that to zero and the lesson is over. Nothing you do in this yard follows you up the mountain.",
        focus: Focus::Warden,
        gate: Gate::Next,
    },
    Lesson {
        text: "That is you. Thirty health and one mana crystal. Each turn you gain a crystal, up to ten, and they all refill. Mana is the only thing you spend. Cards are what you spend it on.",
        focus: Focus::You,
        gate: Gate::Next,
    },
    Lesson {
        text: "Your hand. Every card wears its price in the blue gem. Minions carry a gold badge for attack and a red one for health. Rest your cursor on a card and the panel below me explains it.",
        focus: Focus::HandSurface,
        gate: Gate::Next,
    },
    Lesson {
        text: "Start small. The Spearhand costs one, and one is what you have. Click him, or press 1. He walks onto your line.",
        focus: Focus::Hand(CardId::Spearhand),
        gate: Gate::Play(CardId::Spearhand),
    },
    Lesson {
        text: "He arrived, and now he is winded. A minion cannot attack on the turn it is played. When one is ready, its border turns green.",
        focus: Focus::Mine(CardId::Spearhand),
        gate: Gate::Next,
    },
    Lesson {
        text: "Nothing left to spend. End the turn with the button, or E. The straw takes a turn, and then we go again.",
        focus: Focus::EndTurn,
        gate: Gate::EndTurn,
    },
    Lesson {
        text: WATCH,
        focus: Focus::None,
        gate: Gate::EnemyTurn,
    },
    Lesson {
        text: "An Ember Mote. Two attack, one health, and as winded as your Spearhand was. Your Spearhand has rested. See the green.",
        focus: Focus::Foe(CardId::EmberMote),
        gate: Gate::Next,
    },
    Lesson {
        text: "Click the Spearhand, then the Imp. They strike at once: he deals one, it deals two, and neither walks away. That is a trade. A fair one, when it clears their line.",
        focus: Focus::Mine(CardId::Spearhand),
        gate: Gate::Attack(CardId::Spearhand, Aim::Minion(CardId::EmberMote)),
    },
    Lesson {
        text: "Two crystals now. The Shieldbearer has Guard: while she stands, the straw must hit her before it can hit you. Play her.",
        focus: Focus::Hand(CardId::Shieldbearer),
        gate: Gate::Play(CardId::Shieldbearer),
    },
    Lesson {
        text: "End the turn. She will be ready when it comes back around.",
        focus: Focus::EndTurn,
        gate: Gate::EndTurn,
    },
    Lesson {
        text: WATCH,
        focus: Focus::None,
        gate: Gate::EnemyTurn,
    },
    Lesson {
        text: "A Wavesinger. Three health. Your Shieldbearer swings for one, and that would take three turns. There is a faster way.",
        focus: Focus::Foe(CardId::Wavesinger),
        gate: Gate::Next,
    },
    Lesson {
        text: "Spells resolve the moment you cast them and ask for nothing but mana. Ember Bolt deals three. Click it, then the Wavesinger.",
        focus: Focus::Hand(CardId::EmberBolt),
        gate: Gate::Cast(CardId::EmberBolt, Aim::Minion(CardId::Wavesinger)),
    },
    Lesson {
        text: "Their line is empty, so nothing stands between your Shieldbearer and the straw. Click her, then the warden. One damage is one damage.",
        focus: Focus::Mine(CardId::Shieldbearer),
        gate: Gate::Attack(CardId::Shieldbearer, Aim::Warden),
    },
    Lesson {
        text: "You hold one crystal and a Lightning that needs a minion to land on. A card you cannot use goes grey. Keep it. End the turn.",
        focus: Focus::EndTurn,
        gate: Gate::EndTurn,
    },
    Lesson {
        text: WATCH,
        focus: Focus::None,
        gate: Gate::EnemyTurn,
    },
    Lesson {
        text: "A Mercenary, three and three. Too fat for Lightning alone. But you have something the cards do not: a power of your own.",
        focus: Focus::Foe(CardId::Mercenary),
        gate: Gate::Next,
    },
    Lesson {
        text: "Spark. Two mana, one damage, anywhere, once a turn. Press Q or the button, then choose the Mercenary. Every warden has a power. Yours is a small knife that never dulls.",
        focus: Focus::Power,
        gate: Gate::Power(Aim::Minion(CardId::Mercenary)),
    },
    Lesson {
        text: "Two health left on it. Now Lightning reaches. Cast it, aim it.",
        focus: Focus::Hand(CardId::Lightning),
        gate: Gate::Cast(CardId::Lightning, Aim::Minion(CardId::Mercenary)),
    },
    Lesson {
        text: "Clear line again. Shieldbearer, warden.",
        focus: Focus::Mine(CardId::Shieldbearer),
        gate: Gate::Attack(CardId::Shieldbearer, Aim::Warden),
    },
    Lesson {
        text: "End it.",
        focus: Focus::EndTurn,
        gate: Gate::EndTurn,
    },
    Lesson {
        text: WATCH,
        focus: Focus::None,
        gate: Gate::EnemyTurn,
    },
    Lesson {
        text: "A Knight, four and four, and it will be rested next turn. Guard cuts both ways: it must hit your Shieldbearer first, and she will not survive it. That is what she is for.",
        focus: Focus::Foe(CardId::Knight),
        gate: Gate::Next,
    },
    Lesson {
        text: "You have five crystals. Your own Knight costs four. Bigger cards, bigger turns. That is the whole shape of a game. Play him.",
        focus: Focus::Hand(CardId::Knight),
        gate: Gate::Play(CardId::Knight),
    },
    Lesson {
        text: "Their Knight is winded and carries no Guard, so the road to the warden is still open. Shieldbearer, one more time.",
        focus: Focus::Mine(CardId::Shieldbearer),
        gate: Gate::Attack(CardId::Shieldbearer, Aim::Warden),
    },
    Lesson {
        text: "End the turn, and brace.",
        focus: Focus::EndTurn,
        gate: Gate::EndTurn,
    },
    Lesson {
        text: WATCH,
        focus: Focus::None,
        gate: Gate::EnemyTurn,
    },
    Lesson {
        text: "She fell. Their Knight is scratched. And the straw put down a Champion with Shield: the first hit it takes does nothing at all. Never spend a spell on a shield. Poke it with something cheap first.",
        focus: Focus::Foe(CardId::Champion),
        gate: Gate::Next,
    },
    Lesson {
        text: "Now count. The warden has nine. Your Knight is rested and swings for four. Dragon Breath deals six. Four and six is ten. That is lethal, and lethal is the only sum that matters. Attack first, then cast, so a surprise cannot cost you the spell.",
        focus: Focus::Warden,
        gate: Gate::Next,
    },
    Lesson {
        text: "Knight, warden.",
        focus: Focus::Mine(CardId::Knight),
        gate: Gate::Attack(CardId::Knight, Aim::Warden),
    },
    Lesson {
        text: "Six mana, six damage. Finish it.",
        focus: Focus::Hand(CardId::DragonBreath),
        gate: Gate::Cast(CardId::DragonBreath, Aim::Warden),
    },
    Lesson {
        text: "There. Only straw again.",
        focus: Focus::None,
        gate: Gate::Win,
    },
];

pub const REFUSALS: [&str; 4] = [
    "Not that. Do what I said, and then do what you like.",
    "The yard has rules. Follow the light.",
    "Patience. The lesson comes first.",
    "Later. The straw can wait, and so can that.",
];
