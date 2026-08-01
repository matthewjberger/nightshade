
struct Fxaa {
    texel: vec2<f32>,
    relative_threshold: f32,
    absolute_threshold: f32,
    subpixel: f32,
    pad_one: f32,
    pad_two: f32,
    pad_three: f32,
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> fxaa: Fxaa;

const STEPS: i32 = 8;

fn reach_at(step: i32) -> f32 {
    if step < 4 {
        return 1.0;
    }
    if step < 6 {
        return 2.0;
    }
    return 4.0;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);
    out.uv = vec2<f32>(x, y);
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

fn luma(color: vec3<f32>) -> f32 {
    return sqrt(dot(color, vec3<f32>(0.299, 0.587, 0.114)));
}

fn color_at(uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(source, source_sampler, uv, 0.0);
}

fn luma_at(uv: vec2<f32>) -> f32 {
    return luma(color_at(uv).rgb);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = fxaa.texel;
    let middle = color_at(in.uv);
    let luma_middle = luma(middle.rgb);

    let luma_down = luma_at(in.uv + vec2<f32>(0.0, texel.y));
    let luma_up = luma_at(in.uv + vec2<f32>(0.0, -texel.y));
    let luma_right = luma_at(in.uv + vec2<f32>(texel.x, 0.0));
    let luma_left = luma_at(in.uv + vec2<f32>(-texel.x, 0.0));

    let luma_lowest = min(luma_middle, min(min(luma_down, luma_up), min(luma_right, luma_left)));
    let luma_highest = max(luma_middle, max(max(luma_down, luma_up), max(luma_right, luma_left)));
    let range = luma_highest - luma_lowest;

    if range < max(fxaa.absolute_threshold, luma_highest * fxaa.relative_threshold) {
        return middle;
    }

    let luma_down_left = luma_at(in.uv + vec2<f32>(-texel.x, texel.y));
    let luma_up_right = luma_at(in.uv + vec2<f32>(texel.x, -texel.y));
    let luma_up_left = luma_at(in.uv + vec2<f32>(-texel.x, -texel.y));
    let luma_down_right = luma_at(in.uv + vec2<f32>(texel.x, texel.y));

    let luma_down_up = luma_down + luma_up;
    let luma_left_right = luma_left + luma_right;
    let luma_left_corners = luma_down_left + luma_up_left;
    let luma_down_corners = luma_down_left + luma_down_right;
    let luma_right_corners = luma_down_right + luma_up_right;
    let luma_up_corners = luma_up_right + luma_up_left;

    let edge_horizontal = abs(-2.0 * luma_left + luma_left_corners)
        + abs(-2.0 * luma_middle + luma_down_up) * 2.0
        + abs(-2.0 * luma_right + luma_right_corners);
    let edge_vertical = abs(-2.0 * luma_up + luma_up_corners)
        + abs(-2.0 * luma_middle + luma_left_right) * 2.0
        + abs(-2.0 * luma_down + luma_down_corners);
    let horizontal = edge_horizontal >= edge_vertical;

    let luma_side_one = select(luma_left, luma_up, horizontal);
    let luma_side_two = select(luma_right, luma_down, horizontal);
    let gradient_one = luma_side_one - luma_middle;
    let gradient_two = luma_side_two - luma_middle;
    let steeper_is_one = abs(gradient_one) >= abs(gradient_two);
    let gradient_scaled = 0.25 * max(abs(gradient_one), abs(gradient_two));

    var step_length = select(texel.x, texel.y, horizontal);
    var luma_average = 0.0;
    if steeper_is_one {
        step_length = -step_length;
        luma_average = 0.5 * (luma_side_one + luma_middle);
    } else {
        luma_average = 0.5 * (luma_side_two + luma_middle);
    }

    var middle_uv = in.uv;
    if horizontal {
        middle_uv.y = middle_uv.y + step_length * 0.5;
    } else {
        middle_uv.x = middle_uv.x + step_length * 0.5;
    }

    let offset = select(vec2<f32>(0.0, texel.y), vec2<f32>(texel.x, 0.0), horizontal);
    var uv_near = middle_uv - offset;
    var uv_far = middle_uv + offset;

    var luma_near = luma_at(uv_near) - luma_average;
    var luma_far = luma_at(uv_far) - luma_average;
    var ended_near = abs(luma_near) >= gradient_scaled;
    var ended_far = abs(luma_far) >= gradient_scaled;

    if !ended_near {
        uv_near = uv_near - offset;
    }
    if !ended_far {
        uv_far = uv_far + offset;
    }

    if !(ended_near && ended_far) {
        for (var step = 2; step < STEPS; step = step + 1) {
            if !ended_near {
                luma_near = luma_at(uv_near) - luma_average;
                ended_near = abs(luma_near) >= gradient_scaled;
            }
            if !ended_far {
                luma_far = luma_at(uv_far) - luma_average;
                ended_far = abs(luma_far) >= gradient_scaled;
            }
            if ended_near && ended_far {
                break;
            }
            let reach = reach_at(step);
            if !ended_near {
                uv_near = uv_near - offset * reach;
            }
            if !ended_far {
                uv_far = uv_far + offset * reach;
            }
        }
    }

    let distance_near = select(middle_uv.y - uv_near.y, middle_uv.x - uv_near.x, horizontal);
    let distance_far = select(uv_far.y - middle_uv.y, uv_far.x - middle_uv.x, horizontal);
    let near_is_nearer = distance_near < distance_far;
    let distance_nearest = min(distance_near, distance_far);
    let edge_length = distance_near + distance_far;
    var offset_along = 0.5 - distance_nearest / max(edge_length, 0.0001);

    let middle_is_lower = luma_middle < luma_average;
    let luma_nearest = select(luma_far, luma_near, near_is_nearer);
    if (luma_nearest < 0.0) == middle_is_lower {
        offset_along = 0.0;
    }

    let average = (2.0 * (luma_down_up + luma_left_right)
        + luma_left_corners
        + luma_right_corners)
        / 12.0;
    let subpixel_range = clamp(abs(average - luma_middle) / max(range, 0.0001), 0.0, 1.0);
    let subpixel_smooth = (-2.0 * subpixel_range + 3.0) * subpixel_range * subpixel_range;
    let subpixel_offset = subpixel_smooth * subpixel_smooth * fxaa.subpixel;

    let taken = max(offset_along, subpixel_offset);
    var read_uv = in.uv;
    if horizontal {
        read_uv.y = read_uv.y + taken * step_length;
    } else {
        read_uv.x = read_uv.x + taken * step_length;
    }
    return vec4<f32>(color_at(read_uv).rgb, middle.a);
}
