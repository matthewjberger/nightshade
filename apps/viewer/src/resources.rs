use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use nightshade::prelude::Entity;
use nightshade_cli::clap;

#[cfg(not(target_arch = "wasm32"))]
use crate::data::LibraryKind;
use crate::data::{
    BrowserItem, BrowserTab, GizmoHandle, GizmoMode, MaterialControl, PanelTab, PickKind, RowKind,
    TextureSlot, ViewerAction,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialView {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_mode: usize,
    pub alpha_cutoff: f32,
    pub unlit: bool,
    pub double_sided: bool,
    pub emissive: [f32; 3],
    pub emissive_strength: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub ior: f32,
    pub specular: f32,
    pub transmission: f32,
    pub thickness: f32,
    pub dispersion: f32,
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
    pub sheen_roughness: f32,
    pub iridescence: f32,
    pub iridescence_ior: f32,
    pub textures: Vec<(TextureSlot, String)>,
}

pub struct TextureDrop {
    pub slot: TextureSlot,
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightView {
    pub kind: usize,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub inner_cone: f32,
    pub outer_cone: f32,
    pub cast_shadows: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraView {
    pub perspective: bool,
    pub field_of_view: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Default)]
pub struct CameraInput {
    pub frame_requested: bool,
    pub turntable: bool,
    pub viewer_camera: Option<Entity>,
}

pub struct Sun {
    pub entity: Option<Entity>,
    pub intensity: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub shadows: bool,
}

impl Default for Sun {
    fn default() -> Self {
        Self {
            entity: None,
            intensity: 3.5,
            yaw: 45.0,
            pitch: -30.0,
            shadows: true,
        }
    }
}

pub struct Selection {
    pub selected: Option<Entity>,
    pub others: Vec<Entity>,
    pub name: String,
    pub mesh: Option<String>,
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
    pub rotation_quat: nalgebra_glm::Quat,
    pub scale: [f32; 3],
    pub visible: bool,
    pub material: Option<String>,
    pub material_values: MaterialView,
    pub material_loaded: MaterialView,
    pub light: Option<LightView>,
    pub camera: Option<CameraView>,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            selected: None,
            others: Vec::new(),
            name: String::new(),
            mesh: None,
            translation: [0.0; 3],
            rotation: [0.0; 3],
            rotation_quat: nalgebra_glm::Quat::identity(),
            scale: [1.0; 3],
            visible: true,
            material: None,
            material_values: MaterialView::default(),
            material_loaded: MaterialView::default(),
            light: None,
            camera: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SceneRow {
    pub entity: Entity,
    pub parent: Option<Entity>,
    pub name: String,
    pub depth: u32,
    pub kind: RowKind,
    pub visible: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ModelStats {
    pub meshes: u32,
    pub vertices: u32,
    pub triangles: u32,
    pub materials: u32,
    pub textures: u32,
    pub dimensions: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct ClipInfo {
    pub name: String,
    pub duration: f32,
}

#[derive(Clone, Debug)]
pub struct MorphInfo {
    pub entity: Entity,
    pub name: String,
    pub weights: Vec<f32>,
}

pub struct LoadReport {
    pub stats: ModelStats,
    pub delay: u32,
}

#[derive(Default)]
pub struct Model {
    pub pending: Option<nightshade_load::LoadHandle>,
    pub roots: Vec<Entity>,
    pub entities: Vec<Entity>,
    pub rows: Vec<SceneRow>,
    pub stats: Option<ModelStats>,
    pub clips: Vec<ClipInfo>,
    pub variants: Vec<String>,
    pub morphs: Vec<MorphInfo>,
    pub cameras: Vec<(Entity, String)>,
    pub report: Option<LoadReport>,
    pub isolated: Option<Entity>,
    pub held_clips: Option<nightshade_animation::contract::ClipSet>,
    pub previews_decoded: Arc<Mutex<Vec<ThumbnailImage>>>,
    pub previews: HashMap<String, ThumbnailSlot>,
    pub previews_stale: bool,
    pub dropped_textures: Vec<String>,
}

#[derive(Clone)]
pub struct DragStart {
    pub entity: Entity,
    pub transform: TransformSnapshot,
    pub position: nalgebra_glm::Vec3,
    pub frame: nalgebra_glm::Mat4,
}

#[derive(Clone)]
pub struct GizmoDrag {
    pub handle: GizmoHandle,
    pub normal: nalgebra_glm::Vec3,
    pub start_point: nalgebra_glm::Vec3,
    pub origin: nalgebra_glm::Vec3,
    pub starts: Vec<DragStart>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformSnapshot {
    pub translation: nalgebra_glm::Vec3,
    pub rotation: nalgebra_glm::Quat,
    pub scale: nalgebra_glm::Vec3,
}

pub struct Gizmo {
    pub enabled: bool,
    pub mode: GizmoMode,
    pub entity: Option<Entity>,
    pub hovered: Option<GizmoHandle>,
    pub drag: Option<GizmoDrag>,
}

impl Default for Gizmo {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: GizmoMode::Move,
            entity: None,
            hovered: None,
            drag: None,
        }
    }
}

#[derive(Default)]
pub struct Skeleton {
    pub enabled: bool,
    pub entity: Option<Entity>,
}

#[derive(Default)]
pub struct Picking {
    pub outstanding: Option<PickKind>,
    pub waited: u32,
    pub click_wanted: Option<nalgebra_glm::Vec2>,
    pub click_adds: bool,
    pub last_leaf: Option<Entity>,
    pub last_root: Option<Entity>,
    pub cycle_depth: usize,
    pub last_press: Option<nalgebra_glm::Vec2>,
    pub hover_position: Option<nalgebra_glm::Vec2>,
    pub hovered: Option<Entity>,
    pub hover_lines: Option<Entity>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformEdit {
    pub entity: Entity,
    pub before: TransformSnapshot,
    pub after: TransformSnapshot,
}

#[derive(Clone, Debug)]
pub enum Edit {
    Transforms(Vec<TransformEdit>),
    Delete(Vec<Entity>),
    Material {
        name: String,
        before: MaterialView,
        after: MaterialView,
    },
    Light {
        entity: Entity,
        before: LightView,
        after: LightView,
    },
    Camera {
        entity: Entity,
        before: CameraView,
        after: CameraView,
    },
}

#[derive(Default)]
pub struct History {
    pub done: Vec<(Edit, u64)>,
    pub undone: Vec<Edit>,
    pub frame: u64,
}

pub type AssetSlot = Arc<Mutex<Option<PendingAsset>>>;

pub type LoadingSlot = Arc<Mutex<Option<String>>>;

pub type NoticeSlot = Arc<Mutex<Option<String>>>;

pub enum PendingAsset {
    Model(Vec<u8>),
    ModelWithResources {
        gltf: Vec<u8>,
        resources: HashMap<String, Vec<u8>>,
    },
    Sky(Vec<u8>),
}

#[derive(Clone)]
pub struct Crew {
    pub hands: Arc<Mutex<Vec<std::thread::JoinHandle<()>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub agent: Arc<Mutex<Option<Arc<tiny_http::Server>>>>,
}

impl Default for Crew {
    fn default() -> Self {
        Self {
            hands: Arc::new(Mutex::new(Vec::new())),
            #[cfg(not(target_arch = "wasm32"))]
            agent: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Clone)]
pub struct Incoming {
    pub asset: Arc<Mutex<Option<PendingAsset>>>,
    pub loading: Arc<Mutex<Option<String>>>,
    pub notice: Arc<Mutex<Option<String>>>,
    pub texture: Arc<Mutex<Option<TextureDrop>>>,
    pub crew: Crew,
}

impl Default for Incoming {
    fn default() -> Self {
        Self {
            asset: Arc::new(Mutex::new(None)),
            loading: Arc::new(Mutex::new(None)),
            notice: Arc::new(Mutex::new(None)),
            texture: Arc::new(Mutex::new(None)),
            crew: Crew::default(),
        }
    }
}

#[derive(Default)]
pub struct Dropped {
    pub files: Vec<(String, Vec<u8>)>,
    pub settle: u32,
}

#[derive(Clone)]
pub struct KhronosAsset {
    pub name: String,
    pub label: String,
    pub model_url: String,
    pub binary: bool,
    pub thumbnail: Option<String>,
}

#[derive(Clone)]
pub struct PolyAsset {
    pub slug: String,
    pub name: String,
    pub thumbnail: String,
}

#[derive(Default)]
pub enum FetchState<T> {
    #[default]
    Idle,
    Loading,
    Loaded(T),
    Failed,
}

impl<T> FetchState<T> {
    pub fn loaded(&self) -> Option<&T> {
        match self {
            FetchState::Loaded(value) => Some(value),
            _ => None,
        }
    }
}

pub struct ThumbnailImage {
    pub key: String,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy)]
pub struct ThumbnailSlot {
    pub layer: u32,
    pub uv_min: nalgebra_glm::Vec2,
    pub uv_max: nalgebra_glm::Vec2,
}

pub struct Browsers {
    pub khronos: Arc<Mutex<FetchState<Vec<KhronosAsset>>>>,
    pub skies: Arc<Mutex<FetchState<Vec<PolyAsset>>>>,
    pub models: Arc<Mutex<FetchState<Vec<PolyAsset>>>>,
    pub decoded: Arc<Mutex<Vec<ThumbnailImage>>>,
    pub requested: HashSet<String>,
    pub slots: HashMap<String, ThumbnailSlot>,
    pub wanted: Vec<String>,
    pub started: bool,
    pub announced: [bool; 3],
    pub reset_thumbnails: bool,
}

impl Default for Browsers {
    fn default() -> Self {
        Self {
            khronos: Arc::new(Mutex::new(FetchState::Idle)),
            skies: Arc::new(Mutex::new(FetchState::Idle)),
            models: Arc::new(Mutex::new(FetchState::Idle)),
            decoded: Arc::new(Mutex::new(Vec::new())),
            requested: HashSet::new(),
            slots: HashMap::new(),
            wanted: Vec::new(),
            started: false,
            announced: [false; 3],
            reset_thumbnails: false,
        }
    }
}

pub struct BrowserEntry {
    pub label: String,
    pub key: String,
    pub item: BrowserItem,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryEntry {
    pub path: String,
    pub name: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub kind: LibraryKind,
    pub thumbnail: Option<String>,
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub struct PreviewRun {
    pub child: std::process::Child,
    pub key: String,
    pub file: std::path::PathBuf,
    pub started: nightshade::prelude::Instant,
}

#[derive(Default)]
pub struct Library {
    pub built_in: Option<String>,
    pub folders: Vec<String>,
    pub files: Vec<String>,
    pub entries: Vec<LibraryEntry>,
    pub scanned: Arc<Mutex<Option<Vec<LibraryEntry>>>>,
    pub sources: HashMap<String, String>,
    #[cfg(not(target_arch = "wasm32"))]
    pub jobs: Vec<String>,
    #[cfg(not(target_arch = "wasm32"))]
    pub drawn: HashSet<String>,
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    pub running: Vec<PreviewRun>,
    pub scanning: bool,
    pub watch: u32,
    pub dirty: bool,
}

pub struct ViewState {
    pub scene_open: bool,
    pub inspector_open: bool,
    pub add_open: bool,
    pub browser: Option<BrowserTab>,
    pub browser_page: usize,
    pub panel_tab: PanelTab,
    pub interface_visible: bool,
    pub search: String,
    pub tree_filter: String,
    pub tree_filter_dirty: bool,
    pub detail_dirty: bool,
    pub layers_dirty: bool,
    pub resolution: u32,
    pub grid: bool,
    pub snap: bool,
    pub shading: usize,
    pub pbr_debug: usize,
    pub tonemap: usize,
    pub exposure: f32,
    pub show_normals: bool,
    pub show_bounds: bool,
    pub show_sky: bool,
    pub atmosphere: usize,
    pub hdr_loaded: bool,
    pub ssao: bool,
    pub bloom: bool,
    pub ssr: bool,
    pub ssgi: bool,
    pub anti_aliasing: usize,
    pub render_scale: f32,
    pub fog: bool,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 3],
    pub depth_of_field: bool,
    pub focus_distance: f32,
    pub focus_range: f32,
    pub recent_open: bool,
    pub recent_dirty: bool,
    pub active_variant: Option<String>,
    pub tree_dirty: bool,
    pub tree_visibility_dirty: bool,
    pub browser_dirty: bool,
    pub inspector_dirty: bool,
    pub render_panel_dirty: bool,
    pub applied: AppliedUi,
}

#[derive(Default)]
pub struct AppliedUi {
    pub interface_visible: Option<bool>,
    pub show_button: Option<bool>,
    pub scene_open: Option<bool>,
    pub inspector_open: Option<bool>,
    pub inspector_body: Option<bool>,
    pub add_open: Option<bool>,
    pub recent_open: Option<bool>,
    pub browser_open: Option<bool>,
    pub browser_page: Option<(usize, usize)>,
    pub animation_visible: Option<bool>,
    pub layers_visible: Option<bool>,
    pub layer_count: Option<usize>,
    pub clip: Option<Option<usize>>,
    pub blend_time: Option<f32>,
    pub panel_tab: Option<usize>,
    pub selected: Option<Option<Entity>>,
    pub selected_visible: Option<bool>,
    pub isolated: Option<bool>,
    pub loading: Option<Option<String>>,
    pub grid: Option<bool>,
    pub snap: Option<bool>,
    pub turntable: Option<bool>,
    pub browser_tab: Option<usize>,
    pub library_strip: Option<bool>,
    pub gizmo_shifted: Option<bool>,
    pub gizmo_mode: Option<Option<usize>>,
    pub playing: Option<bool>,
    pub looping: Option<bool>,
    pub fps: Option<u32>,
    pub time: Option<(f32, f32)>,
    pub stats: Option<[String; 6]>,
    pub notice: Option<Option<String>>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            scene_open: true,
            inspector_open: true,
            add_open: false,
            browser: None,
            browser_page: 0,
            panel_tab: PanelTab::Scene,
            interface_visible: true,
            search: String::new(),
            tree_filter: String::new(),
            tree_filter_dirty: false,
            detail_dirty: false,
            layers_dirty: false,
            resolution: 2,
            grid: true,
            snap: false,
            shading: 0,
            pbr_debug: 0,
            tonemap: 0,
            exposure: 1.0,
            show_normals: false,
            show_bounds: false,
            show_sky: true,
            atmosphere: 1,
            hdr_loaded: false,
            ssao: true,
            bloom: true,
            ssr: false,
            ssgi: false,
            anti_aliasing: 1,
            render_scale: 1.0,
            fog: false,
            fog_start: 2.0,
            fog_end: 15.0,
            fog_color: [0.5, 0.5, 0.55],
            depth_of_field: false,
            focus_distance: 10.0,
            focus_range: 5.0,
            recent_open: false,
            recent_dirty: true,
            active_variant: None,
            tree_dirty: false,
            tree_visibility_dirty: false,
            browser_dirty: false,
            inspector_dirty: false,
            render_panel_dirty: false,
            applied: AppliedUi::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerView {
    pub clip: usize,
    pub name: String,
    pub weight: f32,
}

pub struct AnimationView {
    pub clip: Option<usize>,
    pub playing: bool,
    pub time: f32,
    pub duration: f32,
    pub speed: f32,
    pub looping: bool,
    pub blend_time: f32,
    pub blending: bool,
    pub layers: Vec<LayerView>,
}

impl Default for AnimationView {
    fn default() -> Self {
        Self {
            clip: None,
            playing: false,
            time: 0.0,
            duration: 0.0,
            speed: 1.0,
            looping: true,
            blend_time: 0.25,
            blending: false,
            layers: Vec::new(),
        }
    }
}

#[derive(clap::Args, Default)]
pub struct ViewerArguments {
    #[arg(
        long,
        value_name = "PATH",
        help = "A model to open at start, a path here or a web address on the web"
    )]
    pub model: Option<String>,
    #[arg(
        long,
        value_name = "PATH",
        help = "A sky to open at start, a path here or a web address on the web"
    )]
    pub sky: Option<String>,
    #[arg(
        long,
        value_name = "PORT",
        help = "The port the agent endpoint listens on"
    )]
    pub agent_port: Option<u16>,
    #[arg(long, help = "Leave the agent port closed")]
    pub no_agent: bool,
    #[arg(
        long,
        help = "Draw one model with no window and no interface, for a library preview"
    )]
    pub preview: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    pub scene_open: bool,
    pub inspector_open: bool,
    pub panel_tab: usize,
    pub resolution: u32,
    pub grid: bool,
    pub snap: bool,
    pub turntable: bool,
    pub shading: usize,
    pub pbr_debug: usize,
    pub tonemap: usize,
    pub exposure: f32,
    pub atmosphere: usize,
    pub show_sky: bool,
    pub show_normals: bool,
    pub show_bounds: bool,
    pub show_skeleton: bool,
    pub ssao: bool,
    pub bloom: bool,
    pub ssr: bool,
    pub ssgi: bool,
    pub anti_aliasing: usize,
    pub render_scale: f32,
    pub fog: bool,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 3],
    pub depth_of_field: bool,
    pub focus_distance: f32,
    pub focus_range: f32,
    pub recent: Vec<String>,
    pub folders: Vec<String>,
    pub assets: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scene_open: true,
            inspector_open: true,
            panel_tab: 0,
            resolution: 2,
            grid: true,
            snap: false,
            turntable: false,
            shading: 0,
            pbr_debug: 0,
            tonemap: 0,
            exposure: 1.0,
            atmosphere: 1,
            show_sky: true,
            show_normals: false,
            show_bounds: false,
            show_skeleton: false,
            ssao: true,
            bloom: true,
            ssr: false,
            ssgi: false,
            anti_aliasing: 1,
            render_scale: 1.0,
            fog: false,
            fog_start: 2.0,
            fog_end: 15.0,
            fog_color: [0.5, 0.5, 0.55],
            depth_of_field: false,
            focus_distance: 10.0,
            focus_range: 5.0,
            recent: Vec::new(),
            folders: Vec::new(),
            assets: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Layout {
    pub narrow: bool,
    pub top_bar_height: f32,
    pub panel_width: f32,
    pub animation_bar_width: f32,
}

impl Default for Layout {
    fn default() -> Self {
        crate::queries::layout::layout_of(1280.0)
    }
}

#[derive(Default)]
pub struct UiHandles {
    pub interface: Entity,
    pub show_button: Entity,
    pub fps_label: Entity,

    pub scene_button: Entity,
    pub inspect_button: Entity,
    pub add_button: Entity,
    pub grid_button: Entity,
    pub spin_button: Entity,
    pub frame_button: Entity,
    pub shot_button: Entity,
    pub recent_button: Entity,
    pub undo_button: Entity,
    pub redo_button: Entity,
    pub gizmo_buttons: [Entity; 3],
    pub snap_button: Entity,
    pub browse_button: Entity,
    pub random_model_button: Entity,
    pub random_sky_button: Entity,

    pub left_panel: Entity,
    pub tab_buttons: [Entity; 3],
    pub tab_roots: [Entity; 3],
    pub tree_view: Entity,
    pub tree_container: Entity,
    pub tree_search: Entity,
    pub tree_empty: Entity,
    pub tree_rows: Vec<TreeRow>,

    pub shading_dropdown: Entity,
    pub pbr_dropdown: Entity,
    pub normals_toggle: Entity,
    pub bounds_toggle: Entity,
    pub skeleton_toggle: Entity,
    pub atmosphere_dropdown: Entity,
    pub sky_toggle: Entity,
    pub exposure_slider: Entity,
    pub tonemap_dropdown: Entity,
    pub sun_intensity: Entity,
    pub sun_yaw: Entity,
    pub sun_pitch: Entity,
    pub sun_shadows: Entity,
    pub ssao_toggle: Entity,
    pub bloom_toggle: Entity,
    pub ssr_toggle: Entity,
    pub ssgi_toggle: Entity,
    pub anti_aliasing_dropdown: Entity,
    pub render_scale_slider: Entity,
    pub fog_toggle: Entity,
    pub fog_start: Entity,
    pub fog_end: Entity,
    pub fog_color: Entity,
    pub depth_of_field_toggle: Entity,
    pub focus_distance: Entity,
    pub focus_range: Entity,
    pub camera_container: Entity,
    pub camera_dropdown: Option<Entity>,
    pub camera_children: Vec<Entity>,
    pub variant_container: Entity,
    pub variant_dropdown: Option<Entity>,
    pub variant_children: Vec<Entity>,
    pub morph_container: Entity,
    pub morph_children: Vec<Entity>,
    pub morph_sliders: Vec<MorphSlider>,

    pub stat_values: [Entity; 6],

    pub inspector_panel: Entity,
    pub inspector_name: Entity,
    pub inspector_mesh: Entity,
    pub inspector_body: Entity,
    pub inspector_empty: Entity,
    pub visible_button: Entity,
    pub isolate_button: Entity,
    pub translation: [Entity; 3],
    pub rotation: [Entity; 3],
    pub scale: [Entity; 3],
    pub light_container: Entity,
    pub light: LightHandles,
    pub camera_section: Entity,
    pub camera_fields: CameraHandles,
    pub material_container: Entity,
    pub material: MaterialHandles,

    pub animation_bar: Entity,
    pub play_button: Entity,
    pub stop_button: Entity,
    pub loop_button: Entity,
    pub seek_slider: Entity,
    pub speed_drag: Entity,
    pub time_label: Entity,
    pub blend_drag: Entity,
    pub add_layer_button: Entity,
    pub layer_panel: Entity,
    pub layer_container: Entity,
    pub layer_children: Vec<Entity>,
    pub layer_rows: Vec<LayerRow>,
    pub clip_container: Entity,
    pub clip_dropdown: Option<Entity>,
    pub clip_children: Vec<Entity>,

    pub browser_panel: Entity,
    pub browser_tabs: [Entity; 4],
    pub browser_search: Entity,
    pub browser_resolution: Entity,
    pub browser_previous: Entity,
    pub browser_next: Entity,
    pub browser_page_label: Entity,
    pub browser_close: Entity,
    pub browser_refresh: Entity,
    pub browser_grid: Entity,
    pub browser_children: Vec<Entity>,
    pub browser_cells: Vec<BrowserCell>,

    pub library_strip: Entity,
    pub library_add_folder: Entity,
    pub library_add_files: Entity,
    pub library_save: Entity,
    pub library_chips: Entity,
    pub library_children: Vec<Entity>,
    pub library_forget: Vec<(Entity, String)>,

    pub recent_panel: Entity,
    pub recent_close: Entity,
    pub recent_container: Entity,
    pub recent_children: Vec<Entity>,
    pub recent_buttons: Vec<(Entity, String)>,

    pub add_panel: Entity,
    pub add_close: Entity,
    pub add_import: Entity,
    pub save_button: Entity,
    pub clips_button: Entity,
    pub primitive_buttons: [Entity; 6],
    pub light_buttons: [Entity; 3],

    pub nav_holder: Entity,
    pub nav_gizmo: Entity,
    pub loading_label: Entity,

    pub notice_dialog: Entity,
    pub notice_backdrop: Entity,
    pub notice_text: Entity,
}

#[derive(Default)]
pub struct MaterialHandles {
    pub children: Vec<Entity>,
    pub base_color: Entity,
    pub emissive: Entity,
    pub alpha_mode: Entity,
    pub unlit: Entity,
    pub double_sided: Entity,
    pub reset: Entity,
    pub sliders: Vec<(Entity, MaterialControl)>,
    pub texture_rows: Vec<TextureRow>,
}

#[derive(Debug)]
pub struct TextureRow {
    pub slot: TextureSlot,
    pub nodes: Vec<Entity>,
    pub replace: Entity,
}

#[derive(Default)]
pub struct LightHandles {
    pub children: Vec<Entity>,
    pub color: Entity,
    pub intensity: Entity,
    pub range: Entity,
    pub inner_cone: Entity,
    pub outer_cone: Entity,
    pub cast_shadows: Entity,
}

#[derive(Default)]
pub struct CameraHandles {
    pub children: Vec<Entity>,
    pub field_of_view: Entity,
    pub near: Entity,
    pub far: Entity,
}

#[derive(Debug)]
pub struct TreeRow {
    pub node: Entity,
    pub wrapper: Entity,
    pub eye: Entity,
    pub target: Entity,
}

#[derive(Debug)]
pub struct LayerRow {
    pub slider: Entity,
    pub remove: Entity,
    pub index: usize,
}

#[derive(Debug)]
pub struct MorphSlider {
    pub slider: Entity,
    pub target: Entity,
    pub index: usize,
}

#[derive(Debug)]
pub struct BrowserCell {
    pub button: Entity,
    pub item: BrowserItem,
}

#[derive(Default)]
pub struct Markers {
    pub lines: Option<Entity>,
    pub drawn: Vec<(nalgebra_glm::Vec3, nalgebra_glm::Vec3, nalgebra_glm::Vec4)>,
}

#[derive(Default)]
pub struct ViewerResources {
    pub camera_input: CameraInput,
    pub sun: Sun,
    pub selection: Selection,
    pub model: Model,
    pub picking: Picking,
    pub history: History,
    pub markers: Markers,
    pub recent: Vec<String>,
    pub incoming: Incoming,
    pub dropped: Dropped,
    pub browsers: Browsers,
    pub library: Library,
    pub skeleton: Skeleton,
    pub gizmo: Gizmo,
    pub view: ViewState,
    pub animation: AnimationView,
    pub actions: Vec<ViewerAction>,
    pub handles: UiHandles,
    pub layout: Layout,
    pub settings: Settings,
    pub settings_delay: u32,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Pending {
    pub name: String,
    pub arguments: serde_json::Value,
    pub reply: std::sync::mpsc::Sender<serde_json::Value>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Default)]
pub struct AgentChannel {
    pub pending: Arc<Mutex<Vec<Pending>>>,
}

impl Drop for ViewerResources {
    fn drop(&mut self) {
        crate::commands::crew::wait_for_the_crew(&mut self.incoming.crew);
    }
}

nightshade::ecs::resource_bundle! {
    pub struct Stage<'a> {
        write {
            viewer: ViewerResources,
            active_camera: nightshade_camera::resources::ActiveCamera,
            settings: nightshade_render::data::config::RenderSettings,
            debug_draw: nightshade_render::data::config::DebugDraw,
            outline: nightshade_render::resources::outline::Outline,
            queues: nightshade_graphics::resources::queues::CommandQueues,
            materials: nightshade_render::resources::material_registry::MaterialRegistry,
            meshes: nightshade_render::resources::mesh_cache::MeshCache,
            mesh_state: nightshade_render::resources::mesh_state::MeshRenderState,
            textures: nightshade_render::resources::texture_cache::TextureCache,
            sources: nightshade_render::resources::texture_sources::TextureSources,
            load_state: nightshade_graphics::resources::loading::LoadingState,
            hierarchy: nightshade::ecs::HierarchyIndex,
            entities: nightshade_scene::resources::entity_registry::EntityRegistry,
            loading: nightshade_gltf::Loading,
            runtime: nightshade_ui::resources::RetainedUiRuntime,
        }
        read {
            arguments: ViewerArguments,
        }
    }
}

impl Stage<'_> {
    pub fn prefab_parts(&mut self) -> nightshade_prefab::PrefabParts<'_> {
        nightshade_prefab::PrefabParts {
            meshes: self.meshes,
            mesh_state: self.mesh_state,
            materials: self.materials,
            textures: self.textures,
            sources: self.sources,
            loading: self.load_state,
            hierarchy: self.hierarchy,
        }
    }
}
