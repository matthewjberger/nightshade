use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, enum2schema::Schema)]
pub struct NavMeshAgent {
    pub current_path: Vec<Vec3>,
    pub current_waypoint_index: usize,
    pub target_position: Option<Vec3>,
    pub movement_speed: f32,
    pub arrival_threshold: f32,
    pub path_recalculation_threshold: f32,
    pub agent_radius: f32,
    pub agent_height: f32,
    pub current_triangle: Option<usize>,
    pub state: NavMeshAgentState,
    pub distance_to_destination: f32,
    pub avoidance_velocity: Vec3,
}

impl Default for NavMeshAgent {
    fn default() -> Self {
        Self {
            current_path: Vec::new(),
            current_waypoint_index: 0,
            target_position: None,
            movement_speed: 5.0,
            arrival_threshold: 0.3,
            path_recalculation_threshold: 2.0,
            agent_radius: 0.3,
            agent_height: 1.8,
            current_triangle: None,
            state: NavMeshAgentState::Idle,
            distance_to_destination: 0.0,
            avoidance_velocity: Vec3::zeros(),
        }
    }
}

impl NavMeshAgent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.movement_speed = speed;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.agent_radius = radius;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.agent_height = height;
        self
    }

    pub fn with_arrival_threshold(mut self, threshold: f32) -> Self {
        self.arrival_threshold = threshold;
        self
    }

    pub fn set_destination(&mut self, destination: Vec3) {
        self.target_position = Some(destination);
        self.state = NavMeshAgentState::PathPending;
    }

    pub fn clear_destination(&mut self) {
        self.target_position = None;
        self.current_path.clear();
        self.current_waypoint_index = 0;
        self.state = NavMeshAgentState::Idle;
    }

    pub fn has_path(&self) -> bool {
        !self.current_path.is_empty()
    }

    pub fn is_moving(&self) -> bool {
        self.state == NavMeshAgentState::Moving
    }

    pub fn current_waypoint(&self) -> Option<Vec3> {
        self.current_path.get(self.current_waypoint_index).copied()
    }

    pub fn remaining_waypoints(&self) -> usize {
        if self.current_waypoint_index >= self.current_path.len() {
            0
        } else {
            self.current_path.len() - self.current_waypoint_index
        }
    }

    pub fn advance_waypoint(&mut self) -> bool {
        self.current_waypoint_index += 1;
        if self.current_waypoint_index >= self.current_path.len() {
            self.state = NavMeshAgentState::Arrived;
            true
        } else {
            false
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum NavMeshAgentState {
    #[default]
    Idle,
    PathPending,
    Moving,
    Arrived,
    NoPath,
}

#[derive(Debug, Clone, Copy)]
pub struct NavigationArrived {
    pub entity: nightshade_ecs::Entity,
}
