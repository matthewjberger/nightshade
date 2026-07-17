struct SpdParams {
    src_size: vec2<u32>,
    mip_count: u32,
    _padding: u32,
}

@group(0) @binding(0) var src_texture: texture_depth_2d;
@group(0) @binding(1) var dst_mip0: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var dst_mip1: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var dst_mip2: texture_storage_2d<r32float, write>;
@group(0) @binding(4) var dst_mip3: texture_storage_2d<r32float, write>;
@group(0) @binding(5) var<uniform> params: SpdParams;

var<workgroup> lds: array<f32, 4096>;

fn reduce4(v0: f32, v1: f32, v2: f32, v3: f32) -> f32 {
    return min(min(v0, v1), min(v2, v3));
}

fn load_from_source(coord: vec2<u32>) -> f32 {
    let clamped = min(coord, params.src_size - vec2(1u));
    return textureLoad(src_texture, clamped, 0);
}

@compute @workgroup_size(256, 1, 1)
fn main(
    @builtin(local_invocation_index) local_index: u32,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    let tile_x = workgroup_id.x;
    let tile_y = workgroup_id.y;

    let base_x = tile_x * 64u;
    let base_y = tile_y * 64u;

    for (var load_index = local_index; load_index < 4096u; load_index += 256u) {
        let local_x = load_index % 64u;
        let local_y = load_index / 64u;
        let src_coord = vec2(base_x + local_x, base_y + local_y);
        lds[load_index] = load_from_source(src_coord);
    }
    workgroupBarrier();

    for (var store_index = local_index; store_index < 4096u; store_index += 256u) {
        let local_x = store_index % 64u;
        let local_y = store_index / 64u;
        let dst_coord = vec2(base_x + local_x, base_y + local_y);
        let mip0_size = params.src_size;
        if dst_coord.x < mip0_size.x && dst_coord.y < mip0_size.y {
            textureStore(dst_mip0, dst_coord, vec4(lds[store_index], 0.0, 0.0, 0.0));
        }
    }

    if params.mip_count <= 1u {
        return;
    }
    workgroupBarrier();

    // Reduce mip0 -> mip1 into private memory first so that no thread writes
    // to LDS while another thread is still reading the higher source indices.
    // Writing in place during the loop would race: iter 0 reads indices up to
    // ~1088 while iter 2 and iter 3 write indices 512..1024 with no barrier
    // between them.
    var mip1_reduced: array<f32, 4>;
    for (var iter = 0u; iter < 4u; iter++) {
        let reduce_index = local_index + iter * 256u;
        if reduce_index < 1024u {
            let local_x = reduce_index % 32u;
            let local_y = reduce_index / 32u;
            let src_x = local_x * 2u;
            let src_y = local_y * 2u;

            let idx00 = src_y * 64u + src_x;
            let idx10 = src_y * 64u + src_x + 1u;
            let idx01 = (src_y + 1u) * 64u + src_x;
            let idx11 = (src_y + 1u) * 64u + src_x + 1u;

            mip1_reduced[iter] = reduce4(lds[idx00], lds[idx10], lds[idx01], lds[idx11]);
        }
    }
    workgroupBarrier();
    for (var iter = 0u; iter < 4u; iter++) {
        let reduce_index = local_index + iter * 256u;
        if reduce_index < 1024u {
            lds[reduce_index] = mip1_reduced[iter];
        }
    }
    workgroupBarrier();

    for (var store_index = local_index; store_index < 1024u; store_index += 256u) {
        let local_x = store_index % 32u;
        let local_y = store_index / 32u;
        let dst_coord = vec2((base_x >> 1u) + local_x, (base_y >> 1u) + local_y);
        let mip1_size = max(vec2(1u), params.src_size >> vec2(1u));
        if dst_coord.x < mip1_size.x && dst_coord.y < mip1_size.y {
            textureStore(dst_mip1, dst_coord, vec4(lds[store_index], 0.0, 0.0, 0.0));
        }
    }

    if params.mip_count <= 2u {
        return;
    }
    workgroupBarrier();

    // Same race-free split for mip1 -> mip2: thread T reads up to lds[idx11]
    // where idx11 may be in [0, 1024), and thread S writes to lds[S] where
    // S in [0, 256), so reads of indices < 256 collide with writes from the
    // very same pass without the staging.
    var mip2_reduced: f32 = 0.0;
    if local_index < 256u {
        let local_x = local_index % 16u;
        let local_y = local_index / 16u;
        let src_x = local_x * 2u;
        let src_y = local_y * 2u;

        let idx00 = src_y * 32u + src_x;
        let idx10 = src_y * 32u + src_x + 1u;
        let idx01 = (src_y + 1u) * 32u + src_x;
        let idx11 = (src_y + 1u) * 32u + src_x + 1u;

        mip2_reduced = reduce4(lds[idx00], lds[idx10], lds[idx01], lds[idx11]);
    }
    workgroupBarrier();
    if local_index < 256u {
        lds[local_index] = mip2_reduced;
    }
    workgroupBarrier();

    if local_index < 256u {
        let local_x = local_index % 16u;
        let local_y = local_index / 16u;
        let dst_coord = vec2((base_x >> 2u) + local_x, (base_y >> 2u) + local_y);
        let mip2_size = max(vec2(1u), params.src_size >> vec2(2u));
        if dst_coord.x < mip2_size.x && dst_coord.y < mip2_size.y {
            textureStore(dst_mip2, dst_coord, vec4(lds[local_index], 0.0, 0.0, 0.0));
        }
    }

    if params.mip_count <= 3u {
        return;
    }
    workgroupBarrier();

    var mip3_reduced: f32 = 0.0;
    if local_index < 64u {
        let local_x = local_index % 8u;
        let local_y = local_index / 8u;
        let src_x = local_x * 2u;
        let src_y = local_y * 2u;

        let idx00 = src_y * 16u + src_x;
        let idx10 = src_y * 16u + src_x + 1u;
        let idx01 = (src_y + 1u) * 16u + src_x;
        let idx11 = (src_y + 1u) * 16u + src_x + 1u;

        mip3_reduced = reduce4(lds[idx00], lds[idx10], lds[idx01], lds[idx11]);
    }
    workgroupBarrier();
    if local_index < 64u {
        lds[local_index] = mip3_reduced;
    }
    workgroupBarrier();

    if local_index < 64u {
        let local_x = local_index % 8u;
        let local_y = local_index / 8u;
        let dst_coord = vec2((base_x >> 3u) + local_x, (base_y >> 3u) + local_y);
        let mip3_size = max(vec2(1u), params.src_size >> vec2(3u));
        if dst_coord.x < mip3_size.x && dst_coord.y < mip3_size.y {
            textureStore(dst_mip3, dst_coord, vec4(lds[local_index], 0.0, 0.0, 0.0));
        }
    }
}
