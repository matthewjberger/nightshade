use nightshade::prelude::{Vec3, Vec4};

pub const STRAIGHT: usize = 0;
pub const CORNER: usize = 1;
pub const FINISH: usize = 2;
pub const TRUCK: usize = 3;
pub const MODEL_COUNT: usize = 4;

pub const COLORMAP: &[u8] = include_bytes!("../assets/models/Textures/colormap.png");
pub const COLORMAP_NAME: &str = "Textures/colormap.png";

pub const MODEL_FILES: [&[u8]; MODEL_COUNT] = [
    include_bytes!("../assets/models/track-straight.glb"),
    include_bytes!("../assets/models/track-corner.glb"),
    include_bytes!("../assets/models/track-finish.glb"),
    include_bytes!("../assets/models/vehicle-truck-red.glb"),
];

pub const TILE: f32 = 10.0;
pub const SIDES: [usize; 4] = [8, 5, 8, 5];
pub const HEADINGS: [Vec3; 4] = [
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, -1.0),
];

pub const LINE: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const LINE_FORWARD: Vec3 = Vec3::new(1.0, 0.0, 0.0);
pub const CAR_START: Vec3 = Vec3::new(-3.0, 0.0, 0.0);
pub const CAR_YAW: f32 = std::f32::consts::FRAC_PI_2;
pub const MINIMUM_LAP_SECONDS: f32 = 5.0;

pub const ENGINE: f32 = 20.0;
pub const BRAKING: f32 = 28.0;
pub const MAXIMUM_SPEED: f32 = 32.0;
pub const REVERSE_SPEED: f32 = 8.0;
pub const DRAG: f32 = 0.4;
pub const GRIP: f32 = 7.0;
pub const DRIFT_GRIP: f32 = 1.3;
pub const TURN_RATE: f32 = 2.3;
pub const TURN_REFERENCE: f32 = 6.0;
pub const STEER_RATE: f32 = 6.0;
pub const MOUSE_LOCK: f32 = 2.4;
pub const WALL_INSET: f32 = 1.5;
pub const WALL_SCRUB: f32 = 3.0;

pub const CAMERA_BACK: f32 = 13.0;
pub const CAMERA_HEIGHT: f32 = 6.5;
pub const CAMERA_LAG: f32 = 6.0;
pub const CAMERA_AIM: f32 = 1.6;
pub const OVERVIEW_EYE: Vec3 = Vec3::new(35.0, 105.0, 108.0);
pub const OVERVIEW_AIM: Vec3 = Vec3::new(35.0, 0.0, 30.0);

pub const FIELD_NAME: &str = "hairpin_field";
pub const FIELD_COLOR: [f32; 4] = [0.30, 0.51, 0.23, 1.0];
pub const FIELD_CENTER: Vec3 = Vec3::new(35.0, -0.05, 30.0);
pub const FIELD_SIZE: Vec3 = Vec3::new(300.0, 1.0, 300.0);

pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.04, 0.05, 0.08, 0.86);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.14);
pub const TEXT_COLOR: Vec4 = Vec4::new(0.93, 0.95, 1.0, 1.0);
pub const OUTLINE_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
pub const MENU_WIDTH: f32 = 360.0;
pub const MENU_HEIGHT: f32 = 250.0;
pub const HUD_WIDTH: f32 = 720.0;
pub const HUD_HEIGHT: f32 = 40.0;
pub const HUD_MARGIN: f32 = 18.0;
pub const TITLE_SIZE: f32 = 34.0;
pub const HEADING_SIZE: f32 = 26.0;
pub const LABEL_SIZE: f32 = 18.0;
pub const ROW_HEIGHT: f32 = 34.0;

pub const TOUCH_CONTROLS: bool = cfg!(target_os = "android");
pub const PAD_COUNT: usize = 5;
pub const PAD_STEER_LEFT: usize = 0;
pub const PAD_STEER_RIGHT: usize = 1;
pub const PAD_BRAKE: usize = 2;
pub const PAD_DRIFT: usize = 3;
pub const PAD_THROTTLE: usize = 4;
pub const PAD_AREAS: [[f32; 4]; PAD_COUNT] = [
    [0.03, 0.58, 0.16, 0.36],
    [0.21, 0.58, 0.16, 0.36],
    [0.55, 0.58, 0.12, 0.36],
    [0.69, 0.58, 0.12, 0.36],
    [0.83, 0.58, 0.14, 0.36],
];
pub const PAD_LABELS: [&str; PAD_COUNT] = ["<", ">", "BRAKE", "DRIFT", "GAS"];
pub const PAD_BACKGROUND: Vec4 = Vec4::new(0.04, 0.05, 0.08, 0.45);
pub const PAD_RADIUS: f32 = 16.0;
pub const PAUSE_AREA: [f32; 4] = [0.92, 0.02, 0.06, 0.09];
pub const PAUSE_LABEL: &str = "II";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Steer,
    Drive,
    Handbrake,
    Pause,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Steer, Self::Drive, Self::Handbrake, Self::Pause]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Steer => "steer",
            Self::Drive => "drive",
            Self::Handbrake => "handbrake",
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
        use nightshade_input::{Binding, Press, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Steer => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::KeyD),
                    negative: Press::Key(KeyCode::KeyA),
                },
                Binding::Pair {
                    positive: Press::Key(KeyCode::ArrowRight),
                    negative: Press::Key(KeyCode::ArrowLeft),
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Drive => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::KeyW),
                    negative: Press::Key(KeyCode::KeyS),
                },
                Binding::Pair {
                    positive: Press::Key(KeyCode::ArrowUp),
                    negative: Press::Key(KeyCode::ArrowDown),
                },
                Binding::Pair {
                    positive: Press::Trigger(GamepadAxis::RightTrigger),
                    negative: Press::Trigger(GamepadAxis::LeftTrigger),
                },
                Binding::Pair {
                    positive: Press::Pad(GamepadButton::South),
                    negative: Press::Pad(GamepadButton::West),
                },
            ],
            Self::Handbrake => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
        }
    }
}
