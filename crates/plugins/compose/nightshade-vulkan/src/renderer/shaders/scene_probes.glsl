
#define PROBE_MAX_X 16
#define PROBE_MAX_Y 8
#define PROBE_MAX_Z 16

#define PROBE_IRRADIANCE_TEXELS 8
#define PROBE_DISTANCE_TEXELS 16
#define PROBE_BORDER 1
#define PROBE_RAYS 64

#define PROBE_IRRADIANCE_TILE (PROBE_IRRADIANCE_TEXELS + PROBE_BORDER * 2)
#define PROBE_DISTANCE_TILE (PROBE_DISTANCE_TEXELS + PROBE_BORDER * 2)

#define PROBE_SURFACE_BIAS 0.6

vec2 octahedral_encode(vec3 direction) {
    vec3 folded = direction / max(abs(direction.x) + abs(direction.y) + abs(direction.z), 1.0e-8);
    vec2 point = folded.z >= 0.0
        ? folded.xy
        : (vec2(1.0) - abs(folded.yx)) * vec2(
            folded.x >= 0.0 ? 1.0 : -1.0,
            folded.y >= 0.0 ? 1.0 : -1.0
        );
    return point * 0.5 + 0.5;
}

vec3 octahedral_decode(vec2 point) {
    vec2 folded = point * 2.0 - 1.0;
    vec3 direction = vec3(folded.x, folded.y, 1.0 - abs(folded.x) - abs(folded.y));
    float below = clamp(-direction.z, 0.0, 1.0);
    direction.x += direction.x >= 0.0 ? -below : below;
    direction.y += direction.y >= 0.0 ? -below : below;
    return normalize(direction);
}

vec3 probe_turned_by(vec4 turn, vec3 direction) {
    vec3 axis = turn.xyz;
    return direction + 2.0 * cross(axis, cross(axis, direction) + turn.w * direction);
}

vec3 probe_ray_direction(uint ray) {
    const float golden = 2.399963229728653;
    float up = 1.0 - (float(ray) + 0.5) * (2.0 / float(PROBE_RAYS));
    float around = golden * float(ray);
    float radius = sqrt(max(1.0 - up * up, 0.0));
    return normalize(vec3(cos(around) * radius, up, sin(around) * radius));
}

vec3 probe_ray_of(uint ray) {
    return probe_turned_by(frame.probe_rotation, probe_ray_direction(ray));
}

vec2 probe_texel_point(ivec2 interior, int texels) {
    return (vec2(interior) + 0.5) / float(texels);
}

ivec2 probe_border_source(ivec2 tile, int texels) {
    int last = texels + 1;
    bool left = tile.x == 0;
    bool right = tile.x == last;
    bool top = tile.y == 0;
    bool bottom = tile.y == last;
    if ((left || right) && (top || bottom)) {
        return ivec2(left ? texels : 1, top ? texels : 1);
    }
    if (top || bottom) {
        return ivec2(last - tile.x, top ? 1 : texels);
    }
    return ivec2(left ? 1 : texels, last - tile.y);
}

uvec3 probe_counts() {
    return clamp(frame.probe_counts.xyz, uvec3(1u), uvec3(PROBE_MAX_X, PROBE_MAX_Y, PROBE_MAX_Z));
}

ivec3 probe_coords_of(uint index) {
    uvec3 counts = probe_counts();
    uint across = counts.x * counts.y;
    return ivec3(
        int(index % counts.x),
        int((index / counts.x) % counts.y),
        int(index / across)
    );
}

uint probe_index_of(ivec3 coords) {
    uvec3 counts = probe_counts();
    return uint(coords.x) + uint(coords.y) * counts.x + uint(coords.z) * counts.x * counts.y;
}

vec3 probe_position(ivec3 coords) {
    vec3 corner = frame.probe_origin.xyz + vec3(coords) * frame.probe_spacing.xyz;
    return corner + probe_offsets[probe_index_of(coords)].xyz;
}

uint probe_ray_slot(uint probe, uint ray) {
    return (probe * uint(PROBE_RAYS) + ray) * 2u;
}

ivec2 probe_tile_origin(ivec3 coords, int tile) {
    int column = coords.x + coords.y * PROBE_MAX_X;
    return ivec2(column * tile, coords.z * tile);
}

vec2 probe_atlas_point(ivec3 coords, vec3 direction, int texels, int tile, vec2 atlas_size) {
    vec2 local = octahedral_encode(normalize(direction));
    vec2 origin = vec2(probe_tile_origin(coords, tile)) + vec2(float(PROBE_BORDER));
    return (origin + local * float(texels)) / atlas_size;
}

vec3 probe_arriving_light(vec3 position, vec3 normal, vec3 to_view) {
    if (frame.probe_counts.w == 0u) {
        return vec3(0.0);
    }
    uvec3 counts = probe_counts();
    vec3 spacing = max(frame.probe_spacing.xyz, vec3(1.0e-4));
    float nearest = min(min(spacing.x, spacing.y), spacing.z);
    vec3 asked_from = position + (normal * 0.2 + to_view * 0.8) * PROBE_SURFACE_BIAS * nearest;

    vec3 local = (asked_from - frame.probe_origin.xyz) / spacing;
    vec3 lowest = floor(local);
    vec3 across = clamp(local - lowest, vec3(0.0), vec3(1.0));
    ivec3 base = clamp(ivec3(lowest), ivec3(0), ivec3(counts) - 1);

    vec2 irradiance_size = vec2(textureSize(probe_irradiance, 0));
    vec2 distance_size = vec2(textureSize(probe_distance, 0));

    vec3 gathered = vec3(0.0);
    float total = 0.0;
    for (int corner = 0; corner < 8; ++corner) {
        ivec3 step_to = ivec3(corner & 1, (corner >> 1) & 1, (corner >> 2) & 1);
        ivec3 coords = clamp(base + step_to, ivec3(0), ivec3(counts) - 1);
        vec3 stands_at = probe_position(coords);
        vec3 toward = stands_at - position;
        float lies = length(toward);
        vec3 direction = lies > 1.0e-5 ? toward / lies : normal;
        vec3 from_probe = asked_from - stands_at;
        float away = length(from_probe);
        vec3 looked = away > 1.0e-5 ? from_probe / away : -normal;

        vec3 share = mix(vec3(1.0) - across, across, vec3(step_to));
        float weight = share.x * share.y * share.z;

        float facing = dot(direction, normal) * 0.5 + 0.5;
        weight *= facing * facing + 0.2;

        vec2 moments = texture(
            probe_distance,
            probe_atlas_point(coords, looked, PROBE_DISTANCE_TEXELS, PROBE_DISTANCE_TILE, distance_size)
        ).rg;
        float mean = moments.x;
        if (away > mean) {
            float variance = abs(mean * mean - moments.y);
            float apart = away - mean;
            float seen = variance / max(variance + apart * apart, 1.0e-6);
            weight *= max(seen * seen * seen, 0.0);
        }

        const float faintest = 0.2;
        if (weight < faintest) {
            weight *= (weight * weight) / (faintest * faintest);
        }
        weight = max(weight, 1.0e-4);

        vec3 carried = texture(
            probe_irradiance,
            probe_atlas_point(coords, normal, PROBE_IRRADIANCE_TEXELS, PROBE_IRRADIANCE_TILE, irradiance_size)
        ).rgb;
        gathered += carried * carried * weight;
        total += weight;
    }
    if (total <= 1.0e-6) {
        return vec3(0.0);
    }
    return gathered / total * frame.probe_origin.w;
}
