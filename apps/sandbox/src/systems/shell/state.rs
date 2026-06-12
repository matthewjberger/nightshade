use nightshade::prelude::*;

#[derive(Default)]
pub struct SandboxShellContext {
    pub queue: Vec<ShellAction>,
}

#[derive(Debug, Clone)]
pub enum ShellAction {
    TeleportPlayer(Vec3),
    PrintPlayerPosition,
    ResetPlayer,
    SetFly(Option<bool>),
    SetThirdPerson(Option<bool>),
    SetStrideWheel(Option<bool>),
    SetTimeOfDay(f32),
    PauseTimeOfDay(bool),
    SetTimeSpeed(f32),
    SaveLevel,
}
