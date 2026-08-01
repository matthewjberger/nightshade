
struct Uniforms {
    step: vec4<f32>,
    weights: vec4<f32>,
    counts: vec4<u32>,
    search: vec4<f32>,
};

@group(0) @binding(0) var<storage, read> positions_in: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> velocities_in: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> positions_out: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> velocities_out: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> cell_counts: array<atomic<u32>>;
@group(0) @binding(5) var<storage, read_write> cell_items: array<u32>;
@group(0) @binding(6) var<uniform> uniforms: Uniforms;

fn cell_axis(value: f32) -> u32 {
    let bounds = uniforms.step.y;
    let cell_size = uniforms.step.z;
    let dimension = i32(uniforms.counts.y);
    let cell = i32(floor((value + bounds) / cell_size));
    return u32(clamp(cell, 0, dimension - 1));
}

fn cell_of(position: vec3<f32>) -> u32 {
    let dimension = uniforms.counts.y;
    let x = cell_axis(position.x);
    let y = cell_axis(position.y);
    let z = cell_axis(position.z);
    return (z * dimension + y) * dimension + x;
}

@compute @workgroup_size(256)
fn clear_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= uniforms.counts.z {
        return;
    }
    atomicStore(&cell_counts[id.x], 0u);
}

@compute @workgroup_size(256)
fn fill_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= uniforms.counts.x {
        return;
    }

    let cell = cell_of(positions_in[id.x].xyz);
    let slot = atomicAdd(&cell_counts[cell], 1u);
    let capacity = uniforms.counts.w;
    if slot < capacity {
        cell_items[cell * capacity + slot] = id.x;
    }
}

struct Neighborhood {
    pushed: vec3<f32>,
    heading: vec3<f32>,
    middle: vec3<f32>,
    found: f32,
};

fn survey(index: u32, position: vec3<f32>) -> Neighborhood {
    var found = Neighborhood(vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0), 0.0);

    let dimension = i32(uniforms.counts.y);
    let capacity = uniforms.counts.w;
    let radius_squared = uniforms.search.y;

    let base_x = i32(cell_axis(position.x));
    let base_y = i32(cell_axis(position.y));
    let base_z = i32(cell_axis(position.z));

    for (var offset_z = -1; offset_z <= 1; offset_z++) {
        let z = base_z + offset_z;
        if z < 0 || z >= dimension { continue; }
        for (var offset_y = -1; offset_y <= 1; offset_y++) {
            let y = base_y + offset_y;
            if y < 0 || y >= dimension { continue; }
            for (var offset_x = -1; offset_x <= 1; offset_x++) {
                let x = base_x + offset_x;
                if x < 0 || x >= dimension { continue; }

                let cell = (u32(z) * u32(dimension) + u32(y)) * u32(dimension) + u32(x);
                let held = min(atomicLoad(&cell_counts[cell]), capacity);
                for (var slot = 0u; slot < held; slot++) {
                    let other = cell_items[cell * capacity + slot];
                    if other == index { continue; }

                    let offset = positions_in[other].xyz - position;
                    let distance_squared = dot(offset, offset);
                    if distance_squared >= radius_squared || distance_squared <= 0.0 {
                        continue;
                    }

                    found.pushed -= offset / distance_squared;
                    found.heading += velocities_in[other].xyz;
                    found.middle += positions_in[other].xyz;
                    found.found += 1.0;
                }
            }
        }
    }

    return found;
}

fn turn_from_walls(position: vec3<f32>) -> vec3<f32> {
    let bounds = uniforms.step.y;
    let strength = uniforms.weights.w;
    let past = abs(position) - bounds;
    let pull = -sign(position) * max(past, vec3<f32>(0.0));
    return pull * strength;
}

@compute @workgroup_size(256)
fn steer(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if index >= uniforms.counts.x {
        return;
    }

    let position = positions_in[index].xyz;
    var velocity = velocities_in[index].xyz;

    let found = survey(index, position);
    if found.found > 0.0 {
        let middle = found.middle / found.found;
        let heading = found.heading / found.found;
        velocity += found.pushed * uniforms.weights.x;
        velocity += (heading - velocity) * uniforms.weights.y;
        velocity += (middle - position) * uniforms.weights.z;
    }

    velocity += turn_from_walls(position);

    let max_speed = uniforms.step.w;
    let speed = length(velocity);
    if speed > max_speed {
        velocity = velocity * (max_speed / speed);
    } else if speed < max_speed * 0.35 && speed > 0.0 {
        velocity = velocity * (max_speed * 0.35 / speed);
    }

    positions_out[index] = vec4<f32>(position + velocity * uniforms.step.x, 1.0);
    velocities_out[index] = vec4<f32>(velocity, 0.0);
}
