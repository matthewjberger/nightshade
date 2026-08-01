#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D composed;

layout(push_constant) uniform Push {
    vec2 texel;
    float relative_threshold;
    float absolute_threshold;
    float subpixel;
    float snap;
} push;

const int STEPS = 8;

float reach_at(int step) {
    if (step < 4) {
        return 1.0;
    }
    if (step < 6) {
        return 2.0;
    }
    return 4.0;
}

float luma(vec3 color) {
    return sqrt(dot(color, vec3(0.299, 0.587, 0.114)));
}

vec4 color_at(vec2 at) {
    return textureLod(composed, at, 0.0);
}

float luma_at(vec2 at) {
    return luma(color_at(at).rgb);
}

void main() {
    vec2 texel = push.texel;
    vec2 uv = uv;
    if (push.snap > 0.5) {
        uv = (floor(uv / texel) + 0.5) * texel;
    }
    vec4 middle = color_at(uv);
    float luma_middle = luma(middle.rgb);

    float luma_down = luma_at(uv + vec2(0.0, texel.y));
    float luma_up = luma_at(uv - vec2(0.0, texel.y));
    float luma_right = luma_at(uv + vec2(texel.x, 0.0));
    float luma_left = luma_at(uv - vec2(texel.x, 0.0));

    float luma_lowest = min(luma_middle, min(min(luma_down, luma_up), min(luma_right, luma_left)));
    float luma_highest = max(luma_middle, max(max(luma_down, luma_up), max(luma_right, luma_left)));
    float range = luma_highest - luma_lowest;

    if (range < max(push.absolute_threshold, luma_highest * push.relative_threshold)) {
        out_color = middle;
        return;
    }

    float luma_down_left = luma_at(uv + vec2(-texel.x, texel.y));
    float luma_up_right = luma_at(uv + vec2(texel.x, -texel.y));
    float luma_up_left = luma_at(uv - texel);
    float luma_down_right = luma_at(uv + texel);

    float luma_down_up = luma_down + luma_up;
    float luma_left_right = luma_left + luma_right;
    float luma_left_corners = luma_down_left + luma_up_left;
    float luma_down_corners = luma_down_left + luma_down_right;
    float luma_right_corners = luma_down_right + luma_up_right;
    float luma_up_corners = luma_up_right + luma_up_left;

    float edge_horizontal = abs(-2.0 * luma_left + luma_left_corners)
        + abs(-2.0 * luma_middle + luma_down_up) * 2.0
        + abs(-2.0 * luma_right + luma_right_corners);
    float edge_vertical = abs(-2.0 * luma_up + luma_up_corners)
        + abs(-2.0 * luma_middle + luma_left_right) * 2.0
        + abs(-2.0 * luma_down + luma_down_corners);
    bool horizontal = edge_horizontal >= edge_vertical;

    float luma_side_one = horizontal ? luma_up : luma_left;
    float luma_side_two = horizontal ? luma_down : luma_right;
    float gradient_one = luma_side_one - luma_middle;
    float gradient_two = luma_side_two - luma_middle;
    bool steeper_is_one = abs(gradient_one) >= abs(gradient_two);
    float gradient_scaled = 0.25 * max(abs(gradient_one), abs(gradient_two));

    float step_length = horizontal ? texel.y : texel.x;
    float luma_average;
    if (steeper_is_one) {
        step_length = -step_length;
        luma_average = 0.5 * (luma_side_one + luma_middle);
    } else {
        luma_average = 0.5 * (luma_side_two + luma_middle);
    }

    vec2 middle_uv = uv;
    if (horizontal) {
        middle_uv.y += step_length * 0.5;
    } else {
        middle_uv.x += step_length * 0.5;
    }

    vec2 offset = horizontal ? vec2(texel.x, 0.0) : vec2(0.0, texel.y);
    vec2 uv_near = middle_uv - offset;
    vec2 uv_far = middle_uv + offset;

    float luma_near = luma_at(uv_near) - luma_average;
    float luma_far = luma_at(uv_far) - luma_average;
    bool ended_near = abs(luma_near) >= gradient_scaled;
    bool ended_far = abs(luma_far) >= gradient_scaled;

    if (!ended_near) {
        uv_near -= offset;
    }
    if (!ended_far) {
        uv_far += offset;
    }

    if (!(ended_near && ended_far)) {
        for (int step = 2; step < STEPS; step++) {
            if (!ended_near) {
                luma_near = luma_at(uv_near) - luma_average;
                ended_near = abs(luma_near) >= gradient_scaled;
            }
            if (!ended_far) {
                luma_far = luma_at(uv_far) - luma_average;
                ended_far = abs(luma_far) >= gradient_scaled;
            }
            if (ended_near && ended_far) {
                break;
            }
            float reach = reach_at(step);
            if (!ended_near) {
                uv_near -= offset * reach;
            }
            if (!ended_far) {
                uv_far += offset * reach;
            }
        }
    }

    float distance_near = horizontal ? middle_uv.x - uv_near.x : middle_uv.y - uv_near.y;
    float distance_far = horizontal ? uv_far.x - middle_uv.x : uv_far.y - middle_uv.y;
    bool near_is_nearer = distance_near < distance_far;
    float distance_nearest = min(distance_near, distance_far);
    float edge_length = distance_near + distance_far;
    float offset_along = 0.5 - distance_nearest / max(edge_length, 0.0001);

    bool middle_is_lower = luma_middle < luma_average;
    float luma_nearest = near_is_nearer ? luma_near : luma_far;
    if ((luma_nearest < 0.0) == middle_is_lower) {
        offset_along = 0.0;
    }

    float average = (2.0 * (luma_down_up + luma_left_right)
        + luma_left_corners
        + luma_right_corners)
        / 12.0;
    float subpixel_range = clamp(abs(average - luma_middle) / max(range, 0.0001), 0.0, 1.0);
    float subpixel_smooth = (-2.0 * subpixel_range + 3.0) * subpixel_range * subpixel_range;
    float subpixel_offset = subpixel_smooth * subpixel_smooth * push.subpixel;

    float taken = max(offset_along, subpixel_offset);
    vec2 read_uv = uv;
    if (horizontal) {
        read_uv.y += taken * step_length;
    } else {
        read_uv.x += taken * step_length;
    }
    out_color = vec4(color_at(read_uv).rgb, middle.a);
}
