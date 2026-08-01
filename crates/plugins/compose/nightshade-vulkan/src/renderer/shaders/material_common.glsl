
float reflectance_from_ior(float ior) {
    float ratio = (ior - 1.0) / max(ior + 1.0, 1.0e-4);
    return ratio * ratio;
}

float visibility_smith(float normal_view, float normal_light, float roughness) {
    float alpha = roughness * roughness;
    float alpha_squared = alpha * alpha;
    float from_view = normal_light
        * sqrt(normal_view * normal_view * (1.0 - alpha_squared) + alpha_squared);
    float from_light = normal_view
        * sqrt(normal_light * normal_light * (1.0 - alpha_squared) + alpha_squared);
    return 0.5 / max(from_view + from_light, 1.0e-5);
}

float visibility_anisotropic(
    float normal_light,
    float normal_view,
    float bitangent_view,
    float tangent_view,
    float bitangent_light,
    float tangent_light,
    float along,
    float across
) {
    float from_view = normal_light
        * length(vec3(along * tangent_view, across * bitangent_view, normal_view));
    float from_light = normal_view
        * length(vec3(along * tangent_light, across * bitangent_light, normal_light));
    return clamp(0.5 / max(from_view + from_light, 1.0e-4), 0.0, 1.0);
}

const float FACING_SPREAD = 0.25;
const float FACING_SPREAD_MOST = 0.18;

float roughness_over_a_pixel(float roughness, vec3 normal) {
    vec3 across = dFdx(normal);
    vec3 down = dFdy(normal);
    float spread = FACING_SPREAD * (dot(across, across) + dot(down, down));
    float alpha = roughness * roughness;
    float widened = min(2.0 * spread, FACING_SPREAD_MOST);
    return sqrt(sqrt(clamp(alpha * alpha + widened, 0.0, 1.0)));
}

float specular_occlusion(float normal_view, float hidden, float roughness) {
    return clamp(
        pow(normal_view + hidden, exp2(-16.0 * roughness - 1.0)) - 1.0 + hidden,
        0.0,
        1.0
    );
}

float sheen_distribution(float normal_half, float roughness) {
    float alpha = max(roughness * roughness, 1.0e-4);
    float inverse = 1.0 / alpha;
    float cosine = normal_half * normal_half;
    float sine = max(1.0 - cosine, 0.0078125);
    return (2.0 + inverse) * pow(sine, inverse * 0.5) / (2.0 * PI);
}

float sheen_visibility(float normal_view, float normal_light) {
    return clamp(
        1.0 / (4.0 * (normal_light + normal_view - normal_light * normal_view)),
        0.0,
        1.0
    );
}

vec3 sheen_lobe(
    vec3 normal,
    vec3 to_view,
    vec3 to_light,
    vec3 color,
    float roughness
) {
    float normal_light = max(dot(normal, to_light), 0.0);
    if (normal_light <= 0.0) {
        return vec3(0.0);
    }
    vec3 half_vector = normalize(to_view + to_light);
    float normal_half = max(dot(normal, half_vector), 0.0);
    float normal_view = max(dot(normal, to_view), 1.0e-4);
    float distribution = sheen_distribution(normal_half, roughness);
    float visibility = sheen_visibility(normal_view, normal_light);
    return color * distribution * visibility * normal_light;
}

float clearcoat_lobe(
    vec3 normal,
    vec3 to_view,
    vec3 to_light,
    float roughness
) {
    float normal_light = max(dot(normal, to_light), 0.0);
    if (normal_light <= 0.0) {
        return 0.0;
    }
    vec3 half_vector = normalize(to_view + to_light);
    float normal_half = max(dot(normal, half_vector), 0.0);
    float normal_view = max(dot(normal, to_view), 1.0e-4);
    float view_half = max(dot(to_view, half_vector), 0.0);
    float distribution = distribution_ggx(normal_half, roughness);
    float visibility = visibility_smith(normal_view, normal_light, roughness);
    float fresnel = 0.04 + 0.96 * pow(1.0 - view_half, 5.0);
    return fresnel * distribution * visibility * normal_light;
}

float distribution_anisotropic(
    float normal_half,
    float tangent_half,
    float bitangent_half,
    float along,
    float across
) {
    float term = tangent_half * tangent_half / max(along * along, 1.0e-8)
        + bitangent_half * bitangent_half / max(across * across, 1.0e-8)
        + normal_half * normal_half;
    return 1.0 / (PI * along * across * term * term);
}

vec3 reflectance_from_fresnel(vec3 fresnel, vec3 grazing, float normal_view) {
    float away = clamp(1.0 - normal_view, 0.0, 1.0);
    float fifth = clamp(away * away * away * away * away, 0.0, 0.9999);
    return (fresnel - grazing * fifth) / (1.0 - fifth);
}

vec3 bending_from_reflectance(vec3 reflectance) {
    vec3 root = sqrt(clamp(reflectance, vec3(0.0), vec3(0.9999)));
    return (vec3(1.0) + root) / (vec3(1.0) - root);
}

vec3 reflectance_between(vec3 into, float from) {
    vec3 ratio = (into - vec3(from)) / (into + vec3(from));
    return ratio * ratio;
}

float reflectance_between_one(float into, float from) {
    float ratio = (into - from) / (into + from);
    return ratio * ratio;
}

vec3 film_response(float path, vec3 shift) {
    float phase = 2.0 * PI * path * 1.0e-9;
    vec3 strength = vec3(5.4856e-13, 4.4201e-13, 5.2481e-13);
    vec3 middle = vec3(1.6810e+06, 1.7953e+06, 2.2084e+06);
    vec3 spread = vec3(4.3278e+09, 9.3046e+09, 6.6121e+09);
    vec3 gathered = strength * sqrt(2.0 * PI * spread)
        * cos(middle * phase + shift) * exp(-phase * phase * spread);
    gathered.x += 9.7470e-14 * sqrt(2.0 * PI * 4.5282e+09)
        * cos(2.2399e+06 * phase + shift.x) * exp(-4.5282e+09 * phase * phase);
    gathered /= 1.0685e-7;
    mat3 to_display = mat3(
        vec3(3.2404542, -0.9692660, 0.0556434),
        vec3(-1.5371385, 1.8760108, -0.2040259),
        vec3(-0.4985314, 0.0415560, 1.0572252)
    );
    return to_display * gathered;
}

vec3 thin_film(float normal_view, float thickness, float film_ior, vec3 reflectance) {
    if (thickness <= 0.0) {
        return reflectance;
    }
    const float OUTSIDE = 1.0;
    float film = mix(OUTSIDE, film_ior, smoothstep(0.0, 0.03, thickness));
    float ratio = OUTSIDE / film;
    float sine_squared = ratio * ratio * (1.0 - normal_view * normal_view);
    float inside_squared = 1.0 - sine_squared;
    if (inside_squared < 0.0) {
        return vec3(1.0);
    }
    float inside = sqrt(inside_squared);

    float outer = reflectance_between_one(film, OUTSIDE);
    float away = clamp(1.0 - normal_view, 0.0, 1.0);
    float first = outer + (1.0 - outer) * away * away * away * away * away;
    float through = 1.0 - first;
    float shift_outer = film < OUTSIDE ? PI : 0.0;
    float shift_inner = PI - shift_outer;

    vec3 base = bending_from_reflectance(reflectance);
    vec3 under = reflectance_between(base, film);
    float leaving = clamp(1.0 - inside, 0.0, 1.0);
    vec3 second = under
        + (vec3(1.0) - under) * leaving * leaving * leaving * leaving * leaving;
    vec3 shift_under = vec3(
        base.x < film ? PI : 0.0,
        base.y < film ? PI : 0.0,
        base.z < film ? PI : 0.0
    );

    float path = 2.0 * film * thickness * inside;
    vec3 shift = vec3(shift_inner) + shift_under;

    vec3 round_trip = clamp(vec3(first) * second, vec3(1.0e-5), vec3(0.9999));
    vec3 root = sqrt(round_trip);
    vec3 carried = (through * through) * second / (vec3(1.0) - round_trip);

    vec3 summed = vec3(first) + carried;
    vec3 term = (carried - vec3(through)) * root;
    summed += term * 2.0 * film_response(path, shift);
    term *= root;
    summed += term * 2.0 * film_response(2.0 * path, 2.0 * shift);
    return max(summed, vec3(0.0));
}

vec3 reflection_in_a_box(
    vec3 direction,
    vec3 position,
    vec3 lowest,
    vec3 highest
) {
    vec3 toward_high = (highest - position) / direction;
    vec3 toward_low = (lowest - position) / direction;
    vec3 furthest = max(toward_high, toward_low);
    float reach = min(min(furthest.x, furthest.y), furthest.z);
    vec3 center = (lowest + highest) * 0.5;
    return normalize(position + direction * reach - center);
}

float reveal_noise_at(vec3 point) {
    return fract(sin(dot(point, vec3(12.9898, 78.233, 37.719))) * 43758.5453);
}
