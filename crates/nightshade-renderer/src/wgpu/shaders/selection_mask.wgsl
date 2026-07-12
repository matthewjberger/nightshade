struct Uniforms {
    bit_word_count: u32,
    _padding_0: u32,
    _padding_1: u32,
    _padding_2: u32,
};

@group(0) @binding(0) var entity_id_texture: texture_2d<f32>;
@group(0) @binding(1) var<storage, read> selection_bits: array<u32>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) f32 {
    let bit_word_count = uniforms.bit_word_count;
    if bit_word_count == 0u {
        return 0.0;
    }
    let coord = vec2<i32>(i32(frag_coord.x), i32(frag_coord.y));
    let entity_id_float = textureLoad(entity_id_texture, coord, 0).r;
    let entity_id = bitcast<u32>(entity_id_float);
    if entity_id == 0u {
        return 0.0;
    }
    let word_idx = entity_id / 32u;
    if word_idx >= bit_word_count {
        return 0.0;
    }
    let bit_idx = entity_id & 31u;
    let bit = (selection_bits[word_idx] >> bit_idx) & 1u;
    if bit == 1u {
        return 1.0;
    }
    return 0.0;
}
