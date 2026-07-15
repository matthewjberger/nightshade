//! Navigation mesh pathfinding.
//!
//! Provides AI navigation using pre-computed walkable surfaces:
//!
//! - [`NavMeshAgent`]: Component for entities that navigate via navmesh
//! - [`NavMeshAgentState`]: Agent state machine (Idle, Moving, Arrived, etc.)
//! - `PathRequest`: Async pathfinding request (requires `navmesh` feature)
//! - `find_path_with_algorithm`: Compute a path between two points (requires `navmesh` feature)
//! - `generate_navmesh_recast`: Bake a navmesh from geometry (requires `navmesh-bake` feature)
//!
//! Navigation meshes are triangle meshes representing walkable areas. Pathfinding
//! uses A* with funnel smoothing for natural-looking paths.
//!
//! Two features split bake-time and runtime:
//!
//! - `navmesh` enables runtime systems (pathfinding, agents, debug). No external deps.
//! - `navmesh-bake` adds offline baking via Recast. Pulls `rerecast` and `glam`. Implies `navmesh`.
//!
//! Games typically enable only `navmesh` and load baked navmeshes from level files.
//! Editor and build tooling enable `navmesh-bake` to produce them.
//!
//! # Baking a NavMesh (requires `navmesh-bake`)
//!
//! ```ignore
//! let config = RecastNavMeshConfig::default();
//! let navmesh = generate_navmesh_recast(&vertices, &indices, &config);
//! ```
//!
//! # Spawning NavMesh Agents
//!
//! ```ignore
//! let agent = spawn_navmesh_agent(world, Vec3::new(0.0, 0.0, 0.0), 3.0);  // 3.0 units/sec
//! set_agent_speed(world, agent, 4.0);  // change the speed later
//! ```
//!
//! # Setting Destinations
//!
//! ```ignore
//! // Move agent to target position
//! set_agent_destination(world, agent, Vec3::new(10.0, 0.0, 5.0));
//!
//! // Stop agent
//! stop_agent(world, agent);
//! ```
//!
//! # Checking Agent State
//!
//! ```ignore
//! match get_agent_state(world, agent) {
//!     NavMeshAgentState::Idle => { /* Standing still */ }
//!     NavMeshAgentState::Moving => { /* Walking to destination */ }
//!     NavMeshAgentState::Arrived => { /* Reached destination */ }
//!     NavMeshAgentState::NoPath => { /* No valid path found */ }
//! }
//! ```
//!
//! # Running the System
//!
//! Call each frame to update agent movement:
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     run_navmesh_systems(world);
//! }
//! ```
//!
//! # Debug Visualization
//!
//! ```ignore
//! set_navmesh_debug_draw(world, true);  // Show navmesh triangles and agent paths
//! ```
//!
//! # Sampling Navmesh Height
//!
//! Get the walkable height at a world position:
//!
//! ```ignore
//! if let Some(height) = sample_navmesh_height(world.plugin_resource::<NavMeshWorld>(), x, z) {
//!     let ground_pos = Vec3::new(x, height, z);
//! }
//! ```
//!
//! # RecastNavMeshConfig Parameters
//!
//! | Parameter | Description |
//! |-----------|-------------|
//! | `cell_size_fraction` | XZ voxel size as a fraction of agent radius (smaller = more accurate) |
//! | `cell_height_fraction` | Y voxel height as a fraction of agent radius |
//! | `agent_height` | Character height for clearance |
//! | `agent_radius` | Character radius for obstacle avoidance |
//! | `walkable_climb` | Maximum step height the agent can climb |
//! | `walkable_slope_angle` | Maximum walkable slope, in radians |
//! | `min_region_size` | Minimum region area (filters noise) |
//! | `merge_region_size` | Region merge threshold |
//!
//! [`NavMeshAgent`]: components::NavMeshAgent
//! [`NavMeshAgentState`]: components::NavMeshAgentState

#[cfg(feature = "navmesh")]
pub mod commands;
pub mod components;
#[cfg(feature = "navmesh")]
pub mod debug;
#[cfg(feature = "navmesh")]
pub mod funnel;
#[cfg(feature = "navmesh-bake")]
pub mod generation;
#[cfg(feature = "navmesh")]
pub mod pathfinding;
#[cfg(feature = "navmesh")]
pub mod resources;
#[cfg(feature = "navmesh")]
pub mod systems;

#[cfg(feature = "navmesh")]
pub use commands::*;
pub use components::*;
#[cfg(feature = "navmesh-bake")]
pub use generation::{RecastNavMeshConfig, generate_navmesh_recast};
#[cfg(feature = "navmesh")]
pub use pathfinding::{
    PathRequest, PathResult, PathStatus, PathfindingAlgorithm, find_closest_point_on_navmesh,
    find_path_with_algorithm, sample_navmesh_height,
};
#[cfg(feature = "navmesh")]
pub use resources::*;
#[cfg(feature = "navmesh")]
pub use systems::run_navmesh_systems;

/// Registers the pathfinding and agent-steering pass into the frame
/// schedule's update phase, before transform propagation picks up the
/// agents' writes.
#[cfg(feature = "navmesh")]
pub fn install(world: &mut crate::ecs::world::World) {
    use crate::schedule::{FramePhase, schedule_push};
    schedule_push(
        &mut world.resources.schedules.frame,
        FramePhase::Update,
        systems::run_navmesh_systems,
    );
}

/// Installs navmesh pathfinding through [`install`]. Without this plugin
/// the frame schedule carries no navigation work and agents stand still.
#[cfg(feature = "navmesh")]
pub struct NavmeshPlugin;

#[cfg(feature = "navmesh")]
impl crate::app::Plugin for NavmeshPlugin {
    fn build(&self, app: &mut crate::app::App) {
        app.add_system(crate::app::Stage::Startup, install);
    }
}
