use crate::data::{Prop, Tool};
use nalgebra_glm::{Quat, Vec3};
use nightshade::prelude::Entity;
use nightshade_cli::clap;
use nightshade_sdf::contract::SdfShape;
use nightshade_sdf::contract::render_data::SdfChunkKey;
use nightshade_sdf::contract::render_data::SdfDebugView;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(clap::Args, Default)]
pub struct SdfArguments {
    #[arg(
        long,
        help = "Open on one of the cache pictures: 1 bricks, 2 clipmap levels, 3 march steps, 4 baked bricks"
    )]
    pub view: Option<u32>,
    #[arg(
        long,
        help = "Draw the field at this share of the window, from 0.25 to 1"
    )]
    pub scale: Option<f32>,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: Entity,
    pub rate_readout: Entity,
    pub view_readout: Entity,
    pub tool_slots: [Entity; 4],
    pub tool_labels: [Entity; 4],
}

#[derive(Default)]
pub struct SdfScene {
    pub camera: Option<Entity>,
    pub tunnel: Option<Entity>,
    pub hole: Option<Entity>,
    pub digging: bool,
    pub hud: HudHandles,
    pub last_rate: u32,
    pub last_gpu_tenths: u32,
    pub last_view: Option<SdfDebugView>,
    pub last_scale: f32,
    pub last_tool: Option<Tool>,
    pub ground: HashMap<SdfChunkKey, Entity>,
    pub stones: VecDeque<Entity>,
    pub waiting: Vec<Waiting>,
    pub terrain: Option<SdfShape>,
    pub tool: Tool,
    pub aim: Option<Vec3>,
    pub eye: Option<Vec3>,
    pub bite: Option<Bite>,
    pub strokes: Vec<Entity>,
    pub stroke_end: Option<Vec3>,
    pub next_order: u32,
}

#[derive(Clone, Copy)]
pub struct Bite {
    pub edit: Entity,
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy)]
pub struct Waiting {
    pub prop: Prop,
    pub spot: Vec3,
    pub ground: Vec3,
    pub rotation: Quat,
    pub anchor: Entity,
}
