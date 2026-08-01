struct ModelMatrix {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
};

struct Uniforms {
    parent_transform: mat4x4<f32>,
    instance_count: u32,
    output_offset: u32,
    _pad: vec2<u32>,
};

@group(0) @binding(0) var<storage, read> local_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read_write> model_matrices: array<ModelMatrix>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

fn compute_normal_matrix(m: mat4x4<f32>) -> mat3x3<f32> {
    let m3 = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
    let sx = dot(m3[0], m3[0]);
    let sy = dot(m3[1], m3[1]);
    let sz = dot(m3[2], m3[2]);
    let is_uniform = abs(sx - sy) < 0.001 && abs(sy - sz) < 0.001;
    if is_uniform {
        let inv = inverseSqrt(sx);
        return mat3x3<f32>(m3[0] * inv, m3[1] * inv, m3[2] * inv);
    }
    let inv = inverseSqrt(vec3<f32>(sx, sy, sz));
    return mat3x3<f32>(m3[0] * inv.x, m3[1] * inv.y, m3[2] * inv.z);
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if index >= uniforms.instance_count {
        return;
    }

    let local = local_matrices[uniforms.output_offset + index];
    let world = uniforms.parent_transform * local;
    let out_idx = uniforms.output_offset + index;
    model_matrices[out_idx].model = world;
    model_matrices[out_idx].normal_matrix = compute_normal_matrix(world);
}
