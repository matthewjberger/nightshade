
layout(std430, set = 0, binding = 0) buffer BoneTransforms { mat4 bone_transforms[]; };

vec4 turn_about(vec3 axis, float angle) {
    float half_angle = angle * 0.5;
    return vec4(axis * sin(half_angle), cos(half_angle));
}

vec3 turned(vec4 turn, vec3 point) {
    vec3 axis = turn.xyz;
    return point + 2.0 * cross(axis, cross(axis, point) + turn.w * point);
}

mat3 turn_matrix(vec4 turn) {
    float x = turn.x;
    float y = turn.y;
    float z = turn.z;
    float w = turn.w;
    return mat3(
        vec3(1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + w * z), 2.0 * (x * z - w * y)),
        vec3(2.0 * (x * y - w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + w * x)),
        vec3(2.0 * (x * z + w * y), 2.0 * (y * z - w * x), 1.0 - 2.0 * (x * x + y * y))
    );
}

vec4 turn_from_to(vec3 from_direction, vec3 to_direction) {
    vec3 source = normalize(from_direction);
    vec3 destination = normalize(to_direction);
    float along = dot(source, destination);
    if (along > 0.99999) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    if (along < -0.99999) {
        vec3 axis = cross(vec3(1.0, 0.0, 0.0), source);
        if (length(axis) < 0.0001) {
            axis = cross(vec3(0.0, 1.0, 0.0), source);
        }
        return vec4(normalize(axis), 0.0);
    }
    return turn_about(normalize(cross(source, destination)), acos(clamp(along, -1.0, 1.0)));
}

vec4 turn_weighted(vec4 turn, float weight) {
    vec4 shortest = turn.w < 0.0 ? -turn : turn;
    vec4 none = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 mixed = mix(none, shortest, weight);
    float size = dot(mixed, mixed);
    return size < 1.0e-6 ? none : mixed / sqrt(size);
}

vec4 turn_clamped(vec4 turn, float limit) {
    if (limit <= 0.0) {
        return turn;
    }
    vec4 shortest = turn.w < 0.0 ? -turn : turn;
    float angle = 2.0 * acos(clamp(shortest.w, -1.0, 1.0));
    if (angle <= limit) {
        return shortest;
    }
    float across = length(shortest.xyz);
    if (across < 1.0e-6) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    return turn_about(shortest.xyz / across, limit);
}

float angle_between(vec3 first, vec3 second) {
    float size = length(first) * length(second);
    if (size < 1.0e-6) {
        return 0.0;
    }
    return acos(clamp(dot(first, second) / size, -1.0, 1.0));
}

float angle_across(float first, float second, float opposite) {
    float size = 2.0 * first * second;
    if (size < 1.0e-6) {
        return 0.0;
    }
    return acos(clamp(
        (first * first + second * second - opposite * opposite) / size,
        -1.0,
        1.0
    ));
}

void turn_bone_about(uint index, vec3 pivot, vec4 turn) {
    mat4 standing = bone_transforms[index];
    vec3 position = standing[3].xyz;
    mat3 rotation = turn_matrix(turn);
    bone_transforms[index] = mat4(
        vec4(rotation * standing[0].xyz, 0.0),
        vec4(rotation * standing[1].xyz, 0.0),
        vec4(rotation * standing[2].xyz, 0.0),
        vec4(pivot + turned(turn, position - pivot), 1.0)
    );
}
