
uint cube_face_of(vec3 direction) {
    vec3 size = abs(direction);
    if (size.x >= size.y && size.x >= size.z) {
        return direction.x > 0.0 ? 0u : 1u;
    }
    if (size.y >= size.z) {
        return direction.y > 0.0 ? 2u : 3u;
    }
    return direction.z > 0.0 ? 4u : 5u;
}

bool local_shadow_of(Light light, vec3 world_position, out LocalShadow shadow, out vec3 settled) {
    if (light.shadow_index < 0 || frame.shadow_settings.z == 0.0) {
        return false;
    }
    uint slot = uint(light.shadow_index);
    if (light.light_type == LIGHT_POINT) {
        slot += cube_face_of(world_position - light.position.xyz);
    }
    if (slot >= LOCAL_SHADOW_COUNT) {
        return false;
    }
    shadow = local_shadows[slot];
    if (shadow.light.w <= 0.0) {
        return false;
    }
    vec4 in_light = shadow.view_projection * vec4(world_position, 1.0);
    if (in_light.w <= 0.0) {
        return false;
    }
    settled = in_light.xyz / in_light.w;
    return all(greaterThanEqual(settled.xy, vec2(-1.0))) && all(lessThanEqual(settled.xy, vec2(1.0)));
}

float local_reaches(Light light, vec3 world_position) {
    LocalShadow shadow;
    vec3 settled;
    if (!local_shadow_of(light, world_position, shadow, settled)) {
        return 1.0;
    }
    vec2 across = settled.xy * 0.5 + 0.5;
    vec4 place = shadow.slot;
    vec2 texel = vec2(1.0 / frame.local_shadow.x);
    vec2 at = clamp(
        place.xy + across * place.z,
        place.xy + texel,
        place.xy + vec2(place.z) - texel
    );
    return texture(local_shadow_atlas, vec3(at, settled.z + place.w));
}

vec3 spot_picture(Light light, vec3 world_position) {
    if (light.cookie_layer == NO_COOKIE || light.shadow_index < 0) {
        return vec3(1.0);
    }
    uint slot = uint(light.shadow_index);
    if (slot >= LOCAL_SHADOW_COUNT) {
        return vec3(1.0);
    }
    vec4 in_light = local_shadows[slot].view_projection * vec4(world_position, 1.0);
    if (in_light.w <= 0.0) {
        return vec3(0.0);
    }
    vec3 settled = in_light.xyz / in_light.w;
    vec2 across = vec2(settled.x * 0.5 + 0.5, 0.5 - settled.y * 0.5);
    if (any(lessThan(across, vec2(0.0))) || any(greaterThan(across, vec2(1.0)))) {
        return vec3(0.0);
    }
    return picture_sample(light.cookie_layer, across).rgb;
}

vec3 point_picture(Light light, vec3 world_position) {
    if (light.cookie_layer == NO_COOKIE) {
        return vec3(1.0);
    }
    vec3 direction = normalize(world_position - light.position.xyz);
    vec2 across = vec2(
        atan(direction.z, direction.x) / (2.0 * PI) + 0.5,
        acos(clamp(direction.y, -1.0, 1.0)) / PI
    );
    return picture_sample(light.cookie_layer, across).rgb;
}
