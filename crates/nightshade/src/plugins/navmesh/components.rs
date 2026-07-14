//! Navigation mesh component definitions.

use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

/// Navigation agent component for navmesh-based movement.
///
/// Attach to an entity with a transform to enable pathfinding. Set a destination
/// via [`set_destination`](NavMeshAgent::set_destination) and the navigation system
/// will compute and follow a path.
#[derive(Debug, Clone, Serialize, Deserialize, enum2schema::Schema)]
pub struct NavMeshAgent {
    /// Computed path as a list of waypoints.
    pub current_path: Vec<Vec3>,
    /// Index of the next waypoint to reach.
    pub current_waypoint_index: usize,
    /// Target destination (if any).
    pub target_position: Option<Vec3>,
    /// Movement speed in units per second.
    pub movement_speed: f32,
    /// Distance at which waypoints are considered reached.
    pub arrival_threshold: f32,
    /// Distance target must move before path is recalculated.
    pub path_recalculation_threshold: f32,
    /// Agent collision radius for navmesh queries.
    pub agent_radius: f32,
    /// Agent height for navmesh queries.
    pub agent_height: f32,
    /// Current navmesh triangle (for height sampling).
    pub current_triangle: Option<usize>,
    /// Current navigation state.
    pub state: NavMeshAgentState,
    /// Remaining distance to final destination.
    pub distance_to_destination: f32,
    /// Velocity adjustment for local obstacle avoidance.
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
    /// Creates a new navigation agent with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the movement speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.movement_speed = speed;
        self
    }

    /// Sets the agent collision radius.
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.agent_radius = radius;
        self
    }

    /// Sets the agent height.
    pub fn with_height(mut self, height: f32) -> Self {
        self.agent_height = height;
        self
    }

    /// Sets the distance threshold for reaching waypoints.
    pub fn with_arrival_threshold(mut self, threshold: f32) -> Self {
        self.arrival_threshold = threshold;
        self
    }

    /// Sets a new destination and triggers pathfinding.
    pub fn set_destination(&mut self, destination: Vec3) {
        self.target_position = Some(destination);
        self.state = NavMeshAgentState::PathPending;
    }

    /// Clears the current destination and stops movement.
    pub fn clear_destination(&mut self) {
        self.target_position = None;
        self.current_path.clear();
        self.current_waypoint_index = 0;
        self.state = NavMeshAgentState::Idle;
    }

    /// Returns true if a valid path exists.
    pub fn has_path(&self) -> bool {
        !self.current_path.is_empty()
    }

    /// Returns true if the agent is actively navigating.
    pub fn is_moving(&self) -> bool {
        self.state == NavMeshAgentState::Moving
    }

    /// Returns the current waypoint position.
    pub fn current_waypoint(&self) -> Option<Vec3> {
        self.current_path.get(self.current_waypoint_index).copied()
    }

    /// Returns the number of waypoints remaining.
    pub fn remaining_waypoints(&self) -> usize {
        if self.current_waypoint_index >= self.current_path.len() {
            0
        } else {
            self.current_path.len() - self.current_waypoint_index
        }
    }

    /// Advances to the next waypoint. Returns true if destination reached.
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

/// Navigation agent state machine.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum NavMeshAgentState {
    /// No destination set.
    #[default]
    Idle,
    /// Waiting for pathfinding to complete.
    PathPending,
    /// Following path to destination.
    Moving,
    /// Reached the destination.
    Arrived,
    /// No valid path could be found.
    NoPath,
}
