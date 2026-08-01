use nightshade::prelude::*;
use nightshade_openxr::contract::XrHand;

#[derive(Default, Clone, Copy)]
pub struct VisorHandles {
    pub root: Entity,
    pub canvas: Entity,
    pub pin_button: Entity,
    pub flight_button: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct Gauntlet {
    pub flare: Entity,
    pub grip_armed: bool,
    pub bolt_cooldown: f32,
}

#[derive(Default)]
pub struct Targeting {
    pub arming_hand: Option<XrHand>,
    pub locks: Vec<Entity>,
    pub lock_timer: f32,
    pub launched: f32,
}

#[derive(Default)]
pub struct Jarvis {
    pub handles: VisorHandles,
    pub velocity: Vec3,
    pub acceleration: f32,
    pub gauntlets: [Gauntlet; 2],
    pub targeting: Targeting,
    pub flight_mode: bool,
    pub boot_start: Option<f32>,
    pub turn_armed: bool,
    pub kills: u32,
    pub shots: u32,
    pub kill_flash: f32,
}

impl Jarvis {
    pub fn gauntlet_mut(&mut self, hand: XrHand) -> &mut Gauntlet {
        match hand {
            XrHand::Left => &mut self.gauntlets[0],
            XrHand::Right => &mut self.gauntlets[1],
        }
    }
}
