use nightshade::prelude::*;

#[derive(Default, Clone, Copy)]
pub struct Agent(pub Entity);

#[derive(Default, Clone, Copy)]
pub struct Nameplate(pub Entity);

#[derive(Default, Clone)]
pub struct FoxName(pub String);

#[derive(Default, Clone, Copy)]
pub struct NameColor(pub Vec4);

#[derive(Default, Clone, Copy)]
pub struct Facing(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Clip(pub Option<usize>);

#[derive(Default, Clone, Copy)]
pub struct GroundSpeed(pub f32);

#[derive(Default, Clone, Copy)]
pub struct StepBack(pub Option<Vec3>);

#[derive(Default, Clone, Copy)]
pub struct StillFrames(pub u32);
