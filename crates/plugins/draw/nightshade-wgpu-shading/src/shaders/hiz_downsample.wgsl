struct HizParams {
    src_mip_size: vec2<u32>,
    dst_mip_size: vec2<u32>,
}

@group(0) @binding(0) var src_mip: texture_2d<f32>;
@group(0) @binding(1) var dst_mip: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var<uniform> params: HizParams;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.dst_mip_size.x || id.y >= params.dst_mip_size.y {
        return;
    }

    let src_width = params.src_mip_size.x;
    let src_height = params.src_mip_size.y;

    var depth: f32;

    if params.src_mip_size.x == params.dst_mip_size.x && params.src_mip_size.y == params.dst_mip_size.y {
        depth = textureLoad(src_mip, id.xy, 0).r;
    } else {
        let src_texel = id.xy * 2u;

        let d00 = textureLoad(src_mip, min(src_texel + vec2(0u, 0u), vec2(src_width - 1u, src_height - 1u)), 0).r;
        let d10 = textureLoad(src_mip, min(src_texel + vec2(1u, 0u), vec2(src_width - 1u, src_height - 1u)), 0).r;
        let d01 = textureLoad(src_mip, min(src_texel + vec2(0u, 1u), vec2(src_width - 1u, src_height - 1u)), 0).r;
        let d11 = textureLoad(src_mip, min(src_texel + vec2(1u, 1u), vec2(src_width - 1u, src_height - 1u)), 0).r;

        depth = min(min(d00, d10), min(d01, d11));
    }

    textureStore(dst_mip, id.xy, vec4(depth, 0.0, 0.0, 0.0));
}
