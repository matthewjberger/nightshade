use crate::data::{CursorPartKind, Growth, RoomKind, Team, Upgrade};
use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy, Debug)]
pub struct Called(pub &'static str);

#[derive(Default, Clone, Copy, Debug)]
pub struct Title(pub &'static str);

#[derive(Default, Clone, Copy, Debug)]
pub struct Slot(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Column(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Row(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Experience(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rapport(pub u32);

#[derive(Default, Clone, Debug)]
pub struct Bonds(pub Vec<(Entity, u32)>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Morale(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Wounded(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Deployed(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Busy(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Kills(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Battles(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Joined(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct ArcStep(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Boon(pub Growth);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fallen(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Marks(pub u32);

#[derive(Default, Clone, Debug)]
pub struct Taken(pub Vec<Upgrade>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Avatar(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Merit(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Hp(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct MaxHp(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Mp(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct MaxMp(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Attack(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Defense(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Magic(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Resist(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct MoveRange(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Jump(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Evasion(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Ct(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flipped(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flash(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct FlashTint(pub [f32; 4]);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stunned(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Suppressed(pub u32);

#[derive(Default, Clone, Debug)]
pub struct Room {
    pub kind: RoomKind,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Stride(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Shade {
    pub owner: Entity,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Owner(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fill(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Holder(pub Option<Team>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flag(pub Option<Entity>);

#[derive(Default, Clone, Debug)]
pub struct StatusIcon {
    pub owner: Entity,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Age(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tint(pub [f32; 4]);

#[derive(Default, Clone, Debug)]
pub struct CursorPart {
    pub kind: CursorPartKind,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Pulse(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Strength(pub f32);
