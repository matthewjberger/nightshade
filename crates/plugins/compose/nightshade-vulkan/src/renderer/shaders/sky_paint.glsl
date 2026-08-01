
#ifndef COMMON_PI
#define COMMON_PI
const float PI = 3.141592653589793;
#endif

vec3 mod289(vec3 value) {
    return value - floor(value * (1.0 / 289.0)) * 289.0;
}

vec4 mod289(vec4 value) {
    return value - floor(value * (1.0 / 289.0)) * 289.0;
}

vec4 permute(vec4 value) {
    return mod289(((value * 34.0) + 1.0) * value);
}

vec4 taylor_inverse_square_root(vec4 value) {
    return 1.79284291400159 - 0.85373472095314 * value;
}

float simplex_noise(vec3 point) {
    const vec2 constants = vec2(1.0 / 6.0, 1.0 / 3.0);
    const vec4 offsets = vec4(0.0, 0.5, 1.0, 2.0);

    vec3 cell = floor(point + dot(point, vec3(constants.y)));
    vec3 first = point - cell + dot(cell, vec3(constants.x));

    vec3 greater = step(first.yzx, first.xyz);
    vec3 lesser = 1.0 - greater;
    vec3 step_one = min(greater.xyz, lesser.zxy);
    vec3 step_two = max(greater.xyz, lesser.zxy);

    vec3 second = first - step_one + constants.x;
    vec3 third = first - step_two + constants.y;
    vec3 fourth = first - offsets.yyy;

    cell = mod289(cell);
    vec4 permuted = permute(permute(permute(
        cell.z + vec4(0.0, step_one.z, step_two.z, 1.0))
        + cell.y + vec4(0.0, step_one.y, step_two.y, 1.0))
        + cell.x + vec4(0.0, step_one.x, step_two.x, 1.0));

    float seventh = 0.142857142857;
    vec3 scaled = seventh * offsets.wyz - offsets.xzx;

    vec4 index = permuted - 49.0 * floor(permuted * scaled.z * scaled.z);

    vec4 index_x = floor(index * scaled.z);
    vec4 index_y = floor(index - 7.0 * index_x);

    vec4 grid_x = index_x * scaled.x + scaled.y;
    vec4 grid_y = index_y * scaled.x + scaled.y;
    vec4 height = 1.0 - abs(grid_x) - abs(grid_y);

    vec4 low = vec4(grid_x.xy, grid_y.xy);
    vec4 high = vec4(grid_x.zw, grid_y.zw);

    vec4 low_sign = floor(low) * 2.0 + 1.0;
    vec4 high_sign = floor(high) * 2.0 + 1.0;
    vec4 height_sign = -step(height, vec4(0.0));

    vec4 low_gradient = low.xzyw + low_sign.xzyw * height_sign.xxyy;
    vec4 high_gradient = high.xzyw + high_sign.xzyw * height_sign.zzww;

    vec3 gradient_one = vec3(low_gradient.xy, height.x);
    vec3 gradient_two = vec3(low_gradient.zw, height.y);
    vec3 gradient_three = vec3(high_gradient.xy, height.z);
    vec3 gradient_four = vec3(high_gradient.zw, height.w);

    vec4 normalization = taylor_inverse_square_root(vec4(
        dot(gradient_one, gradient_one),
        dot(gradient_two, gradient_two),
        dot(gradient_three, gradient_three),
        dot(gradient_four, gradient_four)
    ));
    gradient_one *= normalization.x;
    gradient_two *= normalization.y;
    gradient_three *= normalization.z;
    gradient_four *= normalization.w;

    vec4 falloff = max(0.6 - vec4(
        dot(first, first),
        dot(second, second),
        dot(third, third),
        dot(fourth, fourth)
    ), vec4(0.0));
    falloff = falloff * falloff;
    return 42.0 * dot(falloff * falloff, vec4(
        dot(gradient_one, first),
        dot(gradient_two, second),
        dot(gradient_three, third),
        dot(gradient_four, fourth)
    ));
}

float fractal_noise(vec3 point, int octaves) {
    float value = 0.0;
    float amplitude = 0.5;
    float frequency = 1.0;
    for (int octave = 0; octave < octaves; ++octave) {
        value += amplitude * (simplex_noise(point * frequency) * 0.5 + 0.5);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value;
}

float hash_cell(vec2 cell) {
    vec3 spread = fract(vec3(cell.x, cell.y, cell.x) * 0.1031);
    spread += dot(spread, spread.yzx + 33.33);
    return fract((spread.x + spread.y) * spread.z);
}

vec2 hash_cell_offset(vec2 cell) {
    const vec2 constants = vec2(0.3183099, 0.3678794);
    vec2 scaled = cell * constants + constants.yx;
    return fract(16.0 * constants * fract(scaled.x * scaled.y * (scaled.x + scaled.y)))
        * 2.0 - 1.0;
}

float hash_point(vec2 point) {
    const vec2 constants = vec2(0.3183099, 0.3678794);
    vec2 scaled = point * constants + constants.yx;
    return fract(16.0 * constants.x * fract(scaled.x * scaled.y * (scaled.x + scaled.y)));
}

vec2 direction_to_uv(vec3 direction) {
    float longitude = atan(direction.z, direction.x);
    float latitude = asin(clamp(direction.y, -1.0, 1.0));
    return vec2(longitude / 6.283185 + 0.5, latitude / 3.141593 + 0.5);
}

float star_layer(vec2 uv, float scale, float seed, float seconds, bool point_hash) {
    vec2 grid = uv * scale;
    vec2 cell = floor(grid);
    vec2 within = fract(grid) - 0.5;

    float chance = point_hash ? hash_point(cell + seed) : hash_cell(cell + seed);
    if (chance <= 0.97) {
        return 0.0;
    }

    vec2 offset = hash_cell_offset(cell + seed + 100.0) * 0.4;
    float distance_to_star = length(within - offset);
    float size = 0.02
        + (point_hash ? hash_point(cell + seed + 200.0) : hash_cell(cell + seed + 200.0)) * 0.03;
    float brightness = smoothstep(size, 0.0, distance_to_star);

    float speed = 1.0
        + (point_hash ? hash_point(cell + seed + 300.0) : hash_cell(cell + seed + 300.0)) * 3.0;
    float phase =
        (point_hash ? hash_point(cell + seed + 400.0) : hash_cell(cell + seed + 400.0)) * 6.283;
    return brightness * (0.7 + 0.3 * sin(seconds * speed + phase));
}

vec3 star_tint(float hue) {
    if (hue < 0.3) {
        return vec3(0.8, 0.85, 1.0);
    }
    if (hue > 0.7) {
        return vec3(1.0, 0.95, 0.85);
    }
    return vec3(1.0);
}

vec3 sky_gradient(vec3 direction) {
    const vec3 top = vec3(0.385, 0.454, 0.55);
    const vec3 horizon = vec3(0.646, 0.656, 0.67);
    const vec3 ground_bottom = vec3(0.2, 0.169, 0.133);

    float height = direction.y;
    vec3 color;
    if (height > 0.0) {
        float blend = 1.0 - pow(1.0 - height, 1.0 / 0.15);
        color = mix(horizon, top, clamp(blend, 0.0, 1.0));
    } else {
        float blend = 1.0 - pow(1.0 + height, 1.0 / 0.02);
        color = mix(horizon, ground_bottom, clamp(blend, 0.0, 1.0));
    }
    color *= 1.3;

    vec3 sun_direction = normalize(vec3(0.0, 0.5, -1.0));
    float sun_disk = 1.0 - smoothstep(0.0, 0.02, acos(dot(direction, sun_direction)));
    return mix(color, vec3(1.0, 0.95, 0.8), sun_disk * 0.5);
}

vec3 cloudy_sky(vec3 direction, float seconds) {
    const vec3 top = vec3(0.25, 0.4, 0.7);
    const vec3 horizon = vec3(0.4, 0.55, 0.75);
    float height = direction.y;

    vec3 color = horizon;
    if (height > 0.0) {
        float blend = 1.0 - pow(1.0 - height, 1.0 / 0.3);
        color = mix(horizon, top, clamp(blend, 0.0, 1.0));
    }

    vec3 sun_direction = normalize(vec3(0.0, 0.5, -1.0));
    float sun_disk = 1.0 - smoothstep(0.0, 0.02, acos(dot(direction, sun_direction)));
    color = mix(color, vec3(1.0, 0.95, 0.8), sun_disk * 0.5);

    if (height > 0.0) {
        vec3 drift = vec3(seconds * 0.012, 0.0, seconds * 0.006);
        vec3 cloud_point = direction * 2.5 + drift;
        float density = fractal_noise(cloud_point * 0.8, 5)
            + fractal_noise(cloud_point * 2.0, 4) * 0.5 * 0.4;
        float coverage = 0.62;
        float shape = smoothstep(coverage, coverage + 0.2, density);
        float fade = smoothstep(0.15, 0.45, height) * (1.0 - smoothstep(0.7, 0.95, height));
        float amount = shape * fade;

        float lit = pow(max(dot(direction, sun_direction), 0.0), 1.2);
        const vec3 shadow = vec3(0.65, 0.7, 0.78);
        const vec3 lit_color = vec3(1.0, 0.99, 0.97);
        const vec3 bright = vec3(1.0, 1.0, 0.98);

        vec3 cloud = mix(shadow, lit_color, smoothstep(0.0, 0.5, lit));
        cloud = mix(cloud, bright, smoothstep(0.5, 1.0, lit) * 0.5);
        cloud = mix(cloud, shadow, smoothstep(coverage, coverage + 0.35, density) * 0.3);
        color = mix(color, cloud, amount * 0.85);
    }
    return color;
}

vec3 space_sky(vec3 direction, float seconds) {
    vec2 uv = direction_to_uv(direction);
    float brightness = 0.0;
    brightness += star_layer(uv, 50.0, 0.0, seconds, true);
    brightness += star_layer(uv, 100.0, 10.0, seconds, true);
    brightness += star_layer(uv, 200.0, 20.0, seconds, true) * 0.7;
    brightness += star_layer(uv, 400.0, 30.0, seconds, true) * 0.4;
    return star_tint(hash_point(floor(uv * 100.0))) * brightness;
}

float nebula_stars(vec3 direction, float density, float seconds) {
    float noise = simplex_noise(direction * density);
    const float threshold = 0.75;
    if (noise <= threshold) {
        return 0.0;
    }
    float intensity = (noise - threshold) / (1.0 - threshold);
    vec3 cell = floor(direction * density);
    vec3 spread = fract(cell * vec3(443.8975, 397.2973, 491.1871));
    spread += dot(spread, spread.yxz + 19.19);
    float speed = 2.0 + fract((spread.x + spread.y) * spread.z) * 4.0;
    return pow(intensity, 3.0) * (0.7 + 0.3 * sin(seconds * speed));
}

vec3 nebula_sky(vec3 direction, float seconds) {
    float slow = seconds * 0.012;
    vec3 point = direction * 2.0;
    vec3 drift = vec3(slow * 0.05, slow * 0.03, slow * 0.04);

    float density_one = fractal_noise(point * 1.0 + drift, 6);
    float density_two = fractal_noise(point * 1.5 + vec3(43.0, 17.0, 29.0) + drift * 0.7, 6);
    float density_three = fractal_noise(point * 2.2 + vec3(97.0, 61.0, 83.0) + drift * 0.5, 5);

    const vec3 purple = vec3(0.4, 0.1, 0.6);
    const vec3 blue = vec3(0.1, 0.2, 0.5);
    const vec3 pink = vec3(0.6, 0.15, 0.4);
    const vec3 cyan = vec3(0.1, 0.4, 0.5);
    const vec3 magenta = vec3(0.5, 0.1, 0.5);

    float layer_one = smoothstep(0.3, 0.7, density_one);
    float layer_two = smoothstep(0.35, 0.75, density_two);
    float layer_three = smoothstep(0.4, 0.8, density_three);

    vec3 color = vec3(0.0);
    color += mix(purple, blue, density_two) * layer_one * 0.5;
    color += mix(pink, cyan, density_one) * layer_two * 0.4;
    color += mix(magenta, purple, density_two) * layer_three * 0.3;
    color += vec3(0.8, 0.6, 0.9) * smoothstep(0.6, 0.95, density_one * density_two) * 0.3;

    float dust = fractal_noise(point * 3.5 + vec3(31.0, 47.0, 23.0), 4);
    color *= 1.0 - smoothstep(0.4, 0.7, dust) * 0.35;

    float brightness = 0.0;
    brightness += nebula_stars(direction, 50.0, seconds) * 0.8;
    brightness += nebula_stars(direction, 100.0, seconds) * 0.5;
    brightness += nebula_stars(direction, 200.0, seconds) * 0.3;
    brightness += nebula_stars(direction, 400.0, seconds) * 0.15;

    float mask = max(layer_one, max(layer_two, layer_three));
    float temperature = simplex_noise(direction * 80.0);
    vec3 tint = vec3(1.0);
    if (temperature < -0.3) {
        tint = vec3(0.8, 0.85, 1.0);
    } else if (temperature > 0.3) {
        tint = vec3(1.0, 0.95, 0.85);
    }

    float bright_noise = simplex_noise(direction * 20.0);
    float bright_star = 0.0;
    if (bright_noise > 0.85) {
        float intensity = (bright_noise - 0.85) / 0.15;
        bright_star = pow(intensity, 2.0)
            * (0.85 + 0.15 * sin(seconds * 1.5 + bright_noise * 10.0)) * 2.0;
    }

    vec3 result = vec3(0.01, 0.005, 0.02);
    result += color;
    result += tint * brightness * (1.0 - mask * 0.5);
    result += vec3(1.0, 0.95, 0.9) * bright_star;
    return result;
}

vec3 sunset_sky(vec3 direction, float seconds) {
    vec3 sun_direction = normalize(vec3(0.5, 0.12, -0.8));
    float height = direction.y;
    float toward_sun = dot(direction, sun_direction);
    float proximity = max(toward_sun, 0.0);

    const vec3 deep_orange = vec3(0.95, 0.35, 0.08);
    const vec3 warm_orange = vec3(1.0, 0.5, 0.15);
    const vec3 salmon = vec3(0.95, 0.55, 0.45);
    const vec3 pink = vec3(0.85, 0.45, 0.55);
    const vec3 purple = vec3(0.35, 0.2, 0.45);
    const vec3 dark_blue = vec3(0.1, 0.08, 0.25);

    float above = abs(height);
    vec3 base;
    if (above < 0.1) {
        base = mix(deep_orange, warm_orange, above / 0.1);
    } else if (above < 0.25) {
        base = mix(warm_orange, salmon, (above - 0.1) / 0.15);
    } else if (above < 0.45) {
        base = mix(salmon, pink, (above - 0.25) / 0.2);
    } else if (above < 0.65) {
        base = mix(pink, purple, (above - 0.45) / 0.2);
    } else {
        base = mix(purple, dark_blue, (above - 0.65) / 0.35);
    }

    float warmth = pow(proximity, 1.5) * 0.4;
    vec3 boost = mix(vec3(1.0, 0.6, 0.3), vec3(1.0, 0.8, 0.6), above);
    vec3 color = mix(base, base + boost * 0.3, warmth);

    if (height < 0.0) {
        float darken = 1.0 - smoothstep(-0.4, 0.0, height);
        color *= mix(1.0, 0.15, darken);
        color = mix(color, vec3(0.01, 0.008, 0.015), 1.0 - smoothstep(-0.8, -0.3, height));
    }

    const float angular_radius = 0.9995;
    float core = smoothstep(angular_radius - 0.0003, angular_radius, toward_sun);
    float limb = smoothstep(angular_radius - 0.001, angular_radius - 0.0003, toward_sun) * 0.3;
    color += vec3(1.0, 0.98, 0.9) * core;
    color += vec3(1.0, 0.8, 0.4) * limb;
    color += vec3(1.0, 0.6, 0.25) * pow(proximity, 32.0) * 0.6;
    color += vec3(1.0, 0.6, 0.25) * pow(proximity, 8.0) * 0.35;
    color += deep_orange * pow(proximity, 2.5) * 0.2;

    if (height > 0.0 && height < 0.6) {
        vec3 cloud_point = (direction + vec3(seconds * 0.008, 0.0, seconds * 0.004)) * 3.0;
        float density = fractal_noise(cloud_point * 0.8, 5)
            + fractal_noise(cloud_point * 2.5, 3) * 0.4 * 0.5;
        float shape = smoothstep(0.4, 0.7, density);
        float amount = shape
            * smoothstep(0.0, 0.15, height)
            * (1.0 - smoothstep(0.35, 0.6, height));

        float lit = pow(max(dot(direction, sun_direction), 0.0), 1.5);
        const vec3 shadow = vec3(0.25, 0.12, 0.18);
        const vec3 middle = vec3(0.6, 0.35, 0.3);
        const vec3 bright = vec3(1.0, 0.65, 0.4);
        const vec3 rim = vec3(1.0, 0.85, 0.6);

        vec3 cloud = mix(shadow, middle, smoothstep(0.0, 0.3, lit));
        cloud = mix(cloud, bright, smoothstep(0.3, 0.7, lit));
        cloud = mix(cloud, rim, smoothstep(0.7, 1.0, lit) * 0.5);
        float edge = smoothstep(0.5, 0.7, density) - smoothstep(0.7, 0.9, density);
        cloud += rim * edge * lit * 0.3;
        color = mix(color, cloud, amount * 0.9);
    }

    if (height > 0.5) {
        float visibility = smoothstep(0.5, 0.8, height);
        float noise = simplex_noise(direction * 180.0);
        if (noise > 0.82) {
            float intensity = (noise - 0.82) / 0.18;
            float twinkle = 0.75 + 0.25 * sin(seconds * 2.0 + noise * 50.0);
            color += vec3(1.0, 0.98, 0.95) * pow(intensity, 2.5) * twinkle * visibility * 0.5;
        }
    }
    return color;
}

vec3 day_night_sun_direction(float hour) {
    float angle = (hour - 6.0) / 12.0 * PI;
    return normalize(vec3(-cos(angle), sin(angle), -0.3));
}

vec3 day_night_moon_direction(float hour) {
    float shifted = hour < 12.0 ? hour + 24.0 : hour;
    float angle = (shifted - 18.0) / 12.0 * PI;
    return normalize(vec3(-cos(angle), sin(angle), 0.3));
}

vec3 day_night_gradient(vec3 direction, float hour) {
    const vec3 night = vec3(0.01, 0.01, 0.03);
    const vec3 dawn_horizon = vec3(0.8, 0.4, 0.15);
    const vec3 dawn_sky = vec3(0.2, 0.25, 0.5);
    const vec3 morning = vec3(0.35, 0.5, 0.75);
    const vec3 midday_top = vec3(0.25, 0.45, 0.8);
    const vec3 midday_horizon = vec3(0.55, 0.65, 0.8);
    const vec3 afternoon = vec3(0.35, 0.5, 0.7);
    const vec3 sunset_horizon = vec3(0.95, 0.35, 0.08);
    const vec3 sunset_upper = vec3(0.35, 0.2, 0.45);
    const vec3 dusk = vec3(0.1, 0.08, 0.2);

    vec3 top;
    vec3 horizon;
    if (hour < 5.0) {
        top = night;
        horizon = night;
    } else if (hour < 6.0) {
        float blend = hour - 5.0;
        top = mix(night, dawn_sky * 0.3, blend);
        horizon = mix(night, dawn_horizon * 0.3, blend);
    } else if (hour < 7.0) {
        float blend = hour - 6.0;
        top = mix(dawn_sky * 0.3, morning * 0.7, blend);
        horizon = mix(dawn_horizon * 0.3, dawn_horizon, blend);
    } else if (hour < 9.0) {
        float blend = (hour - 7.0) / 2.0;
        top = mix(morning * 0.7, midday_top, blend);
        horizon = mix(dawn_horizon, midday_horizon, blend);
    } else if (hour < 15.0) {
        top = midday_top;
        horizon = midday_horizon;
    } else if (hour < 17.0) {
        float blend = (hour - 15.0) / 2.0;
        top = mix(midday_top, afternoon, blend);
        horizon = mix(midday_horizon, vec3(0.7, 0.55, 0.45), blend);
    } else if (hour < 18.0) {
        float blend = hour - 17.0;
        top = mix(afternoon, sunset_upper, blend);
        horizon = mix(vec3(0.7, 0.55, 0.45), sunset_horizon, blend);
    } else if (hour < 19.0) {
        float blend = hour - 18.0;
        top = mix(sunset_upper, dusk, blend);
        horizon = mix(sunset_horizon, dusk * 1.5, blend);
    } else if (hour < 20.0) {
        float blend = hour - 19.0;
        top = mix(dusk, night, blend);
        horizon = mix(dusk * 1.5, night, blend);
    } else {
        top = night;
        horizon = night;
    }

    float height = direction.y;
    if (height >= 0.0) {
        float blend = 1.0 - pow(1.0 - height, 1.0 / 0.15);
        return mix(horizon, top, clamp(blend, 0.0, 1.0));
    }
    float darken = 1.0 - smoothstep(-0.4, 0.0, height);
    return mix(horizon, night, darken) * mix(1.0, 0.15, darken);
}

vec3 milky_way(vec3 direction) {
    vec3 pole = normalize(vec3(0.2, 0.9, 0.1));
    float band = 1.0 - smoothstep(0.0, 0.35, abs(dot(direction, pole)));
    vec3 center = normalize(vec3(-0.8, -0.2, 0.5));
    float toward_center = pow(max(dot(direction, center), 0.0), 3.0);

    float detail = simplex_noise(direction * 8.0) * 0.5
        + simplex_noise(direction * 16.0) * 0.25
        + simplex_noise(direction * 32.0) * 0.125;
    detail = detail * 0.5 + 0.5;

    float dust = smoothstep(0.3, 0.6, simplex_noise(direction * 12.0 + vec3(0.0, 5.0, 0.0))) * 0.4;
    float brightness = max(band * (0.4 + toward_center * 0.6) * detail - dust * band, 0.0);

    vec3 tint = mix(vec3(0.04, 0.045, 0.06), vec3(0.06, 0.05, 0.045), toward_center);
    return tint * brightness;
}

vec3 moon_surface(vec2 local) {
    float maria = smoothstep(
        0.1,
        0.5,
        simplex_noise(vec3(local * 3.0, 0.0)) * 0.6
            + simplex_noise(vec3(local * 1.5 + vec2(10.0, 5.0), 0.0)) * 0.4
    );
    float craters = simplex_noise(vec3(local * 25.0, 1.0)) * 0.3
        + simplex_noise(vec3(local * 50.0, 2.0)) * 0.15
        + simplex_noise(vec3(local * 100.0, 3.0)) * 0.08;
    vec3 color = mix(vec3(0.85, 0.83, 0.78), vec3(0.45, 0.43, 0.42), maria);
    return color * (1.0 + craters * 0.3);
}

vec3 day_night_sky(vec3 direction, float seconds, float raw_hour) {
    float hour = raw_hour - floor(raw_hour / 24.0) * 24.0;
    vec3 color = day_night_gradient(direction, hour);
    float height = direction.y;

    if (hour >= 5.5 && hour <= 18.5) {
        vec3 sun_direction = day_night_sun_direction(hour);
        float toward_sun = dot(direction, sun_direction);
        float proximity = max(toward_sun, 0.0);

        float intensity = 1.0;
        if (hour < 6.5) {
            intensity = smoothstep(5.5, 6.5, hour);
        } else if (hour > 17.5) {
            intensity = 1.0 - smoothstep(17.5, 18.5, hour);
        }

        vec3 core_color = vec3(1.0, 0.98, 0.9);
        vec3 glow_color = vec3(1.0, 0.8, 0.5);
        if (hour < 7.0 || hour > 17.0) {
            core_color = vec3(1.0, 0.85, 0.6);
            glow_color = vec3(1.0, 0.5, 0.2);
        }

        const float angular_radius = 0.9995;
        float core = smoothstep(angular_radius - 0.0003, angular_radius, toward_sun);
        float limb = smoothstep(angular_radius - 0.001, angular_radius - 0.0003, toward_sun) * 0.3;

        color += core_color * core * intensity;
        color += vec3(1.0, 0.8, 0.4) * limb * intensity;
        color += glow_color * pow(proximity, 32.0) * 0.6 * intensity;
        color += glow_color * pow(proximity, 8.0) * 0.35 * intensity;
        color += glow_color * 0.5 * pow(proximity, 2.5) * 0.2 * intensity;
    }

    if (height > 0.0 && height < 0.6) {
        vec3 sun_direction = day_night_sun_direction(clamp(hour, 6.0, 18.0));
        vec3 cloud_point = (direction + vec3(seconds * 0.008, 0.0, seconds * 0.004)) * 3.0;
        float density = fractal_noise(cloud_point * 0.8, 5)
            + fractal_noise(cloud_point * 2.5, 3) * 0.4 * 0.5;
        float shape = smoothstep(0.4, 0.7, density);
        float amount = shape
            * smoothstep(0.0, 0.15, height)
            * (1.0 - smoothstep(0.35, 0.6, height));

        float lit = pow(max(dot(direction, sun_direction), 0.0), 1.5);

        vec3 shadow;
        vec3 bright;
        vec3 rim;
        if (hour >= 17.0 && hour <= 18.0) {
            float blend = hour - 17.0;
            shadow = mix(vec3(0.5, 0.5, 0.55), vec3(0.35, 0.25, 0.3), blend);
            bright = mix(vec3(0.95, 0.95, 0.93), vec3(1.0, 0.65, 0.4), blend);
            rim = mix(vec3(1.0, 1.0, 0.98), vec3(1.0, 0.85, 0.6), blend);
        } else if (hour >= 5.0 && hour <= 7.0) {
            float blend = 1.0 - (hour - 5.0) / 2.0;
            shadow = mix(vec3(0.5, 0.5, 0.55), vec3(0.35, 0.25, 0.3), blend);
            bright = mix(vec3(0.95, 0.95, 0.93), vec3(1.0, 0.55, 0.35), blend);
            rim = mix(vec3(1.0, 1.0, 0.98), vec3(1.0, 0.75, 0.5), blend);
        } else {
            shadow = vec3(0.5, 0.5, 0.55);
            bright = vec3(0.95, 0.95, 0.93);
            rim = vec3(1.0, 1.0, 0.98);
        }

        float edge = smoothstep(0.5, 0.7, density) - smoothstep(0.7, 0.9, density);
        vec3 middle = mix(shadow, bright, 0.4);
        vec3 day_cloud = mix(shadow, middle, smoothstep(0.0, 0.3, lit));
        day_cloud = mix(day_cloud, bright, smoothstep(0.3, 0.7, lit));
        day_cloud = mix(day_cloud, rim, smoothstep(0.7, 1.0, lit) * 0.5);
        day_cloud += rim * edge * lit * 0.3;

        vec3 moon_direction = day_night_moon_direction(hour);
        float moon_lit = pow(max(dot(direction, moon_direction), 0.0), 2.0);
        const vec3 night_shadow = vec3(0.02, 0.02, 0.04);
        const vec3 night_bright = vec3(0.06, 0.07, 0.10);
        const vec3 night_rim = vec3(0.10, 0.12, 0.16);
        vec3 night_middle = mix(night_shadow, night_bright, 0.3);
        vec3 night_cloud = mix(night_shadow, night_middle, smoothstep(0.0, 0.3, moon_lit));
        night_cloud = mix(night_cloud, night_bright, smoothstep(0.3, 0.7, moon_lit));
        night_cloud = mix(night_cloud, night_rim, smoothstep(0.7, 1.0, moon_lit) * 0.4);
        night_cloud += night_rim * edge * moon_lit * 0.2;
        night_cloud *= 0.5 + 0.5 * max(moon_direction.y, 0.0);

        float day_factor = 1.0;
        if (hour < 5.0 || hour > 20.0) {
            day_factor = 0.0;
        } else if (hour < 7.0) {
            day_factor = smoothstep(5.0, 7.0, hour);
        } else if (hour > 18.0) {
            day_factor = 1.0 - smoothstep(18.0, 20.0, hour);
        }

        color = mix(color, mix(night_cloud, day_cloud, day_factor), amount * 0.9);
    }

    float night_visibility = 0.0;
    if (hour < 5.0 || hour > 20.0) {
        night_visibility = 1.0;
    } else if (hour < 7.0) {
        night_visibility = 1.0 - smoothstep(5.0, 7.0, hour);
    } else if (hour > 18.0) {
        night_visibility = smoothstep(18.0, 20.0, hour);
    }

    if (night_visibility > 0.0 && height > -0.1) {
        vec2 uv = direction_to_uv(direction);
        float brightness = 0.0;
        brightness += star_layer(uv, 50.0, 0.0, seconds, false);
        brightness += star_layer(uv, 100.0, 10.0, seconds, false);
        brightness += star_layer(uv, 200.0, 20.0, seconds, false) * 0.7;
        brightness += star_layer(uv, 400.0, 30.0, seconds, false) * 0.4;

        float fade = smoothstep(-0.1, 0.1, height);
        color += star_tint(hash_cell(floor(uv * 100.0))) * brightness * night_visibility * fade;
        color += milky_way(direction) * night_visibility * fade;
    }

    vec3 moon_direction = day_night_moon_direction(hour);
    if (moon_direction.y > -0.05) {
        const float angular_size = 0.03;
        float distance_to_moon = acos(clamp(dot(direction, moon_direction), -1.0, 1.0));
        if (distance_to_moon < angular_size * 1.5) {
            float horizon_fade = smoothstep(-0.05, 0.05, moon_direction.y);
            float disc = 1.0 - smoothstep(angular_size * 0.95, angular_size, distance_to_moon);
            float glow =
                (1.0 - smoothstep(angular_size, angular_size * 1.5, distance_to_moon)) * 0.15;

            vec3 right = normalize(cross(moon_direction, vec3(0.0, 1.0, 0.0)));
            vec3 up = normalize(cross(right, moon_direction));
            vec2 local = vec2(
                dot(direction - moon_direction, right) / angular_size,
                dot(direction - moon_direction, up) / angular_size
            );

            float phase_dot = dot(moon_direction, day_night_sun_direction(hour));
            float phase = smoothstep(-0.2, 0.8, local.x * sign(phase_dot) * 0.5 + 0.5);
            color = mix(color, moon_surface(local) * phase * 0.8, disc * horizon_fade);
            color += vec3(0.7, 0.75, 0.85) * glow * horizon_fade;
        }
    }

    return color;
}
