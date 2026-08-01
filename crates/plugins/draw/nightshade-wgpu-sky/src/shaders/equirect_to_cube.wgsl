@group(0) @binding(0)
var equirect_texture: texture_2d<f32>;

@group(0) @binding(1)
var equirect_sampler: sampler;

@group(0) @binding(2)
var output_texture: texture_storage_2d_array<rgba16float, write>;

const PI: f32 = 3.141592653589793;
const FACE_SIZE: u32 = 1024u;

fn cube_to_world(face: u32, uv: vec2<f32>) -> vec3<f32> {
    var dir: vec3<f32>;
    let x = 2.0 * uv.x - 1.0;
    let y = 2.0 * uv.y - 1.0;

    switch face {
        case 0u: {
            dir = vec3<f32>(1.0, -y, -x);
        }
        case 1u: {
            dir = vec3<f32>(-1.0, -y, x);
        }
        case 2u: {
            dir = vec3<f32>(x, 1.0, y);
        }
        case 3u: {
            dir = vec3<f32>(x, -1.0, -y);
        }
        case 4u: {
            dir = vec3<f32>(x, -y, 1.0);
        }
        default: {
            dir = vec3<f32>(-x, -y, -1.0);
        }
    }
    return normalize(dir);
}

fn world_to_equirect(dir: vec3<f32>) -> vec2<f32> {
    let phi = atan2(dir.z, dir.x);
    let theta = asin(dir.y);

    var uv = vec2<f32>(phi / (2.0 * PI), theta / PI);
    uv.x = uv.x + 0.5;
    uv.y = 0.5 - uv.y;
    return uv;
}

@compute @workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    let face = group_id.z;
    if face >= 6u {
        return;
    }

    let coords = vec2<u32>(global_id.xy);
    if coords.x >= FACE_SIZE || coords.y >= FACE_SIZE {
        return;
    }

    let uv = (vec2<f32>(coords) + 0.5) / f32(FACE_SIZE);

    let dir = cube_to_world(face, uv);

    let equirect_uv = world_to_equirect(dir);

    var color = textureSampleLevel(
        equirect_texture,
        equirect_sampler,
        equirect_uv,
        0.0
    );
    color = vec4<f32>(clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(65000.0)), color.a);

    textureStore(
        output_texture,
        coords,
        face,
        color
    );
}
