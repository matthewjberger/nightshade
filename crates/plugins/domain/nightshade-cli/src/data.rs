use clap::{Args, Parser};

pub const DEFAULT_CAPTURE_FRAME: u64 = 180;

#[derive(Args, Clone, Debug, Default)]
pub struct CaptureArguments {
    #[arg(
        long,
        value_name = "PATH",
        help = "Write one frame to this file, then close the window"
    )]
    pub capture: Option<std::path::PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        requires = "capture",
        help = "Write the frame two after the first to this file as well"
    )]
    pub capture_next: Option<std::path::PathBuf>,
    #[arg(
        long,
        value_name = "FRAMES",
        help = "How many frames after the scene arrives the picture is taken on"
    )]
    pub capture_frame: Option<u64>,
    #[arg(
        long,
        value_name = "FRAMES",
        help = "How many quiet frames of the loading pipeline say the scene has arrived"
    )]
    pub capture_settle: Option<u32>,
}

#[derive(Args, Clone, Debug, Default)]
pub struct WindowArguments {
    #[arg(
        long,
        value_name = "WIDTHxHEIGHT",
        help = "Open the window at this size in pixels"
    )]
    pub size: Option<WindowSize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl std::str::FromStr for WindowSize {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (width, height) = text
            .split_once('x')
            .ok_or_else(|| format!("expected WIDTHxHEIGHT, got {text}"))?;
        Ok(Self {
            width: width.parse().map_err(|error| format!("{error}"))?,
            height: height.parse().map_err(|error| format!("{error}"))?,
        })
    }
}

#[derive(Args, Clone, Debug, Default)]
pub struct NoArguments {}

#[derive(Parser, Debug, Default)]
pub struct Arguments<T: Args> {
    #[command(flatten)]
    pub capture: CaptureArguments,
    #[command(flatten)]
    pub window: WindowArguments,
    #[command(flatten)]
    pub app: T,
}
