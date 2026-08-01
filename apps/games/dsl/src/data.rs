pub mod actions;
pub mod ask;
pub mod cell;
pub mod change;
pub mod edit;
pub mod facility;
pub mod form;
pub mod kept;
pub mod keys;
pub mod lesson;
pub mod plan;

pub const TITLE: &str = "Description Language";
pub const FILES: &[(&str, &str)] = &[
    ("breakout", include_str!("../games/breakout.ns")),
    ("coin_rush", include_str!("../games/coin_rush.ns")),
    (
        "breakout_juiced",
        include_str!("../games/breakout_juiced.ns"),
    ),
    ("dash", include_str!("../games/dash.ns")),
    ("flap", include_str!("../games/flap.ns")),
    ("snake", include_str!("../games/snake.ns")),
    ("helmet", include_str!("../games/helmet.ns")),
    ("lights", include_str!("../games/lights.ns")),
    ("machine", include_str!("../games/machine.ns")),
    ("template", include_str!("../games/template.ns")),
    ("composed", include_str!("../games/composed.ns")),
    ("tumble", include_str!("../games/tumble.ns")),
    ("swarm", include_str!("../games/swarm.ns")),
    ("textured", include_str!("../games/textured.ns")),
    ("sprites", include_str!("../games/sprites.ns")),
    ("decals", include_str!("../games/decals.ns")),
    ("picking", include_str!("../games/picking.ns")),
    ("copies", include_str!("../games/copies.ns")),
    ("paths", include_str!("../games/paths.ns")),
    ("fields", include_str!("../games/fields.ns")),
    ("voxels", include_str!("../games/voxels.ns")),
    ("orbit.ns", include_str!("../games/orbit.ns")),
    ("arena.ns", include_str!("../games/arena.ns")),
    ("camera.ns", include_str!("../games/camera.ns")),
    ("daylight.ns", include_str!("../games/daylight.ns")),
    ("turns.ns", include_str!("../games/turns.ns")),
    ("readout.ns", include_str!("../games/readout.ns")),
];
pub const DEFAULT_VIEW_HEIGHT: f32 = 13.5;
pub const VIEW_HEIGHT: &str = "viewHeight";
pub const CLEAR_COLOR: [f32; 4] = [0.006, 0.007, 0.013, 1.0];
pub const TEXTURE_BINDING: &str = "Texture";
pub const REPEAT_COLUMN: &str = "Repeat";
pub const TILE_TEXTURE: &str = "tile";
pub const GRID_TEXTURE: &str = "grid";
pub const SPOT_TEXTURE: &str = "spot";
pub const SKIN_SIDE: u32 = 64;
pub const DECAL_BINDING: &str = "Decal";
pub const FIELD_COLUMN: &str = "Field";
pub const CARVE_COLUMN: &str = "Carve";
pub const BLEND_COLUMN: &str = "Blend";
pub const GROUND_COLUMN: &str = "Ground";
pub const PATH_COLUMN: &str = "Path";
pub const PACE_COLUMN: &str = "Pace";
pub const LAND_RESOURCE: &str = "Land";
pub const BLOCK_COLUMN: &str = "Block";
pub const DIG_COLUMN: &str = "Dig";
pub const LAND_TEXTURE: &str = "land";
pub const LAND_SOLID_MATERIAL: &str = "land solid";
pub const LAND_CUTOUT_MATERIAL: &str = "land cutout";
pub const LAND_CLEAR_MATERIAL: &str = "land clear";
pub const LAND_KINDS: &[&str] = &["rock", "earth", "grass", "sand"];
pub const LAND_CELLS: [u32; 2] = [4, 1];
pub const LAND_CELL_SIDE: u32 = 32;
pub const LAND_REACH: i32 = 8;
pub const LAND_FARTHEST: i32 = 12;
pub const LAND_HILLS: f32 = 7.0;
pub const LAND_SHORE: f32 = 0.0;
pub const LAND_FLOOR: i32 = -4;
pub const LAND_SPAN: i32 = 64;
pub const LAND_SIDE: i32 = 16;
pub const LAND_DEPTH: i32 = 3;
pub const LAND_SKIN: i32 = 3;
pub const LAND_GRAIN: f32 = 0.03;

pub const AGENT_RADIUS: f32 = 0.4;
pub const AGENT_HEIGHT: f32 = 1.6;
pub const AGENT_PACE: f32 = 3.0;
pub const AGENT_STEP: f32 = 0.25;
pub const PICK_COLUMN: &str = "Pick";
pub const PICKED_COLUMN: &str = "Picked";
pub const COPIES_COLUMN: &str = "Copies";
pub const SPREAD_COLUMN: &str = "Spread";
pub const SPRITE_BINDING: &str = "Sprite";
pub const TILE_COLUMN: &str = "Tile";
pub const SHEET_ATLAS: &str = "sheet";
pub const SHEET_ATLAS_SIDE: u32 = 512;
pub const SHEET_SIDE: u32 = 32;
pub const SHEET_ACROSS: u32 = 4;
pub const SHEET_TILES: &str = "tiles";
pub const SHEET_FACE: &str = "face";
pub const SUN_TONE: [f32; 3] = [1.0, 0.95, 0.8];
pub const SUN_STRENGTH: f32 = 5.0;
pub const SUN_PITCH: f32 = -std::f32::consts::FRAC_PI_4;
pub const CAMERA_DISTANCE: f32 = 30.0;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Word(String),
    Number(f32),
    Text(String),
    Symbol(String),
    Begin,
    End,
    Newline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Range,
}

#[derive(Clone, Debug)]
pub enum Expression {
    Number(f32),
    Text(String),
    Truth(bool),
    Name(String),
    Field {
        of: Box<Expression>,
        name: String,
    },
    Index {
        of: Box<Expression>,
        at: Box<Expression>,
    },
    Call {
        name: String,
        arguments: Vec<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        value: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Tuple(Vec<Expression>),
    List(Vec<Expression>),
    Record(Vec<(String, Expression)>),
    IsSome(Box<Expression>),
}

#[derive(Clone, Debug)]
pub struct Place {
    pub name: String,
    pub subject: Option<Expression>,
    pub field: Option<String>,
    pub index: Option<Expression>,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
    },
    Assign {
        place: Place,
        value: Expression,
    },
    Add {
        place: Place,
        value: Expression,
    },
    Increase {
        place: Place,
        value: Expression,
    },
    Spawn {
        bindings: Vec<(String, Expression)>,
    },
    Remove {
        subject: Expression,
    },
    When {
        condition: Expression,
        then: Vec<Statement>,
        otherwise: Vec<Statement>,
    },
    ForEach {
        name: String,
        over: Expression,
        body: Vec<Statement>,
    },
    Emit {
        name: String,
        value: Expression,
    },
    Send {
        name: String,
        value: Expression,
    },
    Break,
    Stop,
    Keep {
        name: String,
    },
    Recall {
        name: String,
    },
}

#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
    pub written: bool,
    pub start: Option<Expression>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cardinality {
    Every,
    One,
    Set,
}

#[derive(Clone, Debug)]
pub struct Selection {
    pub label: String,
    pub cardinality: Cardinality,
    pub loose: bool,
    pub columns: Vec<Column>,
    pub requires: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum Port {
    Reads(Vec<String>),
    Writes(Vec<String>),
    Select(Selection),
    Enable(Expression),
    Once,
    Creates,
    Removes,
    Phase(String),
    Needs(Vec<String>),
    Provides(Vec<String>),
    Pure,
}

#[derive(Clone, Debug)]
pub struct System {
    pub name: String,
    pub module: String,
    pub ports: Vec<Port>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct Asset {
    pub label: String,
    pub kind: String,
    pub file: String,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub kind: Option<String>,
    pub default: Option<Expression>,
}

#[derive(Clone, Debug)]
pub enum Declaration {
    Component {
        name: String,
        kind: Option<String>,
    },
    Resource {
        name: String,
        fields: Vec<Field>,
    },
    Config {
        name: String,
        fields: Vec<Field>,
    },
    Event {
        name: String,
        fields: Vec<Field>,
    },
    Command {
        name: String,
        fields: Vec<Field>,
    },
    Action {
        name: String,
        key: String,
        held: bool,
    },
    Query {
        name: String,
        parameters: Vec<String>,
        body: Expression,
    },
    System(System),
    Order {
        earlier: String,
        later: String,
    },
    Kind {
        variants: Vec<String>,
    },
    Use {
        file: String,
        sets: Vec<(String, Expression)>,
        runs: bool,
    },
    Module {
        name: String,
    },
    Pulse {
        name: String,
    },
    Assets {
        name: String,
        source: String,
        root: String,
        rows: Vec<Asset>,
    },
    Shows {
        names: Vec<String>,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub module: String,
    pub owner: Vec<(String, String)>,
    pub visible: Vec<(String, String)>,
    pub shows: Vec<String>,
    pub shown: Vec<(String, String)>,
    pub components: Vec<String>,
    pub shapes: Vec<(String, String)>,
    pub resources: Vec<(String, Vec<Field>)>,
    pub configs: Vec<(String, Vec<Field>)>,
    pub events: Vec<String>,
    pub commands: Vec<String>,
    pub payloads: Vec<(String, Vec<String>)>,
    pub actions: Vec<(String, String, bool)>,
    pub queries: Vec<(String, Vec<String>, Expression)>,
    pub systems: Vec<System>,
    pub orders: Vec<(String, String)>,
    pub variants: Vec<String>,
    pub uses: Vec<String>,
    pub runs: Vec<String>,
    pub sets: Vec<(String, String, Expression)>,
    pub pulses: Vec<String>,
    pub assets: Vec<(String, String, String, Vec<Asset>)>,
    pub turn: Vec<usize>,
}

pub const MESH_BINDING: &str = "Mesh";
pub const MODEL_BINDING: &str = "Model";
pub const SOUND_BINDING: &str = "Sound";
pub const BURST_BINDING: &str = "Burst";
pub const BODY_COLUMN: &str = "Body";
pub const BOUNCE_COLUMN: &str = "Bounce";
pub const BURST_LIFE: f32 = 3.0;
pub const SPARK_SPRITE: u32 = 256;
pub const VOLUME_COLUMN: &str = "Volume";
pub const SOUND_LIFE: f32 = 12.0;
pub const COLOR_BINDING: &str = "Color";
pub const SCALE_BINDING: &str = "Scale";
pub const TRANSFORM_COLUMN: &str = "Transform";
pub const ROTATION_COLUMN: &str = "Rotation";
pub const PARENT_COLUMN: &str = "Parent";
pub const VISIBLE_COLUMN: &str = "Visible";
pub const LIFETIME_COLUMN: &str = "Lifetime";
pub const LABEL_COLUMN: &str = "Label";
pub const SIZE_COLUMN: &str = "Size";
pub const ANCHOR_COLUMN: &str = "Anchor";
pub const JUSTIFY_COLUMN: &str = "Justify";
pub const PANEL_COLUMN: &str = "Panel";
pub const BUTTON_COLUMN: &str = "Button";
pub const CLICKED_COLUMN: &str = "Clicked";
pub const INK_COLUMN: &str = "Ink";
pub const LIGHT_COLUMN: &str = "Light";
pub const STRENGTH_COLUMN: &str = "Strength";
pub const REACH_COLUMN: &str = "Reach";

pub const TIME_RESOURCE: &str = "Time";
pub const INPUT_RESOURCE: &str = "Input";
pub const WINDOW_RESOURCE: &str = "Window";
pub const CAMERA_RESOURCE: &str = "Camera";
pub const RANDOM_RESOURCE: &str = "Random";
pub const FLOW_RESOURCE: &str = "Flow";
pub const SHAKE_RESOURCE: &str = "Shake";
pub const COST_RESOURCE: &str = "Cost";
pub const SENSED: &[&str] = &[
    TIME_RESOURCE,
    INPUT_RESOURCE,
    RANDOM_RESOURCE,
    FLOW_RESOURCE,
    SHAKE_RESOURCE,
    COST_RESOURCE,
];
pub const SUN_RESOURCE: &str = "Sun";
pub const SKY_RESOURCE: &str = "Sky";

pub const CAMERA_EYE: &str = "cameraEye";
pub const LONGEST_FRAME: f32 = 1.0 / 30.0;
pub const DEFAULT_TEXT_SIZE: f32 = 20.0;
pub const SIGN_LINE: f32 = 1.4;
pub const DEFAULT_LIGHT_STRENGTH: f32 = 60.0;
pub const DEFAULT_LIGHT_REACH: f32 = 25.0;

pub const PANEL_WIDTH: f32 = 210.0;
pub const STUDIO_WIDTH: f32 = 520.0;
pub const TAPE_LENGTH: usize = 900;
pub const TRANSPORT_HEIGHT: f32 = 108.0;
pub const BUTTON_WIDTH: f32 = 38.0;
pub const ICON_SIZE: f32 = 17.0;
pub const CHIP_SIZE: f32 = 12.0;
pub const SMALL_SIZE: f32 = 11.0;
pub const LINE_WIDTH: usize = 52;
pub const BUTTON_HEIGHT: f32 = 26.0;
pub const GROUP_LIMIT: usize = 5;
pub const TAB_HEIGHT: f32 = 30.0;
pub const LABEL_WIDTH: f32 = 96.0;
pub const PICK_WIDTH: f32 = 150.0;
pub const PANEL_PADDING: f32 = 12.0;
pub const PANEL_SPACING: f32 = 8.0;

pub fn joined(root: &str, file: &str) -> String {
    if root.is_empty() {
        return file.to_string();
    }
    format!("{}/{file}", root.trim_end_matches('/'))
}

pub fn games() -> Vec<&'static str> {
    FILES
        .iter()
        .filter(|(name, _)| !name.ends_with(".ns"))
        .map(|(name, _)| *name)
        .collect()
}
