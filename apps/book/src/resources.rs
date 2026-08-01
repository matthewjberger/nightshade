use std::collections::HashMap;

use nightshade::prelude::{Entity, Vec2, Vec3};
use nightshade_cli::clap;

use crate::commands::figure::ControlKind;
use crate::data::book::Chapter;

#[derive(Default)]
pub struct Library {
    pub chapters: Vec<Chapter>,
}

#[derive(Default)]
pub struct Shell {
    pub page_host: Entity,
    pub content_area: Entity,
    pub scroll_area: Entity,
    pub backdrops: Vec<Entity>,
    pub nav_view: Entity,
    pub expand_all: Entity,
    pub collapse_all: Entity,
    pub rail_host: Entity,
    pub nav_nodes: Vec<Entity>,
    pub nav_chapters: Vec<usize>,
    pub nav_anchors: Vec<Option<String>>,
    pub demo_rows: Vec<Entity>,
    pub demo_targets: Vec<(usize, Option<String>)>,
    pub breadcrumb: Entity,
    pub previous_button: Entity,
    pub next_button: Entity,
    pub search_button: Entity,
    pub palette: Entity,
    pub progress_fill: Entity,
    pub appearance_button: Entity,
    pub appearance_panel: Entity,
    pub source_panel: Entity,
    pub source_host: Entity,
    pub source_title: Entity,
    pub source_close: Entity,
    pub theme_rows: Vec<Entity>,
    pub theme_marks: Vec<Entity>,
    pub size_rows: Vec<Entity>,
    pub size_marks: Vec<Entity>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PagePhase {
    #[default]
    Idle,
    Clear,
    Build,
}

pub struct BookState {
    pub current: usize,
    pub requested: Option<usize>,
    pub pending: Option<usize>,
    pub page_root: Option<Entity>,
    pub phase: PagePhase,
    pub theme_name: String,
    pub type_scale: f32,
    pub page_width: f32,
    pub jump_to: Option<String>,
    pub rail_root: Option<Entity>,
    pub fold_all: Option<bool>,
    pub scroll_to: Option<f32>,
}

impl Default for BookState {
    fn default() -> Self {
        Self {
            current: 0,
            requested: Some(0),
            pending: None,
            page_root: None,
            phase: PagePhase::Idle,
            theme_name: String::new(),
            type_scale: 1.0,
            page_width: 0.0,
            jump_to: None,
            rail_root: None,
            fold_all: None,
            scroll_to: None,
        }
    }
}

#[derive(Default)]
pub struct PageStage {
    pub figures: Vec<Figure>,
    pub headings: Vec<(String, Entity)>,
    pub expanded: Option<usize>,
    pub reading_from: f32,
}

pub struct Figure {
    pub demo: &'static str,
    pub card: Entity,
    pub node: Entity,
    pub controls: Vec<(&'static str, Entity, ControlKind)>,
    pub source_button: Entity,
    pub expand_button: Entity,
    pub expand_label: Entity,
    pub card_height: f32,
}

#[derive(Default)]
pub struct SourceView {
    pub demo: Option<&'static str>,
    pub file: usize,
    pub root: Option<Entity>,
    pub shown: Option<(&'static str, usize, i32)>,
    pub rows: Vec<Entity>,
    pub row_files: Vec<usize>,
}

#[derive(Default)]
pub struct DemoRuntime {
    pub active: Vec<&'static str>,
    pub mounting: Option<&'static str>,
    pub mounting_bit: u32,
    pub saved_settings: Option<nightshade_render::data::config::RenderSettings>,
    pub saved_camera: Option<Entity>,
    pub meshes: HashMap<&'static str, Vec<String>>,
    pub materials: HashMap<&'static str, Vec<String>>,
    pub cameras: HashMap<&'static str, Entity>,
}

#[derive(Default)]
pub struct DemoInput {
    pub demo: Option<&'static str>,
    pub hovered: bool,
    pub pointer: Vec2,
    pub drag: Vec2,
    pub dragging: bool,
    pub wheel: f32,
    pub pressed: bool,
    pub stage_size: Vec2,
    pub camera_origin: Vec3,
    pub camera_forward: Vec3,
    pub camera_right: Vec3,
    pub camera_up: Vec3,
    pub camera_half_height: f32,
    pub clicked: Vec<&'static str>,
    pub values: HashMap<&'static str, f32>,
    pub stages: HashMap<&'static str, StageAim>,
    pub held: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StageAim {
    pub hovered: bool,
    pub pointer: Vec2,
    pub drag: Vec2,
    pub dragging: bool,
    pub wheel: f32,
    pub pressed: bool,
    pub stage_size: Vec2,
    pub camera_origin: Vec3,
    pub camera_forward: Vec3,
    pub camera_right: Vec3,
    pub camera_up: Vec3,
    pub camera_half_height: f32,
}

impl DemoInput {
    pub fn value(&self, action: &str, fallback: f32) -> f32 {
        self.values.get(action).copied().unwrap_or(fallback)
    }

    pub fn switched_on(&self, action: &str, fallback: bool) -> bool {
        self.value(action, if fallback { 1.0 } else { 0.0 }) > 0.5
    }
}

#[derive(Default)]
pub struct Reading {
    pub offset: f32,
    pub height: f32,
    pub visible: f32,
    pub progress: f32,
}

#[derive(Default)]
pub struct Search {
    pub open: bool,
    pub filled: bool,
    pub entries: Vec<SearchEntry>,
}

pub struct Appearance {
    pub open: bool,
    pub asked: bool,
    pub sizes: Vec<(&'static str, f32)>,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            open: false,
            asked: false,
            sizes: vec![
                ("Small", 0.85),
                ("Normal", 1.0),
                ("Large", 1.15),
                ("Largest", 1.32),
            ],
        }
    }
}

impl Appearance {
    pub fn scale_at(&self, index: usize) -> f32 {
        self.sizes
            .get(index)
            .map(|(_, scale)| *scale)
            .unwrap_or(1.0)
    }
}

#[derive(clap::Args, Default)]
pub struct BookArguments {
    #[arg(
        long,
        value_name = "PAGE",
        help = "Open the book at this chapter, by number or by title"
    )]
    pub page: Option<String>,
    #[arg(
        long,
        value_name = "PIXELS",
        help = "Put the reader this far down the page"
    )]
    pub scroll: Option<f32>,
    #[arg(long, help = "Open with the panel of themes and sizes shown")]
    pub appearance: bool,
    #[arg(
        long,
        value_name = "DIRECTORY",
        help = "Work the panel of themes by itself and write pictures of it here"
    )]
    pub rehearse: Option<std::path::PathBuf>,
}

#[derive(Default)]
pub struct Rehearsal {
    pub driver: Option<nightshade_ui_test::UiTestDriver>,
    pub directory: std::path::PathBuf,
    pub frame: u64,
}

#[derive(Clone, Debug)]
pub struct SearchEntry {
    pub label: String,
    pub chapter: usize,
    pub anchor: Option<String>,
}

nightshade::ecs::resource_bundle! {
    pub struct DemoScene<'a> {
        write {
            runtime: DemoRuntime,
            settings: nightshade_render::data::config::RenderSettings,
            active_camera: nightshade_camera::resources::ActiveCamera,
            materials: nightshade_render::resources::material_registry::MaterialRegistry,
            meshes: nightshade_render::resources::mesh_cache::MeshCache,
            mesh_state: nightshade_render::resources::mesh_state::MeshRenderState,
            entities: nightshade_scene::resources::entity_registry::EntityRegistry,
            textures: nightshade_render::resources::texture_cache::TextureCache,
            hierarchy: nightshade::ecs::HierarchyIndex,
            input: DemoInput,
        }
        read {}
    }
}
