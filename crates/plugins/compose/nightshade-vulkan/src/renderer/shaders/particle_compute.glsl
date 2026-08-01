layout(scalar, set = 0, binding = 0) buffer Particles { Particle particles[]; };
layout(scalar, set = 0, binding = 1) readonly buffer Emitters { EmitterData emitters[]; };
layout(scalar, set = 0, binding = 2) uniform Simulation {
    float delta_time;
    float time;
    uint max_particles;
    uint padding;
} simulation;
layout(scalar, set = 0, binding = 3) buffer FreeIndices { uint free_indices[]; };
layout(scalar, set = 0, binding = 4) buffer FreeCount { uint free_count; };
layout(scalar, set = 0, binding = 5) buffer AliveIndices { uint alive_indices[]; };
layout(scalar, set = 0, binding = 6) buffer AliveCount { uint alive_count; };
layout(scalar, set = 0, binding = 7) buffer DrawCommand {
    uint vertex_count;
    uint instance_count;
    uint first_vertex;
    uint first_instance;
} draw_command;

uint hash(uint seed) {
    uint value = seed;
    value = value ^ 2747636419u;
    value = value * 2654435769u;
    value = value ^ (value >> 16u);
    value = value * 2654435769u;
    value = value ^ (value >> 16u);
    value = value * 2654435769u;
    return value;
}

float random_float(inout uint seed) {
    seed = hash(seed);
    return float(seed) / 4294967295.0;
}

float random_range(inout uint seed, float lowest, float highest) {
    return lowest + random_float(seed) * (highest - lowest);
}

vec3 random_unit_sphere(inout uint seed) {
    float around = random_float(seed) * 6.28318530718;
    float down = acos(2.0 * random_float(seed) - 1.0);
    float sine = sin(down);
    return vec3(sine * cos(around), sine * sin(around), cos(down));
}

vec3 random_cone_direction(inout uint seed, vec3 direction, float angle) {
    float cosine = cos(angle);
    float height = random_range(seed, cosine, 1.0);
    float around = random_float(seed) * 6.28318530718;
    float radius = sqrt(1.0 - height * height);
    vec3 local = vec3(radius * cos(around), radius * sin(around), height);

    vec3 up = abs(direction.y) < 0.999 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
    vec3 tangent = normalize(cross(up, direction));
    vec3 bitangent = cross(direction, tangent);
    return tangent * local.x + bitangent * local.y + direction * local.z;
}

float sample_curve(vec4 curve[8], uint count, float at) {
    if (count == 0u) {
        return 0.0;
    }
    float previous_time = -1.0;
    float previous_value = 0.0;
    for (uint index = 0u; index < count; ++index) {
        vec4 entry = curve[index / 2u];
        bool first = (index % 2u) == 0u;
        float time = first ? entry.x : entry.z;
        float value = first ? entry.y : entry.w;
        if (at <= time) {
            if (previous_time < 0.0) {
                return value;
            }
            float span = time - previous_time;
            float blend = span > 0.0 ? (at - previous_time) / span : 0.0;
            return mix(previous_value, value, blend);
        }
        previous_time = time;
        previous_value = value;
    }
    return previous_value;
}

vec4 sample_gradient(EmitterData emitter, float at) {
    if (emitter.gradient_count == 0u) {
        return vec4(1.0);
    }
    vec4 first = emitter.color_gradient[0];
    float previous_time = first.x;
    vec4 previous_color = vec4(first.yzw, emitter.color_gradient[1].x);

    for (uint index = 0u; index < emitter.gradient_count; ++index) {
        vec4 head = emitter.color_gradient[index * 2u];
        vec4 tail = emitter.color_gradient[index * 2u + 1u];
        float time = head.x;
        vec4 color = vec4(head.yzw, tail.x);
        if (at <= time) {
            float blend = time > previous_time ? (at - previous_time) / (time - previous_time) : 0.0;
            return mix(previous_color, color, blend);
        }
        previous_time = time;
        previous_color = color;
    }
    return previous_color;
}

float value_noise(vec3 point) {
    float sum = (point.x + point.y + point.z) / 3.0;
    vec3 cell = floor(point + vec3(sum));
    float back = (cell.x + cell.y + cell.z) / 6.0;
    vec3 offset = point - cell + vec3(back);
    float squared = dot(offset, offset);
    return sin(squared * 12.9898 + squared * 78.233) * 0.5 + 0.5;
}

vec3 curl_noise(vec3 point, float seconds) {
    const float step_size = 0.01;
    vec3 at = point + vec3(seconds * 0.1);
    float along_x = value_noise(at + vec3(step_size, 0.0, 0.0))
        - value_noise(at - vec3(step_size, 0.0, 0.0));
    float along_y = value_noise(at + vec3(0.0, step_size, 0.0))
        - value_noise(at - vec3(0.0, step_size, 0.0));
    float along_z = value_noise(at + vec3(0.0, 0.0, step_size))
        - value_noise(at - vec3(0.0, 0.0, step_size));
    return vec3(along_y - along_z, along_z - along_x, along_x - along_y) / (2.0 * step_size);
}
