use nightshade::prelude::Vec3;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    #[default]
    Whelp,
    Gunner,
    Lobber,
    Hulk,
    Warden,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyMood {
    #[default]
    Asleep,
    Waking,
    Hunting,
    Winding,
    Reeling,
    Dying,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    #[default]
    Medkit,
    Stimpack,
    Plate,
    Shells,
    Bullets,
    Rockets,
    Cells,
    RedKey,
    BlueKey,
    Chaingun,
    Launcher,
    Plasma,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[default]
    Lever,
    Red,
    Blue,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Health(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Timer(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Cooldown(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Drift(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fall(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Damage(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Splash(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Life(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct RestingHeight(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Group(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct ShutHeight(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct OpenShare(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Travel(pub f32);
