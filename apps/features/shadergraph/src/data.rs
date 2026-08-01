use nalgebra_glm::Vec4;
use nightshade_input::{Action, Binding};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Cut,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Cut]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Cut => "cut the node",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Cut => &[Binding::Key(KeyCode::KeyX), Binding::Key(KeyCode::Delete)],
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Uv,
    Middle,
    Time,
    Value,
    Across,
    Down,
    Add,
    Subtract,
    Multiply,
    Min,
    Max,
    Power,
    Sine,
    Cosine,
    Fract,
    Floor,
    Abs,
    Smoothstep,
    Length,
    Polar,
    Rotate,
    Step,
    Invert,
    Luminance,
    Noise,
    Picture,
    Mix,
    Palette,
    Sweep,
    Seam,
    Lattice,
    Output,
}

pub const EMPTY: &str = "vec4<f32>(0.0)";

pub fn title_of(kind: Kind) -> &'static str {
    match kind {
        Kind::Uv => "UV",
        Kind::Middle => "Middle",
        Kind::Time => "Time",
        Kind::Value => "Value",
        Kind::Across => "Across",
        Kind::Down => "Down",
        Kind::Add => "Add",
        Kind::Subtract => "Subtract",
        Kind::Multiply => "Multiply",
        Kind::Min => "Min",
        Kind::Max => "Max",
        Kind::Power => "Power",
        Kind::Sine => "Sine",
        Kind::Cosine => "Cosine",
        Kind::Fract => "Fract",
        Kind::Floor => "Floor",
        Kind::Abs => "Abs",
        Kind::Smoothstep => "Smooth",
        Kind::Length => "Length",
        Kind::Polar => "Polar",
        Kind::Rotate => "Rotate",
        Kind::Step => "Step",
        Kind::Invert => "Invert",
        Kind::Luminance => "Luma",
        Kind::Noise => "Noise",
        Kind::Picture => "Picture",
        Kind::Mix => "Mix",
        Kind::Palette => "Palette",
        Kind::Sweep => "Sweep",
        Kind::Seam => "Seam",
        Kind::Lattice => "Lattice",
        Kind::Output => "Output",
    }
}

pub fn inlets_of(kind: Kind) -> u32 {
    match kind {
        Kind::Uv | Kind::Middle | Kind::Time | Kind::Value | Kind::Sweep => 0,
        Kind::Across
        | Kind::Down
        | Kind::Sine
        | Kind::Cosine
        | Kind::Fract
        | Kind::Floor
        | Kind::Abs
        | Kind::Smoothstep
        | Kind::Length
        | Kind::Polar
        | Kind::Invert
        | Kind::Luminance
        | Kind::Noise
        | Kind::Picture
        | Kind::Palette
        | Kind::Seam
        | Kind::Lattice
        | Kind::Output => 1,
        Kind::Add
        | Kind::Subtract
        | Kind::Multiply
        | Kind::Min
        | Kind::Max
        | Kind::Power
        | Kind::Rotate
        | Kind::Step => 2,
        Kind::Mix => 3,
    }
}

pub fn outlets_of(kind: Kind) -> u32 {
    match kind {
        Kind::Output => 0,
        _ => 1,
    }
}

pub fn takes_a_value(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Value | Kind::Noise | Kind::Palette | Kind::Sweep | Kind::Seam | Kind::Lattice
    )
}

pub fn value_span_of(kind: Kind) -> (f32, f32, f32) {
    match kind {
        Kind::Noise => (0.25, 16.0, 1.0),
        Kind::Palette => (0.0, 1.0, 0.0),
        Kind::Sweep => (1.0, 20.0, 7.0),
        Kind::Seam => (0.005, 0.5, 0.12),
        Kind::Lattice => (2.0, 64.0, 24.0),
        _ => (0.0, 48.0, 1.0),
    }
}

pub fn tint_of(kind: Kind) -> Vec4 {
    match kind {
        Kind::Uv | Kind::Middle | Kind::Time => Vec4::new(0.24, 0.44, 0.66, 1.0),
        Kind::Value => Vec4::new(0.32, 0.50, 0.38, 1.0),
        Kind::Across | Kind::Down | Kind::Length | Kind::Polar | Kind::Rotate | Kind::Luminance => {
            Vec4::new(0.26, 0.52, 0.56, 1.0)
        }
        Kind::Add
        | Kind::Subtract
        | Kind::Multiply
        | Kind::Min
        | Kind::Max
        | Kind::Power
        | Kind::Sine
        | Kind::Cosine
        | Kind::Fract
        | Kind::Floor
        | Kind::Abs
        | Kind::Step
        | Kind::Invert
        | Kind::Smoothstep => Vec4::new(0.46, 0.34, 0.60, 1.0),
        Kind::Noise | Kind::Mix | Kind::Palette => Vec4::new(0.62, 0.42, 0.28, 1.0),
        Kind::Sweep | Kind::Seam | Kind::Lattice => Vec4::new(0.20, 0.54, 0.72, 1.0),
        Kind::Picture => Vec4::new(0.30, 0.46, 0.56, 1.0),
        Kind::Output => Vec4::new(0.68, 0.28, 0.30, 1.0),
    }
}

pub fn wgsl_of(kind: Kind, inputs: &[String], value: f32, slot: u32) -> String {
    let first = inputs.first().map_or(EMPTY.to_string(), String::clone);
    let second = inputs.get(1).map_or(EMPTY.to_string(), String::clone);
    let third = inputs.get(2).map_or(EMPTY.to_string(), String::clone);
    match kind {
        Kind::Uv => "vec4<f32>(uv, 0.0, 1.0)".to_string(),
        Kind::Middle => "vec4<f32>(uv - vec2<f32>(u.aspect * 0.5, 0.5), 0.0, 1.0)".to_string(),
        Kind::Time => "vec4<f32>(u.time)".to_string(),
        Kind::Value => format!("vec4<f32>({value:?})"),
        Kind::Across => format!("vec4<f32>({first}.x)"),
        Kind::Down => format!("vec4<f32>({first}.y)"),
        Kind::Add => format!("({first} + {second})"),
        Kind::Subtract => format!("({first} - {second})"),
        Kind::Multiply => format!("({first} * {second})"),
        Kind::Min => format!("min({first}, {second})"),
        Kind::Max => format!("max({first}, {second})"),
        Kind::Power => format!("pow(abs({first}), {second})"),
        Kind::Sine => format!("sin({first})"),
        Kind::Cosine => format!("cos({first})"),
        Kind::Fract => format!("fract({first})"),
        Kind::Floor => format!("floor({first})"),
        Kind::Abs => format!("abs({first})"),
        Kind::Smoothstep => {
            format!("smoothstep(vec4<f32>(0.0), vec4<f32>(1.0), {first})")
        }
        Kind::Length => format!("vec4<f32>(length({first}.xy))"),
        Kind::Polar => format!(
            "vec4<f32>(atan2({first}.y, {first}.x) / 6.28318 + 0.5, length({first}.xy), 0.0, 1.0)"
        ),
        Kind::Rotate => format!("vec4<f32>(spin({first}.xy, {second}.x), {first}.zw)"),
        Kind::Step => format!("step({second}, {first})"),
        Kind::Invert => format!("(vec4<f32>(1.0) - {first})"),
        Kind::Luminance => format!("vec4<f32>(luma({first}.rgb))"),
        Kind::Noise => format!("vec4<f32>(fbm({first}.xy * {value:?}))"),
        Kind::Picture => {
            format!("textureSample(picture{slot}, picture_sampler, {first}.xy)")
        }
        Kind::Mix => format!("mix({first}, {second}, {third})"),
        Kind::Palette => format!("palette({first}.x + {value:?})"),
        Kind::Sweep => format!("vec4<f32>(sweep(u.time, {value:?}))"),
        Kind::Seam => format!("vec4<f32>(seam({first}.x, {value:?}))"),
        Kind::Lattice => format!("vec4<f32>(lattice({first}.xy, {value:?}))"),
        Kind::Output => format!("vec4<f32>({first}.rgb, 1.0)"),
    }
}

pub fn inlet_names_of(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Uv | Kind::Middle | Kind::Time | Kind::Value | Kind::Sweep => &[],
        Kind::Across | Kind::Down | Kind::Length | Kind::Polar | Kind::Luminance => &["v"],
        Kind::Invert => &["a"],
        Kind::Rotate => &["v", "turn"],
        Kind::Step => &["a", "edge"],
        Kind::Sine | Kind::Cosine | Kind::Fract | Kind::Floor | Kind::Abs | Kind::Smoothstep => {
            &["a"]
        }
        Kind::Noise | Kind::Picture | Kind::Lattice => &["uv"],
        Kind::Seam => &["d"],
        Kind::Add | Kind::Subtract | Kind::Multiply | Kind::Min | Kind::Max | Kind::Power => {
            &["a", "b"]
        }
        Kind::Mix => &["a", "b", "t"],
        Kind::Palette => &["n"],
        Kind::Output => &["rgb"],
    }
}

pub fn outlet_names_of(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Uv | Kind::Middle => &["uv"],
        Kind::Time => &["t"],
        Kind::Value | Kind::Noise | Kind::Sweep => &["n"],
        Kind::Seam => &["band"],
        Kind::Lattice => &["wire"],
        Kind::Picture => &["rgba"],
        Kind::Across => &["x"],
        Kind::Down => &["y"],
        Kind::Add => &["a+b"],
        Kind::Subtract => &["a-b"],
        Kind::Multiply => &["a*b"],
        Kind::Min => &["min"],
        Kind::Max => &["max"],
        Kind::Power => &["a^b"],
        Kind::Sine => &["sin"],
        Kind::Cosine => &["cos"],
        Kind::Fract => &["frac"],
        Kind::Floor => &["floor"],
        Kind::Abs => &["abs"],
        Kind::Smoothstep => &["s"],
        Kind::Length => &["len"],
        Kind::Polar => &["a, r"],
        Kind::Rotate => &["v"],
        Kind::Step => &["0 or 1"],
        Kind::Invert => &["1-a"],
        Kind::Luminance => &["luma"],
        Kind::Mix => &["mix"],
        Kind::Palette => &["rgb"],
        Kind::Output => &[],
    }
}

pub fn note_of(kind: Kind, value: f32) -> String {
    match kind {
        Kind::Uv => "x right, y down".to_string(),
        Kind::Middle => "from the center".to_string(),
        Kind::Time => "seconds".to_string(),
        Kind::Value => format!("{value:.3}"),
        Kind::Across => "v.x".to_string(),
        Kind::Down => "v.y".to_string(),
        Kind::Add => "a + b".to_string(),
        Kind::Subtract => "a - b".to_string(),
        Kind::Multiply => "a * b".to_string(),
        Kind::Min => "min(a, b)".to_string(),
        Kind::Max => "max(a, b)".to_string(),
        Kind::Power => "a to the b".to_string(),
        Kind::Sine => "sin(a)".to_string(),
        Kind::Cosine => "cos(a)".to_string(),
        Kind::Fract => "fract(a)".to_string(),
        Kind::Floor => "floor(a)".to_string(),
        Kind::Abs => "abs(a)".to_string(),
        Kind::Smoothstep => "smooth 0 to 1".to_string(),
        Kind::Length => "length(v)".to_string(),
        Kind::Polar => "angle and radius".to_string(),
        Kind::Rotate => "turns v around".to_string(),
        Kind::Step => "0 below, 1 above".to_string(),
        Kind::Invert => "1 - a".to_string(),
        Kind::Luminance => "how bright".to_string(),
        Kind::Noise => format!("scale  {value:.2}"),
        Kind::Picture => "no picture".to_string(),
        Kind::Mix => "a to b by t".to_string(),
        Kind::Palette => format!("turn  {value:.2}"),
        Kind::Sweep => format!("cycle  {value:.2} s"),
        Kind::Seam => format!("width  {value:.3}"),
        Kind::Lattice => format!("cells  {value:.0}"),
        Kind::Output => "what is drawn".to_string(),
    }
}

pub struct Family {
    pub name: &'static str,
    pub kinds: &'static [Kind],
}

pub const FAMILIES: [Family; 7] = [
    Family {
        name: "Sources",
        kinds: &[Kind::Uv, Kind::Middle, Kind::Time, Kind::Value, Kind::Sweep],
    },
    Family {
        name: "Space",
        kinds: &[
            Kind::Across,
            Kind::Down,
            Kind::Length,
            Kind::Polar,
            Kind::Rotate,
        ],
    },
    Family {
        name: "Arithmetic",
        kinds: &[
            Kind::Add,
            Kind::Subtract,
            Kind::Multiply,
            Kind::Min,
            Kind::Max,
            Kind::Power,
        ],
    },
    Family {
        name: "Shaping",
        kinds: &[
            Kind::Sine,
            Kind::Cosine,
            Kind::Fract,
            Kind::Floor,
            Kind::Abs,
            Kind::Smoothstep,
            Kind::Step,
            Kind::Invert,
        ],
    },
    Family {
        name: "Pattern",
        kinds: &[Kind::Noise, Kind::Lattice, Kind::Picture],
    },
    Family {
        name: "Color",
        kinds: &[Kind::Luminance, Kind::Mix, Kind::Palette, Kind::Seam],
    },
    Family {
        name: "Result",
        kinds: &[Kind::Output],
    },
];

pub struct Preset {
    pub name: &'static str,
    pub nodes: &'static [(Kind, f32, f32, f32)],
    pub wires: &'static [(usize, u32, usize, u32)],
}

pub const PRESETS: [Preset; 21] = [
    Preset {
        name: "Noise field",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Noise, 260.0, 40.0, 1.0),
            (Kind::Palette, 480.0, 40.0, 0.0),
            (Kind::Output, 700.0, 40.0, 0.0),
        ],
        wires: &[(0, 0, 1, 0), (1, 0, 2, 0), (2, 0, 3, 0)],
    },
    Preset {
        name: "Drifting noise",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Time, 40.0, 190.0, 0.0),
            (Kind::Value, 40.0, 320.0, 0.12),
            (Kind::Multiply, 260.0, 210.0, 0.0),
            (Kind::Add, 480.0, 100.0, 0.0),
            (Kind::Noise, 700.0, 100.0, 1.0),
            (Kind::Palette, 920.0, 100.0, 0.0),
            (Kind::Output, 1140.0, 100.0, 0.0),
        ],
        wires: &[
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (0, 0, 4, 0),
            (3, 0, 4, 1),
            (4, 0, 5, 0),
            (5, 0, 6, 0),
            (6, 0, 7, 0),
        ],
    },
    Preset {
        name: "Rings",
        nodes: &[
            (Kind::Middle, 40.0, 40.0, 0.0),
            (Kind::Length, 260.0, 40.0, 0.0),
            (Kind::Value, 260.0, 190.0, 24.0),
            (Kind::Multiply, 480.0, 60.0, 0.0),
            (Kind::Sine, 700.0, 60.0, 0.0),
            (Kind::Palette, 920.0, 60.0, 0.0),
            (Kind::Output, 1140.0, 60.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (3, 0, 4, 0),
            (4, 0, 5, 0),
            (5, 0, 6, 0),
        ],
    },
    Preset {
        name: "Ripple",
        nodes: &[
            (Kind::Middle, 40.0, 40.0, 0.0),
            (Kind::Length, 260.0, 40.0, 0.0),
            (Kind::Value, 260.0, 190.0, 40.0),
            (Kind::Multiply, 480.0, 60.0, 0.0),
            (Kind::Time, 40.0, 320.0, 0.0),
            (Kind::Value, 40.0, 450.0, 4.0),
            (Kind::Multiply, 260.0, 340.0, 0.0),
            (Kind::Subtract, 700.0, 160.0, 0.0),
            (Kind::Sine, 920.0, 160.0, 0.0),
            (Kind::Palette, 1140.0, 160.0, 0.0),
            (Kind::Output, 1360.0, 160.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (4, 0, 6, 0),
            (5, 0, 6, 1),
            (3, 0, 7, 0),
            (6, 0, 7, 1),
            (7, 0, 8, 0),
            (8, 0, 9, 0),
            (9, 0, 10, 0),
        ],
    },
    Preset {
        name: "Stripes",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Across, 260.0, 40.0, 0.0),
            (Kind::Value, 260.0, 190.0, 40.0),
            (Kind::Multiply, 480.0, 60.0, 0.0),
            (Kind::Sine, 700.0, 60.0, 0.0),
            (Kind::Palette, 920.0, 60.0, 0.0),
            (Kind::Output, 1140.0, 60.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (3, 0, 4, 0),
            (4, 0, 5, 0),
            (5, 0, 6, 0),
        ],
    },
    Preset {
        name: "Plasma",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Across, 260.0, 40.0, 0.0),
            (Kind::Down, 260.0, 170.0, 0.0),
            (Kind::Value, 260.0, 300.0, 18.0),
            (Kind::Multiply, 480.0, 40.0, 0.0),
            (Kind::Multiply, 480.0, 190.0, 0.0),
            (Kind::Time, 480.0, 340.0, 0.0),
            (Kind::Sine, 700.0, 40.0, 0.0),
            (Kind::Add, 700.0, 210.0, 0.0),
            (Kind::Sine, 920.0, 210.0, 0.0),
            (Kind::Add, 1140.0, 120.0, 0.0),
            (Kind::Palette, 1360.0, 120.0, 0.0),
            (Kind::Output, 1580.0, 120.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (0, 0, 2, 0),
            (1, 0, 4, 0),
            (3, 0, 4, 1),
            (2, 0, 5, 0),
            (3, 0, 5, 1),
            (4, 0, 7, 0),
            (5, 0, 8, 0),
            (6, 0, 8, 1),
            (8, 0, 9, 0),
            (7, 0, 10, 0),
            (9, 0, 10, 1),
            (10, 0, 11, 0),
            (11, 0, 12, 0),
        ],
    },
    Preset {
        name: "Warp",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Noise, 260.0, 40.0, 1.0),
            (Kind::Value, 260.0, 190.0, 0.5),
            (Kind::Multiply, 480.0, 60.0, 0.0),
            (Kind::Add, 700.0, 60.0, 0.0),
            (Kind::Noise, 920.0, 60.0, 1.0),
            (Kind::Palette, 1140.0, 60.0, 0.0),
            (Kind::Output, 1360.0, 60.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (0, 0, 4, 0),
            (3, 0, 4, 1),
            (4, 0, 5, 0),
            (5, 0, 6, 0),
            (6, 0, 7, 0),
        ],
    },
    Preset {
        name: "Vortex",
        nodes: &[
            (Kind::Middle, 40.0, 40.0, 0.0),
            (Kind::Polar, 260.0, 40.0, 0.0),
            (Kind::Across, 480.0, 40.0, 0.0),
            (Kind::Down, 480.0, 170.0, 0.0),
            (Kind::Value, 480.0, 300.0, 6.0),
            (Kind::Value, 480.0, 430.0, 18.0),
            (Kind::Multiply, 700.0, 40.0, 0.0),
            (Kind::Multiply, 700.0, 210.0, 0.0),
            (Kind::Add, 920.0, 120.0, 0.0),
            (Kind::Time, 920.0, 300.0, 0.0),
            (Kind::Subtract, 1140.0, 200.0, 0.0),
            (Kind::Sine, 1360.0, 200.0, 0.0),
            (Kind::Palette, 1580.0, 200.0, 0.0),
            (Kind::Output, 1800.0, 200.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 2, 0),
            (1, 0, 3, 0),
            (2, 0, 6, 0),
            (4, 0, 6, 1),
            (3, 0, 7, 0),
            (5, 0, 7, 1),
            (6, 0, 8, 0),
            (7, 0, 8, 1),
            (8, 0, 10, 0),
            (9, 0, 10, 1),
            (10, 0, 11, 0),
            (11, 0, 12, 0),
            (12, 0, 13, 0),
        ],
    },
    Preset {
        name: "Sun",
        nodes: &[
            (Kind::Middle, 40.0, 40.0, 0.0),
            (Kind::Length, 260.0, 40.0, 0.0),
            (Kind::Value, 260.0, 190.0, 1.0),
            (Kind::Subtract, 480.0, 60.0, 0.0),
            (Kind::Value, 480.0, 210.0, 0.0),
            (Kind::Max, 700.0, 80.0, 0.0),
            (Kind::Value, 700.0, 230.0, 4.0),
            (Kind::Power, 920.0, 100.0, 0.0),
            (Kind::Palette, 1140.0, 100.0, 0.0),
            (Kind::Output, 1360.0, 100.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (2, 0, 3, 0),
            (1, 0, 3, 1),
            (3, 0, 5, 0),
            (4, 0, 5, 1),
            (5, 0, 7, 0),
            (6, 0, 7, 1),
            (7, 0, 8, 0),
            (8, 0, 9, 0),
        ],
    },
    Preset {
        name: "Bands",
        nodes: &[
            (Kind::Middle, 40.0, 40.0, 0.0),
            (Kind::Length, 260.0, 40.0, 0.0),
            (Kind::Value, 260.0, 190.0, 6.0),
            (Kind::Multiply, 480.0, 60.0, 0.0),
            (Kind::Time, 480.0, 210.0, 0.0),
            (Kind::Subtract, 700.0, 100.0, 0.0),
            (Kind::Fract, 920.0, 100.0, 0.0),
            (Kind::Palette, 1140.0, 100.0, 0.0),
            (Kind::Output, 1360.0, 100.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (3, 0, 5, 0),
            (4, 0, 5, 1),
            (5, 0, 6, 0),
            (6, 0, 7, 0),
            (7, 0, 8, 0),
        ],
    },
    Preset {
        name: "Marble",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Across, 260.0, 40.0, 0.0),
            (Kind::Value, 260.0, 170.0, 20.0),
            (Kind::Noise, 260.0, 300.0, 1.0),
            (Kind::Value, 260.0, 430.0, 8.0),
            (Kind::Multiply, 480.0, 60.0, 0.0),
            (Kind::Multiply, 480.0, 330.0, 0.0),
            (Kind::Add, 700.0, 190.0, 0.0),
            (Kind::Sine, 920.0, 190.0, 0.0),
            (Kind::Palette, 1140.0, 190.0, 0.0),
            (Kind::Output, 1360.0, 190.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (0, 0, 3, 0),
            (1, 0, 5, 0),
            (2, 0, 5, 1),
            (3, 0, 6, 0),
            (4, 0, 6, 1),
            (5, 0, 7, 0),
            (6, 0, 7, 1),
            (7, 0, 8, 0),
            (8, 0, 9, 0),
            (9, 0, 10, 0),
        ],
    },
    Preset {
        name: "Grid",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Value, 40.0, 190.0, 10.0),
            (Kind::Multiply, 260.0, 60.0, 0.0),
            (Kind::Fract, 480.0, 60.0, 0.0),
            (Kind::Value, 480.0, 210.0, 0.5),
            (Kind::Subtract, 700.0, 80.0, 0.0),
            (Kind::Abs, 920.0, 80.0, 0.0),
            (Kind::Across, 1140.0, 40.0, 0.0),
            (Kind::Down, 1140.0, 170.0, 0.0),
            (Kind::Min, 1360.0, 100.0, 0.0),
            (Kind::Smoothstep, 1580.0, 100.0, 0.0),
            (Kind::Output, 1800.0, 100.0, 0.0),
        ],
        wires: &[
            (0, 0, 2, 0),
            (1, 0, 2, 1),
            (2, 0, 3, 0),
            (3, 0, 5, 0),
            (4, 0, 5, 1),
            (5, 0, 6, 0),
            (6, 0, 7, 0),
            (6, 0, 8, 0),
            (7, 0, 9, 0),
            (8, 0, 9, 1),
            (9, 0, 10, 0),
            (10, 0, 11, 0),
        ],
    },
    Preset {
        name: "Checker",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Value, 40.0, 190.0, 8.0),
            (Kind::Multiply, 260.0, 60.0, 0.0),
            (Kind::Floor, 480.0, 60.0, 0.0),
            (Kind::Across, 700.0, 40.0, 0.0),
            (Kind::Down, 700.0, 170.0, 0.0),
            (Kind::Add, 920.0, 100.0, 0.0),
            (Kind::Value, 920.0, 250.0, 0.5),
            (Kind::Multiply, 1140.0, 120.0, 0.0),
            (Kind::Fract, 1360.0, 120.0, 0.0),
            (Kind::Value, 1360.0, 270.0, 2.0),
            (Kind::Multiply, 1580.0, 140.0, 0.0),
            (Kind::Output, 1800.0, 140.0, 0.0),
        ],
        wires: &[
            (0, 0, 2, 0),
            (1, 0, 2, 1),
            (2, 0, 3, 0),
            (3, 0, 4, 0),
            (3, 0, 5, 0),
            (4, 0, 6, 0),
            (5, 0, 6, 1),
            (6, 0, 8, 0),
            (7, 0, 8, 1),
            (8, 0, 9, 0),
            (9, 0, 11, 0),
            (10, 0, 11, 1),
            (11, 0, 12, 0),
        ],
    },
    Preset {
        name: "Clouds",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Time, 40.0, 190.0, 0.0),
            (Kind::Value, 40.0, 320.0, 0.04),
            (Kind::Multiply, 260.0, 210.0, 0.0),
            (Kind::Add, 480.0, 100.0, 0.0),
            (Kind::Noise, 700.0, 100.0, 1.0),
            (Kind::Smoothstep, 920.0, 100.0, 0.0),
            (Kind::Output, 1140.0, 100.0, 0.0),
        ],
        wires: &[
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (0, 0, 4, 0),
            (3, 0, 4, 1),
            (4, 0, 5, 0),
            (5, 0, 6, 0),
            (6, 0, 7, 0),
        ],
    },
    Preset {
        name: "Picture",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Picture, 260.0, 40.0, 0.0),
            (Kind::Output, 480.0, 40.0, 0.0),
        ],
        wires: &[(0, 0, 1, 0), (1, 0, 2, 0)],
    },
    Preset {
        name: "Picture warp",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Noise, 260.0, 40.0, 1.0),
            (Kind::Value, 260.0, 190.0, 0.18),
            (Kind::Multiply, 480.0, 60.0, 0.0),
            (Kind::Add, 700.0, 60.0, 0.0),
            (Kind::Picture, 920.0, 60.0, 0.0),
            (Kind::Output, 1140.0, 60.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (0, 0, 4, 0),
            (3, 0, 4, 1),
            (4, 0, 5, 0),
            (5, 0, 6, 0),
        ],
    },
    Preset {
        name: "Picture ripple",
        nodes: &[
            (Kind::Middle, 40.0, 40.0, 0.0),
            (Kind::Length, 260.0, 40.0, 0.0),
            (Kind::Value, 260.0, 190.0, 24.0),
            (Kind::Multiply, 480.0, 60.0, 0.0),
            (Kind::Time, 40.0, 320.0, 0.0),
            (Kind::Value, 40.0, 450.0, 3.0),
            (Kind::Multiply, 260.0, 340.0, 0.0),
            (Kind::Subtract, 700.0, 160.0, 0.0),
            (Kind::Sine, 920.0, 160.0, 0.0),
            (Kind::Value, 920.0, 310.0, 0.04),
            (Kind::Multiply, 1140.0, 200.0, 0.0),
            (Kind::Uv, 1140.0, 40.0, 0.0),
            (Kind::Add, 1360.0, 100.0, 0.0),
            (Kind::Picture, 1580.0, 100.0, 0.0),
            (Kind::Output, 1800.0, 100.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (4, 0, 6, 0),
            (5, 0, 6, 1),
            (3, 0, 7, 0),
            (6, 0, 7, 1),
            (7, 0, 8, 0),
            (8, 0, 10, 0),
            (9, 0, 10, 1),
            (11, 0, 12, 0),
            (10, 0, 12, 1),
            (12, 0, 13, 0),
            (13, 0, 14, 0),
        ],
    },
    Preset {
        name: "Picture kaleidoscope",
        nodes: &[
            (Kind::Middle, 40.0, 40.0, 0.0),
            (Kind::Time, 40.0, 190.0, 0.0),
            (Kind::Value, 40.0, 320.0, 0.05),
            (Kind::Multiply, 260.0, 210.0, 0.0),
            (Kind::Rotate, 480.0, 60.0, 0.0),
            (Kind::Polar, 700.0, 60.0, 0.0),
            (Kind::Value, 700.0, 210.0, 5.0),
            (Kind::Multiply, 920.0, 80.0, 0.0),
            (Kind::Fract, 1140.0, 80.0, 0.0),
            (Kind::Picture, 1360.0, 80.0, 0.0),
            (Kind::Output, 1580.0, 80.0, 0.0),
        ],
        wires: &[
            (1, 0, 3, 0),
            (2, 0, 3, 1),
            (0, 0, 4, 0),
            (3, 0, 4, 1),
            (4, 0, 5, 0),
            (5, 0, 7, 0),
            (6, 0, 7, 1),
            (7, 0, 8, 0),
            (8, 0, 9, 0),
            (9, 0, 10, 0),
        ],
    },
    Preset {
        name: "Picture edges",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Picture, 260.0, 40.0, 0.0),
            (Kind::Luminance, 480.0, 40.0, 0.0),
            (Kind::Value, 480.0, 190.0, 0.45),
            (Kind::Step, 700.0, 60.0, 0.0),
            (Kind::Invert, 920.0, 60.0, 0.0),
            (Kind::Palette, 1140.0, 60.0, 0.0),
            (Kind::Output, 1360.0, 60.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (1, 0, 2, 0),
            (2, 0, 4, 0),
            (3, 0, 4, 1),
            (4, 0, 5, 0),
            (5, 0, 6, 0),
            (6, 0, 7, 0),
        ],
    },
    Preset {
        name: "Stone tiles",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Value, 40.0, 190.0, 4.0),
            (Kind::Multiply, 260.0, 60.0, 0.0),
            (Kind::Picture, 480.0, 60.0, 1.0),
            (Kind::Noise, 480.0, 240.0, 1.0),
            (Kind::Palette, 700.0, 240.0, 0.0),
            (Kind::Multiply, 920.0, 120.0, 0.0),
            (Kind::Output, 1140.0, 120.0, 0.0),
        ],
        wires: &[
            (0, 0, 2, 0),
            (1, 0, 2, 1),
            (2, 0, 3, 0),
            (0, 0, 4, 0),
            (4, 0, 5, 0),
            (3, 0, 6, 0),
            (5, 0, 6, 1),
            (6, 0, 7, 0),
        ],
    },
    Preset {
        name: "Materialize",
        nodes: &[
            (Kind::Uv, 40.0, 40.0, 0.0),
            (Kind::Down, 260.0, 40.0, 0.0),
            (Kind::Noise, 260.0, 190.0, 3.6),
            (Kind::Value, 260.0, 340.0, 0.11),
            (Kind::Multiply, 480.0, 210.0, 0.0),
            (Kind::Sweep, 480.0, 40.0, 7.0),
            (Kind::Add, 700.0, 90.0, 0.0),
            (Kind::Subtract, 920.0, 120.0, 0.0),
            (Kind::Value, 920.0, 300.0, 0.0),
            (Kind::Step, 1140.0, 140.0, 0.0),
            (Kind::Seam, 1140.0, 320.0, 0.12),
            (Kind::Lattice, 480.0, 470.0, 24.0),
            (Kind::Palette, 700.0, 470.0, 0.55),
            (Kind::Multiply, 920.0, 490.0, 0.0),
            (Kind::Picture, 700.0, 640.0, 0.0),
            (Kind::Mix, 1360.0, 380.0, 0.0),
            (Kind::Add, 1580.0, 240.0, 0.0),
            (Kind::Output, 1800.0, 240.0, 0.0),
        ],
        wires: &[
            (0, 0, 1, 0),
            (0, 0, 2, 0),
            (2, 0, 4, 0),
            (3, 0, 4, 1),
            (5, 0, 6, 0),
            (4, 0, 6, 1),
            (6, 0, 7, 0),
            (1, 0, 7, 1),
            (7, 0, 9, 0),
            (8, 0, 9, 1),
            (7, 0, 10, 0),
            (0, 0, 11, 0),
            (11, 0, 12, 0),
            (12, 0, 13, 0),
            (11, 0, 13, 1),
            (0, 0, 14, 0),
            (13, 0, 15, 0),
            (14, 0, 15, 1),
            (9, 0, 15, 2),
            (15, 0, 16, 0),
            (10, 0, 16, 1),
            (16, 0, 17, 0),
        ],
    },
];
