struct PickOutput {
    depth: f32,
    entity_id: u32,
};

@group(0) @binding(0) var depth_texture: texture_depth_2d;
@group(0) @binding(1) var<storage, read_write> output: array<PickOutput>;
@group(0) @binding(2) var<uniform> params: PickParams;
@group(0) @binding(3) var entity_id_texture: texture_2d<f32>;

struct PickParams {
    center_x: u32,
    center_y: u32,
    sample_size: u32,
    _padding: u32,
}

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let half_size = i32(params.sample_size / 2u);
    let px = i32(params.center_x) + i32(global_id.x) - half_size;
    let py = i32(params.center_y) + i32(global_id.y) - half_size;

    let dims = vec2<i32>(textureDimensions(depth_texture));

    var depth_value: f32 = 0.0;
    var entity_id_value: u32 = 0u;
    if px >= 0 && py >= 0 && px < dims.x && py < dims.y {
        let coord = vec2<u32>(u32(px), u32(py));
        depth_value = textureLoad(depth_texture, coord, 0);
        let entity_id_float = textureLoad(entity_id_texture, coord, 0).r;
        entity_id_value = bitcast<u32>(entity_id_float);
    }

    let index = global_id.y * params.sample_size + global_id.x;
    output[index].depth = depth_value;
    output[index].entity_id = entity_id_value;
}
