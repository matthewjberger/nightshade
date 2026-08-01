use nightshade::prelude::*;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceKind {
    #[default]
    Street,
    Facade,
    FacadeUpper,
    Cornice,
    Partition,
    RoomFloor,
    Ceiling,
    Door,
    Crate,
    Metal,
    Rust,
    Wood,
    Concrete,
    Grille,
    Glass,
    GlassLit,
}

impl SurfaceKind {
    pub const ALL: [SurfaceKind; 16] = [
        SurfaceKind::Street,
        SurfaceKind::Facade,
        SurfaceKind::FacadeUpper,
        SurfaceKind::Cornice,
        SurfaceKind::Partition,
        SurfaceKind::RoomFloor,
        SurfaceKind::Ceiling,
        SurfaceKind::Door,
        SurfaceKind::Crate,
        SurfaceKind::Metal,
        SurfaceKind::Rust,
        SurfaceKind::Wood,
        SurfaceKind::Concrete,
        SurfaceKind::Grille,
        SurfaceKind::Glass,
        SurfaceKind::GlassLit,
    ];
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    #[default]
    BrassKey,
    WardKey,
    BoneKey,
    HealthDrink,
    Ammo,
    Battery,
    TownMap,
    Lockbox,
    Case,
    BoneHaft,
    BoneWard,
    Note(usize),
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HuskKind {
    #[default]
    Wanderer,
    Crawler,
    Wretch,
    Screamer,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HuskState {
    #[default]
    Slumped,
    Rising,
    Dormant,
    Investigate,
    Chase,
    Stagger,
    Downed,
    Dead,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lock(pub Option<ItemKind>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tile(pub (i32, i32));

#[derive(Default, Clone, Copy, Debug)]
pub struct Bolted(pub bool);

#[derive(Default, Clone, Debug)]
pub struct Inscription {
    pub index: usize,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Health(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Timer(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Repath(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Scream(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct NextTile(pub Option<(i32, i32)>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Interest(pub Option<Vec3>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stride(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Floor(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Moving(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bled(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Glow(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Opens(pub bool);

#[derive(Default, Clone, Debug)]
pub struct Explosive {
    pub fuse: Option<f32>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct RoomIndex(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lit(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Col(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Row(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Upper(pub bool);

#[derive(Default, Clone, Debug)]
pub struct Heft {
    pub mass: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct NpcIndex(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Alive(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Life(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Armed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Last(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pace(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Body(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rest(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Swing(pub f32);
