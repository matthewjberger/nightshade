struct SpringChain {
    joint_start: u32,
    joint_count: u32,
    descendant_start: u32,
    descendant_count: u32,
    state_start: u32,
    reset: u32,
    enabled: u32,
    pad0: u32,
    gravity: vec4<f32>,
    stiffness: f32,
    damping: f32,
    dt: f32,
    pad1: f32,
}

@group(0) @binding(0) var<storage, read_write> bone_transforms: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> chains: array<SpringChain>;
@group(0) @binding(2) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(3) var<storage, read> tip_descendants: array<u32>;
@group(0) @binding(4) var<storage, read> rest_axes: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> bone_lengths: array<f32>;
@group(0) @binding(6) var<storage, read_write> spring_state: array<vec4<f32>>;

fn quat_from_axis_angle(axis: vec3<f32>, angle: f32) -> vec4<f32> {
    let half = angle * 0.5;
    let s = sin(half);
    return vec4<f32>(axis.x * s, axis.y * s, axis.z * s, cos(half));
}

fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let u = q.xyz;
    return v + 2.0 * cross(u, cross(u, v) + q.w * v);
}

fn quat_to_mat3(q: vec4<f32>) -> mat3x3<f32> {
    let x = q.x;
    let y = q.y;
    let z = q.z;
    let w = q.w;
    return mat3x3<f32>(
        vec3<f32>(1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + w * z), 2.0 * (x * z - w * y)),
        vec3<f32>(2.0 * (x * y - w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + w * x)),
        vec3<f32>(2.0 * (x * z + w * y), 2.0 * (y * z - w * x), 1.0 - 2.0 * (x * x + y * y)),
    );
}

fn quat_from_to(from_direction: vec3<f32>, to_direction: vec3<f32>) -> vec4<f32> {
    let source = normalize(from_direction);
    let destination = normalize(to_direction);
    let cos_angle = dot(source, destination);
    if (cos_angle > 0.99999) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    if (cos_angle < -0.99999) {
        var axis = cross(vec3<f32>(1.0, 0.0, 0.0), source);
        if (length(axis) < 0.0001) {
            axis = cross(vec3<f32>(0.0, 1.0, 0.0), source);
        }
        return vec4<f32>(normalize(axis), 0.0);
    }
    let axis = normalize(cross(source, destination));
    let angle = acos(clamp(cos_angle, -1.0, 1.0));
    return quat_from_axis_angle(axis, angle);
}

fn rotate_bone_about(index: u32, pivot: vec3<f32>, q: vec4<f32>) {
    let m = bone_transforms[index];
    let position = m[3].xyz;
    let new_position = pivot + quat_rotate(q, position - pivot);
    let rotation = quat_to_mat3(q);
    let column0 = rotation * m[0].xyz;
    let column1 = rotation * m[1].xyz;
    let column2 = rotation * m[2].xyz;
    bone_transforms[index] = mat4x4<f32>(
        vec4<f32>(column0, 0.0),
        vec4<f32>(column1, 0.0),
        vec4<f32>(column2, 0.0),
        vec4<f32>(new_position, 1.0),
    );
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let chain_index = global_id.x;
    if (chain_index >= arrayLength(&chains)) {
        return;
    }
    let chain = chains[chain_index];
    if (chain.enabled == 0u || chain.joint_count < 2u) {
        return;
    }

    for (var i = 1u; i < chain.joint_count; i = i + 1u) {
        let joint = joint_indices[chain.joint_start + i];
        let parent = joint_indices[chain.joint_start + i - 1u];
        let parent_matrix = bone_transforms[parent];
        let parent_position = parent_matrix[3].xyz;
        let bone_length = bone_lengths[chain.joint_start + i];
        let rest_axis = rest_axes[chain.joint_start + i].xyz;

        let px = normalize(parent_matrix[0].xyz);
        let py = normalize(parent_matrix[1].xyz);
        let pz = normalize(parent_matrix[2].xyz);
        let rest_goal = parent_position
            + (px * rest_axis.x + py * rest_axis.y + pz * rest_axis.z) * bone_length;

        let slot = 2u * (chain.state_start + i);
        var tip = spring_state[slot].xyz;
        var prev = spring_state[slot + 1u].xyz;
        let current_joint_position = bone_transforms[joint][3].xyz;

        if (chain.reset == 1u || distance(tip, rest_goal) > bone_length * 8.0 + 0.001) {
            tip = rest_goal;
            prev = rest_goal;
        } else {
            let inertia = (tip - prev) * (1.0 - chain.damping);
            var next = tip + inertia + chain.gravity.xyz * chain.dt;
            next = next + (rest_goal - next) * chain.stiffness;
            let direction = next - parent_position;
            let direction_length = length(direction);
            if (direction_length > 0.00001) {
                next = parent_position + direction / direction_length * bone_length;
            } else {
                next = rest_goal;
            }
            prev = tip;
            tip = next;
        }

        spring_state[slot] = vec4<f32>(tip, 0.0);
        spring_state[slot + 1u] = vec4<f32>(prev, 0.0);

        let current_direction = current_joint_position - parent_position;
        let new_direction = tip - parent_position;
        if (length(current_direction) > 0.00001 && length(new_direction) > 0.00001) {
            let rotation = quat_from_to(current_direction, new_direction);
            for (var k = i - 1u; k < chain.joint_count; k = k + 1u) {
                rotate_bone_about(joint_indices[chain.joint_start + k], parent_position, rotation);
            }
            for (var descendant = 0u; descendant < chain.descendant_count; descendant = descendant + 1u) {
                rotate_bone_about(
                    tip_descendants[chain.descendant_start + descendant],
                    parent_position,
                    rotation,
                );
            }
        }
    }
}
