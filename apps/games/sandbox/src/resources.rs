use nightshade::prelude::Entity;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Title,
    InGame,
    Paused,
}

#[derive(Default)]
pub struct ScreenState {
    pub show_fps: bool,
}

pub struct CameraSettings {
    pub look_sensitivity: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            look_sensitivity: 1.0,
        }
    }
}

#[derive(Default)]
pub struct PlayerState {
    pub player_entity: Option<Entity>,
    pub camera_entity: Option<Entity>,
    pub fly_camera: Option<Entity>,
    pub fly: bool,
}

pub struct PoseKeyframe {
    pub name: String,
    pub joints: Vec<(Entity, nalgebra_glm::Quat)>,
}

pub struct GaitTrack {
    pub entity: Entity,
    pub axis: nalgebra_glm::Vec3,
    pub run: [f32; 6],
    pub walk: [f32; 6],
    pub crouch_walk: [f32; 6],
}

#[derive(Clone, Copy)]
pub struct IkChain {
    pub root: Entity,
    pub middle: Entity,
    pub end: Entity,
}

pub struct FootPlant {
    pub chain: IkChain,
    pub phase_offset: f32,
    pub in_stance: bool,
    pub initialized: bool,
    pub plant_ball: nalgebra_glm::Vec3,
    pub plant_normal: nalgebra_glm::Vec3,
    pub stance_ankle: nalgebra_glm::Vec3,
    pub stance_pitch: f32,
    pub lifted: bool,
    pub lift_phase: f32,
    pub last_phase: f32,
    pub lift_offset: nalgebra_glm::Vec3,
    pub lift_height: f32,
    pub lift_pitch: f32,
    pub flat_rotation: nalgebra_glm::Quat,
}

pub struct SoftJoint {
    pub entity: Entity,
    pub stiffness: f32,
    pub damping: f32,
    pub rotation: nalgebra_glm::Quat,
    pub angular_velocity: nalgebra_glm::Vec3,
    pub initialized: bool,
}

#[derive(Default)]
pub struct PoseState {
    pub keyframes: Vec<PoseKeyframe>,
    pub active: Option<usize>,
    pub rest_rotations: HashMap<Entity, nalgebra_glm::Quat>,
    pub gait_tracks: Vec<GaitTrack>,
    pub crouch_index: Option<usize>,
    pub air_rise_index: Option<usize>,
    pub air_fall_index: Option<usize>,
    pub tuck_forward_index: Option<usize>,
    pub tuck_side_index: Option<usize>,
    pub hang_index: Option<usize>,
    pub leg_chains: Vec<IkChain>,
    pub arm_chains: Vec<IkChain>,
    pub feet: Vec<FootPlant>,
    pub leg_length: f32,
    pub ankle_height: f32,
    pub toe_height: f32,
    pub foot_length: f32,
    pub pelvis_drop: f32,
    pub look_joints: Vec<(Entity, f32)>,
    pub idle_joints: Vec<(Entity, nalgebra_glm::Vec3, f32, f32)>,
    pub soft_joints: Vec<SoftJoint>,
    pub hips: Option<Entity>,
    pub hips_rest_translation: nalgebra_glm::Vec3,
    pub hips_drop_local: nalgebra_glm::Vec3,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum RagdollMode {
    #[default]
    Off,
    Limp,
    Match,
    Active,
}

pub struct RagdollLink {
    pub body: Entity,
    pub bone: Entity,
    pub rotation_offset: nalgebra_glm::Quat,
    pub target_world: nalgebra_glm::Quat,
    pub mass: f32,
}

#[derive(Default)]
pub struct RagdollState {
    pub mode: RagdollMode,
    pub links: Vec<RagdollLink>,
    pub auto_recover: bool,
    pub settle_timer: f32,
    pub flail_clock: f32,
}

#[derive(Default)]
pub struct CharacterState {
    pub third_person: bool,
    pub body: Option<Entity>,
    pub camera: Option<Entity>,
    pub spheres: Option<Entity>,
    pub skeleton_lines: Option<Entity>,
    pub model: Option<Entity>,
    pub model_asked: Option<nightshade_load::LoadHandle>,
    pub velocity: nalgebra_glm::Vec3,
    pub grounded: bool,
    pub jump_suppress_timer: f32,
    pub facing_yaw: f32,
    pub lean: nalgebra_glm::Vec3,
    pub stride_phase: f32,
    pub stride_radius: f32,
    pub stride_wheel_visible: bool,
    pub spheres_visible: bool,
    pub skeleton_visible: bool,
    pub gait_weight: f32,
    pub gait_run_factor: f32,
    pub crouch_amount: f32,
    pub crouch_velocity: f32,
    pub air_weight: f32,
    pub air_fall_weight: f32,
    pub flip_time: f32,
    pub flip_duration: f32,
    pub flip_axis: nalgebra_glm::Vec3,
    pub flip_pivot: f32,
    pub flip_side: f32,
    pub tuck_weight: f32,
    pub wall_running: bool,
    pub wall_normal: nalgebra_glm::Vec3,
    pub wall_run_timer: f32,
    pub wall_run_cooldown: f32,
    pub hanging: bool,
    pub hang_normal: nalgebra_glm::Vec3,
    pub hang_tangent: nalgebra_glm::Vec3,
    pub hang_edge_y: f32,
    pub hang_weight: f32,
    pub look_yaw: f32,
    pub look_pitch: f32,
    pub landing_impact_speed: f32,
    pub idle_clock: f32,
}

#[derive(Default)]
pub struct TitleHandles {
    pub root: Entity,
    pub play_button: Entity,
    pub quit_button: Entity,
}

#[derive(Default)]
pub struct PauseHandles {
    pub root: Entity,
    pub resume_button: Entity,
    pub main_menu_button: Entity,
    pub quit_button: Entity,
    pub infinite_world_checkbox: Entity,
    pub third_person_checkbox: Entity,
    pub spheres_checkbox: Entity,
    pub stride_wheel_checkbox: Entity,
    pub skeleton_checkbox: Entity,
    pub fly_checkbox: Entity,
    pub show_fps_checkbox: Entity,
    pub sensitivity_slider: Entity,
    pub main_panel: Entity,
    pub settings_panel: Entity,
    pub settings_button: Entity,
    pub back_button: Entity,
}

#[derive(Default)]
pub struct InfiniteWorldState {
    pub enabled: bool,
    pub ground: Option<Entity>,
}

#[derive(Default)]
pub struct HudHandles {
    pub root: Entity,
    pub fps_label: Entity,
    pub crosshair: Entity,
}

#[derive(Default)]
pub struct UiHandles {
    pub title: TitleHandles,
    pub pause: PauseHandles,
    pub hud: HudHandles,
}

#[derive(Default)]
pub struct SandboxResources {
    pub screen: ScreenState,
    pub camera: CameraSettings,
    pub player: PlayerState,
    pub character: CharacterState,
    pub pose: PoseState,
    pub ragdoll: RagdollState,
    pub ui_handles: UiHandles,
    pub infinite_world: InfiniteWorldState,
}
