struct AimConstraint {
    joint_start: u32,
    joint_count: u32,
    descendant_start: u32,
    descendant_count: u32,
    goal: vec4<f32>,
    forward: vec4<f32>,
    weight: f32,
    cone_angle: f32,
    pad0: u32,
    pad1: u32,
}

@group(0) @binding(0) var<storage, read_write> bone_transforms: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> constraints: array<AimConstraint>;
@group(0) @binding(2) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(3) var<storage, read> tip_descendants: array<u32>;

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

fn quat_weight(q: vec4<f32>, weight: f32) -> vec4<f32> {
    var b = q;
    if (b.w < 0.0) {
        b = -b;
    }
    let identity = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let mixed = mix(identity, b, weight);
    let length_squared = dot(mixed, mixed);
    if (length_squared < 0.000001) {
        return identity;
    }
    return mixed / sqrt(length_squared);
}

fn clamp_quat_angle(q: vec4<f32>, max_angle: f32) -> vec4<f32> {
    var b = q;
    if (b.w < 0.0) {
        b = -b;
    }
    let angle = 2.0 * acos(clamp(b.w, -1.0, 1.0));
    if (angle <= max_angle || angle < 0.00001) {
        return b;
    }
    let axis_length = length(b.xyz);
    if (axis_length < 0.00001) {
        return b;
    }
    return quat_from_axis_angle(b.xyz / axis_length, max_angle);
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
    let constraint_index = global_id.x;
    if (constraint_index >= arrayLength(&constraints)) {
        return;
    }
    let constraint = constraints[constraint_index];
    if (constraint.weight <= 0.0 || constraint.joint_count == 0u) {
        return;
    }
    let forward = normalize(constraint.forward.xyz);

    for (var i = 0u; i < constraint.joint_count; i = i + 1u) {
        let bone = joint_indices[constraint.joint_start + i];
        let m = bone_transforms[bone];
        let bone_position = m[3].xyz;
        let world_forward = (m * vec4<f32>(forward, 0.0)).xyz;
        let to_target = constraint.goal.xyz - bone_position;
        if (length(to_target) < 0.00001 || length(world_forward) < 0.00001) {
            continue;
        }
        var rotation = quat_from_to(world_forward, to_target);
        let fraction = constraint.weight * f32(i + 1u) / f32(constraint.joint_count);
        rotation = quat_weight(rotation, fraction);
        rotation = clamp_quat_angle(rotation, constraint.cone_angle);
        for (var k = i; k < constraint.joint_count; k = k + 1u) {
            rotate_bone_about(joint_indices[constraint.joint_start + k], bone_position, rotation);
        }
        for (var descendant = 0u; descendant < constraint.descendant_count; descendant = descendant + 1u) {
            rotate_bone_about(
                tip_descendants[constraint.descendant_start + descendant],
                bone_position,
                rotation,
            );
        }
    }
}
