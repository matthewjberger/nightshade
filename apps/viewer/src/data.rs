use std::collections::BTreeMap;

use nightshade::prelude::Entity;
use serde::Deserialize;

use crate::resources::MaterialView;

#[derive(Deserialize)]
pub struct KhronosRaw {
    pub label: String,
    pub name: String,
    #[serde(default)]
    pub variants: BTreeMap<String, String>,
    #[serde(default)]
    pub screenshot: Option<String>,
}

#[derive(Deserialize)]
pub struct PolyRaw {
    pub name: String,
}

#[derive(Deserialize)]
pub struct FileLink {
    pub url: String,
}

#[derive(Deserialize)]
pub struct SkyResolution {
    #[serde(default)]
    pub hdr: Option<FileLink>,
}

#[derive(Deserialize)]
pub struct SkyFiles {
    pub hdri: BTreeMap<String, SkyResolution>,
}

#[derive(Deserialize)]
pub struct IncludeFile {
    pub url: String,
}

#[derive(Deserialize)]
pub struct GltfFile {
    pub url: String,
    #[serde(default)]
    pub include: BTreeMap<String, IncludeFile>,
}

#[derive(Deserialize)]
pub struct GltfResolution {
    pub gltf: GltfFile,
}

#[derive(Deserialize)]
pub struct ModelFiles {
    pub gltf: BTreeMap<String, GltfResolution>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ViewerAction {
    ToggleScenePanel,
    ToggleInspectorPanel,
    OpenAddMenu,
    CloseAddMenu,
    SetGrid(bool),
    SetTurntable(bool),
    FrameModel,
    OpenBrowser(BrowserTab),
    ImportFile,
    RefreshLibrary,
    AddLibraryFolder,
    AddLibraryFiles,
    ForgetLibraryPath(String),
    OpenLibraryFile(String),
    CloseBrowser,
    SetBrowserTab(BrowserTab),
    SetBrowserPage(usize),
    SetResolution(u32),
    RandomModel,
    RandomSky,
    SelectPanelTab(PanelTab),
    Select(Option<Entity>),
    ToggleInSelection(Entity),
    DeleteSelected,
    SetTranslation([f32; 3]),
    SetRotation([f32; 3]),
    SetScale([f32; 3]),
    Rename(String),
    SetNodeVisible {
        entity: Entity,
        visible: bool,
    },
    ToggleSelectedVisible,
    ToggleIsolate,
    AddPrimitive(PrimitiveKind),
    AddLight(LightKind),
    SetShadingMode(usize),
    SetPbrDebug(usize),
    SetShowNormals(bool),
    SetShowBounds(bool),
    SetShowSkeleton(bool),
    SetShowSky(bool),
    SetAtmosphere(usize),
    SetExposure(f32),
    SetTonemap(usize),
    SetSsao(bool),
    SetBloom(bool),
    SetSsr(bool),
    SetSsgi(bool),
    SetAntiAliasing(usize),
    SetRenderScale(f32),
    SetFog(bool),
    SetFogStart(f32),
    SetFogEnd(f32),
    SetFogColor([f32; 3]),
    SetDepthOfField(bool),
    SetFocusDistance(f32),
    SetFocusRange(f32),
    SetSunIntensity(f32),
    SetSunYaw(f32),
    SetSunPitch(f32),
    SetSunShadows(bool),
    SetVariant(Option<String>),
    SetMaterialField(MaterialField),
    ResetMaterial,
    ReplaceTexture(TextureSlot),
    SetLightField(LightField),
    SetCameraField(CameraField),
    SetMorphWeight {
        entity: Entity,
        index: usize,
        weight: f32,
    },
    BlendToClip(usize),
    SetBlendTime(f32),
    AddAnimationLayer {
        clip: usize,
        weight: f32,
    },
    SetLayerWeight {
        index: usize,
        weight: f32,
    },
    RemoveAnimationLayer(usize),
    TogglePlayback,
    StopAnimation,
    SeekAnimation(f32),
    SetAnimationSpeed(f32),
    SetAnimationLoop(bool),
    SnapAxis([f32; 3]),
    SetGizmoMode(GizmoMode),
    SetSnap(bool),
    SetActiveCamera(Option<Entity>),
    LoadKhronos(String),
    LoadPolyhavenModel(String),
    LoadPolyhavenSky(String),
    Screenshot,
    ExportModel,
    ApplyHeldClips,
    OpenRecent,
    CloseRecent,
    LoadRecent(String),
    Undo,
    Redo,
    ToggleInterface,
    ShowInterface,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaterialField {
    Texture(TextureSlot, Option<String>),
    BaseColor([f32; 4]),
    Metallic(f32),
    Roughness(f32),
    AlphaMode(usize),
    AlphaCutoff(f32),
    Unlit(bool),
    DoubleSided(bool),
    Emissive([f32; 3]),
    EmissiveStrength(f32),
    NormalScale(f32),
    OcclusionStrength(f32),
    Ior(f32),
    Specular(f32),
    Transmission(f32),
    Thickness(f32),
    Dispersion(f32),
    Clearcoat(f32),
    ClearcoatRoughness(f32),
    SheenRoughness(f32),
    Iridescence(f32),
    IridescenceIor(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureSlot {
    BaseColor,
    MetallicRoughness,
    Normal,
    Occlusion,
    Emissive,
    Transmission,
    Thickness,
    Specular,
}

impl TextureSlot {
    pub const ALL: [TextureSlot; 8] = [
        TextureSlot::BaseColor,
        TextureSlot::MetallicRoughness,
        TextureSlot::Normal,
        TextureSlot::Occlusion,
        TextureSlot::Emissive,
        TextureSlot::Transmission,
        TextureSlot::Thickness,
        TextureSlot::Specular,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TextureSlot::BaseColor => "Base color",
            TextureSlot::MetallicRoughness => "Metallic roughness",
            TextureSlot::Normal => "Normal",
            TextureSlot::Occlusion => "Occlusion",
            TextureSlot::Emissive => "Emissive",
            TextureSlot::Transmission => "Transmission",
            TextureSlot::Thickness => "Thickness",
            TextureSlot::Specular => "Specular",
        }
    }

    pub fn is_color(self) -> bool {
        matches!(self, TextureSlot::BaseColor | TextureSlot::Emissive)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightField {
    Color([f32; 3]),
    Intensity(f32),
    Range(f32),
    InnerCone(f32),
    OuterCone(f32),
    CastShadows(bool),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CameraField {
    FieldOfView(f32),
    Near(f32),
    Far(f32),
}

pub const ALPHA_LABELS: [&str; 4] = ["Opaque", "Mask", "Blend", "Dither"];

pub const LIGHT_KIND_LABELS: [&str; 3] = ["Directional", "Point", "Spot"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserTab {
    Library,
    Khronos,
    Models,
    Skies,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryKind {
    Model,
    Sky,
}

#[cfg(not(target_arch = "wasm32"))]
impl LibraryKind {
    pub fn of(name: &str) -> Option<Self> {
        let lowered = name.to_lowercase();
        if [".glb", ".gltf", ".fbx", ".zip"]
            .iter()
            .any(|extension| lowered.ends_with(extension))
        {
            return Some(Self::Model);
        }
        lowered.ends_with(".hdr").then_some(Self::Sky)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelTab {
    Scene,
    Render,
    Stats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveKind {
    Cube,
    Sphere,
    Cylinder,
    Cone,
    Torus,
    Plane,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightKind {
    Directional,
    Point,
    Spot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    Mesh,
    Light,
    Camera,
    Node,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GizmoMode {
    #[default]
    Move,
    Rotate,
    Scale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoHandle {
    Axis(usize),
    Plane(usize),
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickKind {
    Click,
    Hover,
}

pub const PICK_PATIENCE: u32 = 120;

pub const MERGE_FRAMES: u64 = 30;

pub const SHADING_LABELS: [&str; 4] = ["Rendered", "Solid", "Flat", "Wireframe"];

pub const PBR_LABELS: [&str; 7] = [
    "Off",
    "Base color",
    "Normal",
    "Metallic",
    "Roughness",
    "Occlusion",
    "Emissive",
];

pub const TONEMAP_LABELS: [&str; 6] = ["ACES", "Reinhard", "Uncharted 2", "AgX", "Neutral", "None"];

pub const ATMOSPHERE_LABELS: [&str; 8] = [
    "None",
    "Sky",
    "Cloudy sky",
    "Space",
    "Nebula",
    "Sunset",
    "Day and night",
    "HDR file",
];

pub const ANTI_ALIASING_LABELS: [&str; 3] = ["Off", "FXAA", "TAA"];

pub const RESOLUTION_LABELS: [&str; 4] = ["1k", "2k", "4k", "8k"];

pub const RESOLUTION_VALUES: [u32; 4] = [1, 2, 4, 8];

#[derive(Clone, Debug)]
pub enum BrowserItem {
    Khronos(String),
    Model(String),
    Sky(String),
    File(String),
}

#[derive(Clone, Copy)]
pub enum MaterialControl {
    Metallic,
    Roughness,
    AlphaCutoff,
    EmissiveStrength,
    NormalScale,
    OcclusionStrength,
    Ior,
    Specular,
    Transmission,
    Thickness,
    Dispersion,
    Clearcoat,
    ClearcoatRoughness,
    SheenRoughness,
    Iridescence,
    IridescenceIor,
}

impl MaterialControl {
    pub fn row(self, values: &MaterialView) -> (&'static str, f32, f32, f32) {
        match self {
            Self::Metallic => ("Metallic", 0.0, 1.0, values.metallic),
            Self::Roughness => ("Roughness", 0.0, 1.0, values.roughness),
            Self::AlphaCutoff => ("Alpha cutoff", 0.0, 1.0, values.alpha_cutoff),
            Self::EmissiveStrength => ("Strength", 0.0, 20.0, values.emissive_strength),
            Self::NormalScale => ("Normal scale", 0.0, 4.0, values.normal_scale),
            Self::OcclusionStrength => ("Occlusion", 0.0, 1.0, values.occlusion_strength),
            Self::Ior => ("Index", 1.0, 3.0, values.ior),
            Self::Specular => ("Specular", 0.0, 1.0, values.specular),
            Self::Transmission => ("Transmission", 0.0, 1.0, values.transmission),
            Self::Thickness => ("Thickness", 0.0, 10.0, values.thickness),
            Self::Dispersion => ("Dispersion", 0.0, 1.0, values.dispersion),
            Self::Clearcoat => ("Clearcoat", 0.0, 1.0, values.clearcoat),
            Self::ClearcoatRoughness => ("Coat rough", 0.0, 1.0, values.clearcoat_roughness),
            Self::SheenRoughness => ("Sheen rough", 0.0, 1.0, values.sheen_roughness),
            Self::Iridescence => ("Iridescence", 0.0, 1.0, values.iridescence),
            Self::IridescenceIor => ("Film index", 1.0, 3.0, values.iridescence_ior),
        }
    }

    pub fn field(self, value: f32) -> MaterialField {
        match self {
            Self::Metallic => MaterialField::Metallic(value),
            Self::Roughness => MaterialField::Roughness(value),
            Self::AlphaCutoff => MaterialField::AlphaCutoff(value),
            Self::EmissiveStrength => MaterialField::EmissiveStrength(value),
            Self::NormalScale => MaterialField::NormalScale(value),
            Self::OcclusionStrength => MaterialField::OcclusionStrength(value),
            Self::Ior => MaterialField::Ior(value),
            Self::Specular => MaterialField::Specular(value),
            Self::Transmission => MaterialField::Transmission(value),
            Self::Thickness => MaterialField::Thickness(value),
            Self::Dispersion => MaterialField::Dispersion(value),
            Self::Clearcoat => MaterialField::Clearcoat(value),
            Self::ClearcoatRoughness => MaterialField::ClearcoatRoughness(value),
            Self::SheenRoughness => MaterialField::SheenRoughness(value),
            Self::Iridescence => MaterialField::Iridescence(value),
            Self::IridescenceIor => MaterialField::IridescenceIor(value),
        }
    }
}

pub const BASE: [MaterialControl; 3] = [
    MaterialControl::Metallic,
    MaterialControl::Roughness,
    MaterialControl::AlphaCutoff,
];

pub const SURFACE: [MaterialControl; 4] = [
    MaterialControl::NormalScale,
    MaterialControl::OcclusionStrength,
    MaterialControl::Ior,
    MaterialControl::Specular,
];

pub const THROUGH: [MaterialControl; 3] = [
    MaterialControl::Transmission,
    MaterialControl::Thickness,
    MaterialControl::Dispersion,
];

pub const COAT: [MaterialControl; 5] = [
    MaterialControl::Clearcoat,
    MaterialControl::ClearcoatRoughness,
    MaterialControl::SheenRoughness,
    MaterialControl::Iridescence,
    MaterialControl::IridescenceIor,
];

pub const PRIMITIVES: [PrimitiveKind; 6] = [
    PrimitiveKind::Cube,
    PrimitiveKind::Sphere,
    PrimitiveKind::Cylinder,
    PrimitiveKind::Cone,
    PrimitiveKind::Torus,
    PrimitiveKind::Plane,
];

pub const LIGHTS: [LightKind; 3] = [LightKind::Directional, LightKind::Point, LightKind::Spot];

pub const TABS: [PanelTab; 3] = [PanelTab::Scene, PanelTab::Render, PanelTab::Stats];

pub const BROWSER_TABS: [BrowserTab; 4] = [
    BrowserTab::Library,
    BrowserTab::Khronos,
    BrowserTab::Models,
    BrowserTab::Skies,
];

#[cfg(not(target_arch = "wasm32"))]
pub const LIBRARY_DEPTH: u32 = 6;

#[cfg(not(target_arch = "wasm32"))]
pub const LIBRARY_LIMIT: usize = 40000;

pub const LIBRARY_WATCH_FRAMES: u32 = 300;

pub const LIBRARY_IDLE_FRAMES: u32 = 3600;

#[cfg(not(target_arch = "wasm32"))]
pub const PREVIEW_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

#[cfg(not(target_arch = "wasm32"))]
pub const PREVIEW_FOLDER: &str = "nightshade-viewer/previews";

pub const PREVIEW_MARK: &str = "preview:";

#[cfg(not(target_arch = "wasm32"))]
pub const PREVIEW_PIXELS: u32 = 400;

#[cfg(not(target_arch = "wasm32"))]
pub const PREVIEW_FRAME: u32 = 4;

#[cfg(not(target_arch = "wasm32"))]
pub const PREVIEW_JOBS: usize = 3;

pub const PREVIEW_PATIENCE: u64 = 20_000;

#[cfg(not(target_arch = "wasm32"))]
pub const PREVIEW_GIVE_UP: u64 = 60;

pub const PREVIEW_FIT: f32 = 0.86;

pub const PREVIEW_YAW: f32 = -0.62;

pub const PREVIEW_PITCH: f32 = 0.38;

pub const GIZMO_MODES: [GizmoMode; 3] = [GizmoMode::Move, GizmoMode::Rotate, GizmoMode::Scale];

pub const AXES: [([f32; 3], &str, usize, bool); 6] = [
    ([1.0, 0.0, 0.0], "X", 0, true),
    ([0.0, 1.0, 0.0], "Y", 1, true),
    ([0.0, 0.0, 1.0], "Z", 2, true),
    ([-1.0, 0.0, 0.0], "X", 0, false),
    ([0.0, -1.0, 0.0], "Y", 1, false),
    ([0.0, 0.0, -1.0], "Z", 2, false),
];

pub const PAGE_SIZE: usize = 60;

pub const MAX_THUMBNAILS: usize = 120;

pub const KHRONOS_BASE: &str =
    "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models";

pub const KHRONOS_INDEX: &str = "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/model-index.json";

pub const POLYHAVEN_SKIES: &str = "https://api.polyhaven.com/assets?type=hdris";

pub const POLYHAVEN_MODELS: &str = "https://api.polyhaven.com/assets?type=models";

pub const POLYHAVEN_FILES: &str = "https://api.polyhaven.com/files/";

pub const POLYHAVEN_THUMB: &str = "https://cdn.polyhaven.com/asset_img/thumbs/";

pub const RECENT_LIMIT: usize = 10;

pub const PICK_RADIUS: f32 = 0.09;

pub const PLANE_OFFSET: f32 = 0.42;

pub const PLANE_SIZE: f32 = 0.26;

pub const RING_RADIUS: f32 = 0.95;

pub const CENTER_SIZE: f32 = 0.08;

pub const PLANE_AXES: [(usize, usize); 3] = [(1, 2), (2, 0), (0, 1)];

pub const REACH: f32 = 4.0;

pub const FOLDER: &str = "nightshade-viewer";

pub const FILE: &str = "settings.toml";

#[cfg(not(target_arch = "wasm32"))]
pub const DEFAULT_PORT: u16 = 8791;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct Empty {}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct EntityArgs {
    pub entity: u32,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct SelectArgs {
    pub entity: Option<u32>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct TransformArgs {
    pub entity: u32,
    pub translation: Option<[f32; 3]>,
    pub rotation: Option<[f32; 3]>,
    pub scale: Option<[f32; 3]>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct MaterialArgs {
    pub entity: u32,
    pub base_color: Option<[f32; 4]>,
    pub metallic: Option<f32>,
    pub roughness: Option<f32>,
    pub emissive: Option<[f32; 3]>,
    pub emissive_strength: Option<f32>,
    pub ior: Option<f32>,
    pub specular: Option<f32>,
    pub transmission: Option<f32>,
    pub thickness: Option<f32>,
    pub unlit: Option<bool>,
    pub double_sided: Option<bool>,
    pub alpha_mode: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct PrimitiveArgs {
    pub kind: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct LightArgs {
    pub kind: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct PathArgs {
    pub path: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct NameArgs {
    pub name: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct SlugArgs {
    pub slug: String,
    pub resolution: Option<u32>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct ClipArgs {
    pub index: usize,
    pub blend_time: Option<f32>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct LayerArgs {
    pub index: usize,
    pub weight: Option<f32>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct GizmoArgs {
    pub mode: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct RenderArgs {
    pub exposure: Option<f32>,
    pub tonemap: Option<usize>,
    pub shading: Option<usize>,
    pub pbr_debug: Option<usize>,
    pub atmosphere: Option<usize>,
    pub grid: Option<bool>,
    pub sky: Option<bool>,
    pub normals: Option<bool>,
    pub bounds: Option<bool>,
    pub skeleton: Option<bool>,
    pub turntable: Option<bool>,
    pub ssao: Option<bool>,
    pub bloom: Option<bool>,
    pub ssr: Option<bool>,
    pub ssgi: Option<bool>,
    pub anti_aliasing: Option<usize>,
    pub render_scale: Option<f32>,
    pub fog: Option<bool>,
    pub depth_of_field: Option<bool>,
    pub sun_intensity: Option<f32>,
    pub sun_yaw: Option<f32>,
    pub sun_pitch: Option<f32>,
    pub sun_shadows: Option<bool>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct VisibleArgs {
    pub entity: u32,
    pub visible: bool,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct RenameArgs {
    pub entity: u32,
    pub name: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct LightWriteArgs {
    pub entity: u32,
    pub color: Option<[f32; 3]>,
    pub intensity: Option<f32>,
    pub range: Option<f32>,
    pub inner_cone: Option<f32>,
    pub outer_cone: Option<f32>,
    pub cast_shadows: Option<bool>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct PageArgs {
    pub tab: String,
    pub search: Option<String>,
    pub page: Option<usize>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct ScreenshotArgs {
    pub path: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct ExportArgs {
    pub path: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Deserialize, enum2schema::Schema)]
pub struct CameraArgs {
    pub entity: Option<u32>,
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub const FBX_PACK_SCRIPT: &str = r#"
import bpy
import os
import sys


def after(flag):
    tail = sys.argv[sys.argv.index("--") + 1:]
    return tail[tail.index(flag) + 1]


SOURCE = after("--source")
OUTPUT = after("--output")


def clip_name(filename):
    return os.path.splitext(os.path.basename(filename))[0].strip().lower().replace(" ", "_").replace("-", "_")


def call_filtered(operator, arguments):
    permitted = operator.get_rna_type().properties.keys()
    return operator(**{key: value for key, value in arguments.items() if key in permitted})


def import_fbx(path):
    call_filtered(bpy.ops.import_scene.fbx, {
        "filepath": path,
        "automatic_bone_orientation": True,
        "ignore_leaf_bones": False,
        "use_anim": True,
        "anim_offset": 0.0,
    })


def frame_span(action):
    start, end = action.frame_range
    return int(round(end - start)) + 1


def assign_slot(holder, action):
    if hasattr(holder, "action_slot") and getattr(holder, "action_slot", None) is None:
        slots = getattr(action, "slots", None)
        if slots:
            holder.action_slot = slots[0]


bpy.ops.wm.read_factory_settings(use_empty=True)

records = []
for filename in sorted(os.listdir(SOURCE)):
    if not filename.lower().endswith(".fbx"):
        continue
    before = set(bpy.data.objects)
    import_fbx(os.path.join(SOURCE, filename))
    added = [ob for ob in bpy.data.objects if ob not in before]
    armature = next((ob for ob in added if ob.type == "ARMATURE"), None)
    if armature is None:
        continue
    action = armature.animation_data.action if armature.animation_data else None
    meshes = [ob for ob in added if ob.type == "MESH"]
    records.append((filename, armature, meshes, action, added))

if not records:
    raise SystemExit("the pack holds no skeleton")

character = next((record for record in records if record[2]), records[0])
target = character[1]
target.name = "Armature"
target.data.name = "Armature"

if target.animation_data is None:
    target.animation_data_create()
for track in list(target.animation_data.nla_tracks):
    target.animation_data.nla_tracks.remove(track)

taken = set()
for filename, armature, meshes, action, added in records:
    if action is None:
        continue
    if armature is target and frame_span(action) <= 2 and len(records) > 1:
        armature.animation_data.action = None
        bpy.data.actions.remove(action)
        continue
    armature.animation_data.action = None
    name = clip_name(filename)
    suffix = 1
    while name in taken:
        suffix += 1
        name = "%s_%d" % (clip_name(filename), suffix)
    taken.add(name)
    action.name = name
    action.use_fake_user = True
    track = target.animation_data.nla_tracks.new()
    track.name = name
    strip = track.strips.new(name, int(action.frame_range[0]), action)
    strip.name = name
    assign_slot(strip, action)
    track.mute = True
    print("CLIP", name)

for filename, armature, meshes, action, added in records:
    if armature is target:
        continue
    for obj in added:
        if obj.name in bpy.data.objects:
            bpy.data.objects.remove(obj, do_unlink=True)

target.animation_data.action = None
bpy.ops.outliner.orphans_purge(do_local_ids=True, do_linked_ids=True, do_recursive=True)

call_filtered(bpy.ops.export_scene.gltf, {
    "filepath": OUTPUT,
    "export_format": "GLB",
    "use_selection": False,
    "export_apply": False,
    "export_yup": True,
    "export_materials": "EXPORT",
    "export_skins": True,
    "export_rest_position_armature": True,
    "export_animations": True,
    "export_animation_mode": "ACTIONS",
    "export_frame_range": False,
    "export_force_sampling": True,
    "export_bake_animation": True,
    "export_optimize_animation_size": False,
    "export_anim_single_armature": True,
    "export_reset_pose_bones": True,
    "export_def_bones": False,
    "export_extras": False,
})
print("WROTE", OUTPUT)
"#;
