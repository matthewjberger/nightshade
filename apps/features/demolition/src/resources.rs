use nalgebra_glm::{IVec3, Vec3};
use nightshade::prelude::Entity;
pub use nightshade_voxels::prelude::{VoxelSettings, VoxelViewer, VoxelWorld};

pub struct Digging;

#[derive(Default)]
pub struct Plan;

pub struct Watcher {
    pub camera: Option<Entity>,
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for Watcher {
    fn default() -> Self {
        Self {
            camera: None,
            position: crate::theme::EYE,
            yaw: crate::theme::YAW,
            pitch: crate::theme::PITCH,
        }
    }
}

pub struct Aim {
    pub hit: Option<IVec3>,
    pub marker: Option<Entity>,
    pub marked: bool,
    pub pulse: f32,
    pub radius: i32,
    pub blasted: u64,
}

impl Default for Aim {
    fn default() -> Self {
        Self {
            hit: None,
            marker: None,
            marked: false,
            pulse: 0.0,
            radius: crate::theme::BLAST_RADIUS,
            blasted: 0,
        }
    }
}

#[derive(Default)]
pub struct Cursor {
    pub shown: bool,
}

#[derive(Default, Clone)]
pub struct UiHandles {
    pub built: bool,
    pub hud: Entity,
    pub radius_label: Entity,
    pub tally_label: Entity,
    pub title_panel: Entity,
    pub shown_radius: Option<i32>,
    pub shown_tally: Option<u64>,
    pub shown_locked: Option<bool>,
}
