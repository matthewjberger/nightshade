use nightshade::prelude::*;

#[derive(Default)]
pub struct SandboxShellContext {
    pub queue: Vec<ShellAction>,
    pub pose_names: Vec<String>,
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
    SetTimeOfDay(f32),
    PauseTimeOfDay(bool),
    SetTimeSpeed(f32),
    SaveLevel,
}
