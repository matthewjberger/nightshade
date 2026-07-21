struct AnimBone {
    output_index: u32,
    parent_output_index: i32,
    pose_start: u32,
    pose_count: u32,
    layer_start: u32,
    layer_count: u32,
    flags: u32,
    pad0: u32,
    rest_translation: vec4<f32>,
    rest_rotation: vec4<f32>,
    rest_scale: vec4<f32>,
}

struct AnimClip {
    channel_start: u32,
    channel_count: u32,
    runtime_index: u32,
    pad0: u32,
}

struct AnimChannel {
    property: u32,
    interpolation: u32,
    input_offset: u32,
    key_count: u32,
    output_offset: u32,
    output_stride: u32,
    pad0: u32,
    pad1: u32,
}

struct AnimSkeleton {
    bone_start: u32,
    bone_count: u32,
    pad0: u32,
    pad1: u32,
    armature_root: mat4x4<f32>,
}

@group(0) @binding(0) var<storage, read> skeletons: array<AnimSkeleton>;
@group(0) @binding(1) var<storage, read> bones: array<AnimBone>;
@group(0) @binding(2) var<storage, read> channels: array<AnimChannel>;
@group(0) @binding(3) var<storage, read> times: array<f32>;
@group(0) @binding(4) var<storage, read> values: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> bone_transforms: array<mat4x4<f32>>;
// Pose clips then layer clips per bone, flattened; each references a clip_runtime slot.
@group(0) @binding(6) var<storage, read> bone_clips: array<AnimClip>;
// Per clip runtime state: x is playback time, y is blend weight.
@group(0) @binding(7) var<storage, read> clip_runtime: array<vec2<f32>>;

const PROPERTY_TRANSLATION: u32 = 0u;
const PROPERTY_ROTATION: u32 = 1u;
const PROPERTY_SCALE: u32 = 2u;

const INTERP_LINEAR: u32 = 0u;
const INTERP_STEP: u32 = 1u;
const INTERP_CUBIC: u32 = 2u;

fn find_key(input_offset: u32, key_count: u32, time: f32) -> u32 {
    // Largest index whose keyframe time is <= time, clamped to [0, key_count - 2].
    if (key_count <= 1u) {
        return 0u;
    }
    var low: u32 = 0u;
    var high: u32 = key_count - 1u;
    while (low < high) {
        let mid = (low + high + 1u) / 2u;
        if (times[input_offset + mid] <= time) {
            low = mid;
        } else {
            high = mid - 1u;
        }
    }
    if (low > key_count - 2u) {
        return key_count - 2u;
    }
    return low;
}

fn quat_normalize(q: vec4<f32>) -> vec4<f32> {
    let length_squared = dot(q, q);
    if (length_squared <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return q / sqrt(length_squared);
}

fn quat_slerp(a_in: vec4<f32>, b_in: vec4<f32>, t: f32) -> vec4<f32> {
    let a = quat_normalize(a_in);
    var b = quat_normalize(b_in);
    var cos_theta = dot(a, b);
    if (cos_theta < 0.0) {
        b = -b;
        cos_theta = -cos_theta;
    }
    if (cos_theta > 0.9995) {
        return quat_normalize(a + (b - a) * t);
    }
    let theta = acos(clamp(cos_theta, -1.0, 1.0));
    let sin_theta = sin(theta);
    let wa = sin((1.0 - t) * theta) / sin_theta;
    let wb = sin(t * theta) / sin_theta;
    return quat_normalize(a * wa + b * wb);
}

fn sample_vec3(channel: AnimChannel, time: f32, fallback: vec4<f32>) -> vec4<f32> {
    let count = channel.key_count;
    if (count == 0u) {
        return fallback;
    }
    let stride = channel.output_stride;
    if (count == 1u) {
        let center = select(0u, 1u, stride == 3u);
        return values[channel.output_offset + center];
    }
    let first_time = times[channel.input_offset];
    let last_time = times[channel.input_offset + count - 1u];
    if (time <= first_time) {
        let center = select(0u, 1u, stride == 3u);
        return values[channel.output_offset + center];
    }
    if (time >= last_time) {
        let base = (count - 1u) * stride;
        let center = select(base, base + 1u, stride == 3u);
        return values[channel.output_offset + center];
    }
    let key = find_key(channel.input_offset, count, time);
    let key_time = times[channel.input_offset + key];
    let next_time = times[channel.input_offset + key + 1u];
    let dt = next_time - key_time;
    let local_t = (time - key_time) / dt;

    if (channel.interpolation == INTERP_STEP) {
        let center = select(key, key * 3u + 1u, stride == 3u);
        return values[channel.output_offset + center];
    }
    if (channel.interpolation == INTERP_CUBIC) {
        let base0 = channel.output_offset + key * 3u;
        let base1 = channel.output_offset + (key + 1u) * 3u;
        let p0 = values[base0 + 1u];
        let m0 = values[base0 + 2u] * dt;
        let p1 = values[base1 + 1u];
        let m1 = values[base1] * dt;
        let t2 = local_t * local_t;
        let t3 = t2 * local_t;
        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + local_t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;
        return p0 * h00 + m0 * h10 + p1 * h01 + m1 * h11;
    }
    let a = values[channel.output_offset + key];
    let b = values[channel.output_offset + key + 1u];
    return mix(a, b, local_t);
}

fn sample_quat(channel: AnimChannel, time: f32, fallback: vec4<f32>) -> vec4<f32> {
    let count = channel.key_count;
    if (count == 0u) {
        return fallback;
    }
    let stride = channel.output_stride;
    if (count == 1u) {
        let center = select(0u, 1u, stride == 3u);
        return quat_normalize(values[channel.output_offset + center]);
    }
    let first_time = times[channel.input_offset];
    let last_time = times[channel.input_offset + count - 1u];
    if (time <= first_time) {
        let center = select(0u, 1u, stride == 3u);
        return quat_normalize(values[channel.output_offset + center]);
    }
    if (time >= last_time) {
        let base = (count - 1u) * stride;
        let center = select(base, base + 1u, stride == 3u);
        return quat_normalize(values[channel.output_offset + center]);
    }
    let key = find_key(channel.input_offset, count, time);
    let key_time = times[channel.input_offset + key];
    let next_time = times[channel.input_offset + key + 1u];
    let dt = next_time - key_time;
    let local_t = (time - key_time) / dt;

    if (channel.interpolation == INTERP_STEP) {
        let center = select(key, key * 3u + 1u, stride == 3u);
        return quat_normalize(values[channel.output_offset + center]);
    }
    if (channel.interpolation == INTERP_CUBIC) {
        let base0 = channel.output_offset + key * 3u;
        let base1 = channel.output_offset + (key + 1u) * 3u;
        let p0 = values[base0 + 1u];
        let m0 = values[base0 + 2u] * dt;
        let p1 = values[base1 + 1u];
        let m1 = values[base1] * dt;
        let t2 = local_t * local_t;
        let t3 = t2 * local_t;
        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + local_t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;
        return quat_normalize(p0 * h00 + m0 * h10 + p1 * h01 + m1 * h11);
    }
    let a = values[channel.output_offset + key];
    let b = values[channel.output_offset + key + 1u];
    return quat_slerp(a, b, local_t);
}

fn sample_bone_local(
    bone: AnimBone,
    channel_start: u32,
    channel_count: u32,
    time: f32,
) -> mat4x4<f32> {
    var translation = bone.rest_translation;
    var rotation = bone.rest_rotation;
    var scale = bone.rest_scale;

    for (var index = 0u; index < channel_count; index = index + 1u) {
        let channel = channels[channel_start + index];
        if (channel.property == PROPERTY_TRANSLATION) {
            translation = sample_vec3(channel, time, bone.rest_translation);
        } else if (channel.property == PROPERTY_ROTATION) {
            rotation = sample_quat(channel, time, bone.rest_rotation);
        } else if (channel.property == PROPERTY_SCALE) {
            scale = sample_vec3(channel, time, bone.rest_scale);
        }
    }

    return compose_matrix(translation.xyz, quat_normalize(rotation), scale.xyz);
}

fn compose_matrix(translation: vec3<f32>, q: vec4<f32>, scale: vec3<f32>) -> mat4x4<f32> {
    let x = q.x;
    let y = q.y;
    let z = q.z;
    let w = q.w;
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;

    let column0 = vec4<f32>(
        (1.0 - 2.0 * (yy + zz)) * scale.x,
        (2.0 * (xy + wz)) * scale.x,
        (2.0 * (xz - wy)) * scale.x,
        0.0,
    );
    let column1 = vec4<f32>(
        (2.0 * (xy - wz)) * scale.y,
        (1.0 - 2.0 * (xx + zz)) * scale.y,
        (2.0 * (yz + wx)) * scale.y,
        0.0,
    );
    let column2 = vec4<f32>(
        (2.0 * (xz + wy)) * scale.z,
        (2.0 * (yz - wx)) * scale.z,
        (1.0 - 2.0 * (xx + yy)) * scale.z,
        0.0,
    );
    let column3 = vec4<f32>(translation, 1.0);
    return mat4x4<f32>(column0, column1, column2, column3);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let skeleton_index = global_id.x;
    if (skeleton_index >= arrayLength(&skeletons)) {
        return;
    }
    let skeleton = skeletons[skeleton_index];

    for (var local = 0u; local < skeleton.bone_count; local = local + 1u) {
        let bone = bones[skeleton.bone_start + local];

        var local_matrix = compose_matrix(
            bone.rest_translation.xyz,
            quat_normalize(bone.rest_rotation),
            bone.rest_scale.xyz,
        );

        var running_weight = 0.0;
        for (var pose_index = 0u; pose_index < bone.pose_count; pose_index = pose_index + 1u) {
            let clip = bone_clips[bone.pose_start + pose_index];
            let runtime = clip_runtime[clip.runtime_index];
            let weight = runtime.y;
            if (weight <= 0.0) {
                continue;
            }
            let clip_matrix = sample_bone_local(
                bone,
                clip.channel_start,
                clip.channel_count,
                runtime.x,
            );
            let factor = weight / (running_weight + weight);
            local_matrix = blend_local(local_matrix, clip_matrix, factor);
            running_weight = running_weight + weight;
        }

        if ((bone.flags & 1u) != 0u) {
            local_matrix[3] = vec4<f32>(bone.rest_translation.xyz, 1.0);
        }

        for (var layer_index = 0u; layer_index < bone.layer_count; layer_index = layer_index + 1u) {
            let layer = bone_clips[bone.layer_start + layer_index];
            if (layer.channel_count == 0u) {
                continue;
            }
            let runtime = clip_runtime[layer.runtime_index];
            if (runtime.y <= 0.0) {
                continue;
            }
            let layer_matrix = sample_bone_local(
                bone,
                layer.channel_start,
                layer.channel_count,
                runtime.x,
            );
            local_matrix = blend_local(local_matrix, layer_matrix, clamp(runtime.y, 0.0, 1.0));
        }

        var parent_world = skeleton.armature_root;
        if (bone.parent_output_index >= 0) {
            parent_world = bone_transforms[u32(bone.parent_output_index)];
        }
        bone_transforms[bone.output_index] = parent_world * local_matrix;
    }
}

fn blend_local(from_matrix: mat4x4<f32>, to_matrix: mat4x4<f32>, factor: f32) -> mat4x4<f32> {
    let from_t = from_matrix[3].xyz;
    let to_t = to_matrix[3].xyz;
    let translation = mix(from_t, to_t, factor);

    let from_scale = vec3<f32>(
        length(from_matrix[0].xyz),
        length(from_matrix[1].xyz),
        length(from_matrix[2].xyz),
    );
    let to_scale = vec3<f32>(
        length(to_matrix[0].xyz),
        length(to_matrix[1].xyz),
        length(to_matrix[2].xyz),
    );
    let scale = mix(from_scale, to_scale, factor);

    let from_q = matrix_to_quat(from_matrix, from_scale);
    let to_q = matrix_to_quat(to_matrix, to_scale);
    let rotation = quat_slerp(from_q, to_q, factor);

    return compose_matrix(translation, rotation, scale);
}

fn matrix_to_quat(m: mat4x4<f32>, scale: vec3<f32>) -> vec4<f32> {
    let c0 = m[0].xyz / max(scale.x, 1e-8);
    let c1 = m[1].xyz / max(scale.y, 1e-8);
    let c2 = m[2].xyz / max(scale.z, 1e-8);
    let trace = c0.x + c1.y + c2.z;
    if (trace > 0.0) {
        let s = sqrt(trace + 1.0) * 2.0;
        return quat_normalize(vec4<f32>(
            (c1.z - c2.y) / s,
            (c2.x - c0.z) / s,
            (c0.y - c1.x) / s,
            0.25 * s,
        ));
    } else if (c0.x > c1.y && c0.x > c2.z) {
        let s = sqrt(1.0 + c0.x - c1.y - c2.z) * 2.0;
        return quat_normalize(vec4<f32>(
            0.25 * s,
            (c1.x + c0.y) / s,
            (c2.x + c0.z) / s,
            (c1.z - c2.y) / s,
        ));
    } else if (c1.y > c2.z) {
        let s = sqrt(1.0 + c1.y - c0.x - c2.z) * 2.0;
        return quat_normalize(vec4<f32>(
            (c1.x + c0.y) / s,
            0.25 * s,
            (c2.y + c1.z) / s,
            (c2.x - c0.z) / s,
        ));
    } else {
        let s = sqrt(1.0 + c2.z - c0.x - c1.y) * 2.0;
        return quat_normalize(vec4<f32>(
            (c2.x + c0.z) / s,
            (c2.y + c1.z) / s,
            0.25 * s,
            (c0.y - c1.x) / s,
        ));
    }
}
