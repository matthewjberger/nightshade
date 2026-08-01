#[derive(Default)]
pub struct ScheduledCapture {
    pub path: Option<std::path::PathBuf>,
    pub next: Option<std::path::PathBuf>,
    pub frame: u64,
    pub exit_after: bool,
    pub elapsed: u64,
    pub queued: bool,
    pub settle: u32,
    pub waiting: bool,
}

pub const QUIET_FRAMES: u32 = 8;

pub const NEXT_PICTURE_AFTER: u64 = 2;

pub const FRAMES_AFTER_PICTURE: u64 = 40;
