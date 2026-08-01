# nightshade-navmesh

This crate bakes a navigation mesh from geometry and walks agents over it. `NavMeshWorld` holds the vertices, the triangles, the edges, the adjacency and a spatial hash, and a `NavMeshAgent` on an entity asks for a destination and is carried there. Paths come from A*, Dijkstra or greedy best first, are smoothed by the funnel, and neighboring agents steer around each other.

## Composing it

```rust
app.add_plugin(NavmeshPlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .run()
```

`NavmeshPlugin` inserts `NavMeshWorld` and `AgentNeighborhood`, and registers its systems on `Stage::FrameUpdate`: the triangle each agent stands on, the pending path requests, the neighborhood gather, the avoidance, the movement, the snap of idle agents back onto the mesh, and the debug draw.

## Using it

```rust
use nightshade_navmesh::commands::generation::{RecastNavMeshConfig, generate_navmesh_recast};
use nightshade_navmesh::commands::{NavMeshAgents, set_agent_destination, set_navmesh_debug_draw};
use nightshade_navmesh::contract::NavMeshAgent;
use nightshade_navmesh::resources::NavMeshWorld;

fn wander(mut agents: NavMeshAgents<'_, '_>, entity: Entity, destination: Vec3) {
    set_agent_destination(&mut agents, entity, destination);
}
```

`generate_navmesh_recast` takes the vertices and the triangle indices of the walkable geometry with a `RecastNavMeshConfig` and reports a `NavMeshWorld` to put in the resource. `spawn_navmesh_agent` stands a capsule up at a position with a speed, and `NavMeshAgent::new` with `with_speed`, `with_radius`, `with_height` and `with_arrival_threshold` is the builder when the game spawns the entity itself. `NavMeshAgents` is the query alias the command functions take, so `set_agent_destination`, `stop_agent` and `set_agent_speed` are called from a system that names it as a parameter. An agent's `state` reads `Idle`, `PathPending`, `Moving`, `Arrived` or `NoPath`, and `PathfindingAlgorithm` on the world picks which search runs.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-navmesh = { path = "../nightshade-navmesh", default-features = false, features = ["contract"] }
```

The contract carries `NavMeshAgent` and `NavMeshAgentState`, the component vocabulary a crate names to speak about an agent, without the pathfinding, the recast bake or the systems.
