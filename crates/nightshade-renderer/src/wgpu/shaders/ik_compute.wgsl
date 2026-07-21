struct IkChain {
    root_index: u32,
    mid_index: u32,
    tip_index: u32,
    has_pole: u32,
    goal: vec4<f32>,
    pole: vec4<f32>,
    weight: f32,
    descendant_start: u32,
    descendant_count: u32,
    pad0: u32,
}

@group(0) @binding(0) var<storage, read_write> bone_transforms: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> chains: array<IkChain>;
@group(0) @binding(2) var<storage, read> tip_descendants: array<u32>;

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

fn angle_between(u: vec3<f32>, v: vec3<f32>) -> f32 {
    let denominator = length(u) * length(v);
    if (denominator < 0.000001) {
        return 0.0;
    }
    return acos(clamp(dot(u, v) / denominator, -1.0, 1.0));
}

fn law_of_cosines(adjacent_a: f32, adjacent_b: f32, opposite: f32) -> f32 {
    let denominator = 2.0 * adjacent_a * adjacent_b;
    if (denominator < 0.000001) {
        return 0.0;
    }
    return acos(clamp(
        (adjacent_a * adjacent_a + adjacent_b * adjacent_b - opposite * opposite) / denominator,
        -1.0,
        1.0,
    ));
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

fn rotate_descendants(chain: IkChain, pivot: vec3<f32>, q: vec4<f32>) {
    for (var index = 0u; index < chain.descendant_count; index = index + 1u) {
        rotate_bone_about(tip_descendants[chain.descendant_start + index], pivot, q);
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let chain_index = global_id.x;
    if (chain_index >= arrayLength(&chains)) {
        return;
    }
    let chain = chains[chain_index];
    if (chain.weight <= 0.0) {
        return;
    }

    let a = bone_transforms[chain.root_index][3].xyz;
    let b = bone_transforms[chain.mid_index][3].xyz;
    let c = bone_transforms[chain.tip_index][3].xyz;

    let length_ab = length(b - a);
    let length_bc = length(c - b);
    if (length_ab < 0.00001 || length_bc < 0.00001) {
        return;
    }
    let reach = length_ab + length_bc;
    let to_target = chain.goal.xyz - a;
    let target_distance = clamp(length(to_target), 0.0001, reach - 0.0001);

    var bend_axis: vec3<f32>;
    if (chain.has_pole != 0u) {
        bend_axis = cross(b - a, chain.pole.xyz - a);
    } else {
        bend_axis = cross(c - a, b - a);
    }
    if (length(bend_axis) < 0.00001) {
        bend_axis = cross(to_target, vec3<f32>(0.0, 1.0, 0.0));
        if (length(bend_axis) < 0.00001) {
            bend_axis = cross(to_target, vec3<f32>(1.0, 0.0, 0.0));
        }
    }
    bend_axis = normalize(bend_axis);

    let current_root = angle_between(c - a, b - a);
    let current_mid = angle_between(a - b, c - b);
    let target_root = law_of_cosines(length_ab, target_distance, length_bc);
    let target_mid = law_of_cosines(length_ab, length_bc, target_distance);

    let root_delta = quat_weight(
        quat_from_axis_angle(bend_axis, target_root - current_root),
        chain.weight,
    );
    let mid_delta = quat_weight(
        quat_from_axis_angle(bend_axis, target_mid - current_mid),
        chain.weight,
    );

    rotate_bone_about(chain.root_index, a, root_delta);
    rotate_bone_about(chain.mid_index, a, root_delta);
    rotate_bone_about(chain.tip_index, a, root_delta);
    rotate_descendants(chain, a, root_delta);

    let mid_pivot = bone_transforms[chain.mid_index][3].xyz;
    rotate_bone_about(chain.mid_index, mid_pivot, mid_delta);
    rotate_bone_about(chain.tip_index, mid_pivot, mid_delta);
    rotate_descendants(chain, mid_pivot, mid_delta);

    let root_pivot = bone_transforms[chain.root_index][3].xyz;
    let tip_now = bone_transforms[chain.tip_index][3].xyz;
    let aim = quat_weight(
        quat_from_to(tip_now - root_pivot, chain.goal.xyz - root_pivot),
        chain.weight,
    );
    rotate_bone_about(chain.root_index, root_pivot, aim);
    rotate_bone_about(chain.mid_index, root_pivot, aim);
    rotate_bone_about(chain.tip_index, root_pivot, aim);
    rotate_descendants(chain, root_pivot, aim);
}
