use nightshade::prelude::*;
use nightshade_render::data::config::{DebugDraw, RenderSettings, RendererState};

#[derive(Default)]
pub struct SandboxShellContext {
    pub queue: Vec<ShellAction>,
    pub pose_names: Vec<String>,
    pub readout: Readout,
    pub wanted: Wanted,
}

impl SandboxShellContext {
    pub fn settings(&mut self) -> &mut RenderSettings {
        let readout = &self.readout.settings;
        self.wanted.settings.get_or_insert_with(|| readout.clone())
    }

    pub fn debug_draw(&mut self) -> &mut DebugDraw {
        let readout = &self.readout.debug_draw;
        self.wanted
            .debug_draw
            .get_or_insert_with(|| readout.clone())
    }

    pub fn renderer_state(&mut self) -> &mut RendererState {
        let readout = &self.readout.renderer_state;
        self.wanted
            .renderer_state
            .get_or_insert_with(|| readout.clone())
    }

    pub fn window(&mut self) -> &mut WindowWish {
        let readout = self.readout.window;
        self.wanted.window.get_or_insert(readout)
    }
}

#[derive(Default)]
pub struct Readout {
    pub settings: RenderSettings,
    pub debug_draw: DebugDraw,
    pub renderer_state: RendererState,
    pub gamepad: GamepadReadout,
    pub time_speed: f32,
    pub frames_per_second: f32,
    pub delta_time: f32,
    pub uptime_milliseconds: u64,
    pub window: WindowWish,
}

#[derive(Default)]
pub struct GamepadReadout {
    pub connected: bool,
    pub just_pressed: Vec<String>,
    pub held: Vec<String>,
    pub left_stick: (f32, f32),
    pub right_stick: (f32, f32),
    pub triggers: (f32, f32),
}

#[derive(Clone, Copy, Default)]
pub struct WindowWish {
    pub use_fullscreen: bool,
    pub show_cursor: bool,
}

#[derive(Default)]
pub struct Wanted {
    pub settings: Option<RenderSettings>,
    pub debug_draw: Option<DebugDraw>,
    pub renderer_state: Option<RendererState>,
    pub window: Option<WindowWish>,
    pub time_speed: Option<f32>,
    pub exit: bool,
}

#[derive(Debug, Clone)]
pub enum ShellAction {
    TeleportPlayer(Vec3),
    PrintPlayerPosition,
    ResetPlayer,
    SetFly(Option<bool>),
    SetThirdPerson(Option<bool>),
    SetStrideWheel(Option<bool>),
    SetSpheres(Option<bool>),
    SetSkeleton(Option<bool>),
    SetPose(String),
    SetRagdoll(String),
    SetTimeOfDay(f32),
    PauseTimeOfDay(bool),
    SetTimeSpeed(f32),
    ReadCameraPosition,
    TeleportCamera(Vec3),
    SetFieldOfView(Option<f32>),
    MoveEntity(Entity, String, Vec3),
    RotateEntity(Entity, String, Vec3),
    ScaleEntity(Entity, String, Vec3),
    DeleteEntity(Entity, String),
    InspectEntity(Entity, String),
}
