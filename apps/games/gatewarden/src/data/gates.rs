#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GateRule {
    #[default]
    None,
    Thorns,
    Tithe,
    Drought,
    Riptide,
    Ashfall,
    Stonewall,
    Eclipse,
}

impl GateRule {
    pub fn name(self) -> &'static str {
        match self {
            GateRule::None => "Open gate",
            GateRule::Thorns => "Thorns",
            GateRule::Tithe => "Tithe",
            GateRule::Drought => "Drought",
            GateRule::Riptide => "Riptide",
            GateRule::Ashfall => "Ashfall",
            GateRule::Stonewall => "Stonewall",
            GateRule::Eclipse => "Eclipse",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            GateRule::None => "No gate rule holds here",
            GateRule::Thorns => "A minion that attacks takes 1 damage",
            GateRule::Tithe => "Your first card each turn costs 1 more",
            GateRule::Drought => "Healing you receive is halved",
            GateRule::Riptide => "The warden draws an extra card each turn",
            GateRule::Ashfall => "Each turn ends with 1 damage to both heroes",
            GateRule::Stonewall => "The warden starts behind 10 armor",
            GateRule::Eclipse => "Your hero power costs 1 more",
        }
    }
}

pub const STONEWALL_ARMOR: i32 = 10;
pub const THORNS_DAMAGE: i32 = 1;
pub const ASHFALL_DAMAGE: i32 = 1;
pub const TITHE_SURCHARGE: u32 = 1;
pub const ECLIPSE_SURCHARGE: u32 = 1;

pub fn gate_rule(gate: u32) -> GateRule {
    match gate {
        1 => GateRule::None,
        2 => GateRule::Thorns,
        3 => GateRule::Drought,
        4 => GateRule::Riptide,
        5 => GateRule::Tithe,
        6 => GateRule::Ashfall,
        7 => GateRule::Eclipse,
        _ => GateRule::Stonewall,
    }
}
