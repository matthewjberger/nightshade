struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    color: vec4<f32>,
    size_lifetime: vec4<f32>,
    emitter_data: vec4<f32>,
    physics: vec4<f32>,
    color_start: vec4<f32>,
    color_end: vec4<f32>,
}

struct EmitterData {
    position: vec4<f32>,
    direction: vec4<f32>,
    velocity_range: vec4<f32>,
    lifetime_range: vec4<f32>,
    size_range: vec4<f32>,
    gravity: vec4<f32>,
    color_gradient: array<vec4<f32>, 16>,
    gradient_count: u32,
    spawn_count: u32,
    emitter_id: u32,
    shape_type: u32,
    shape_params: vec4<f32>,
    turbulence: vec4<f32>,
    emissive_strength: f32,
    drag: f32,
    emitter_type: u32,
    texture_index: u32,
    size_curve: array<vec4<f32>, 8>,
    size_curve_count: u32,
    size_curve_pad: array<u32, 3>,
    opacity_curve: array<vec4<f32>, 8>,
    opacity_curve_count: u32,
    opacity_curve_pad: array<u32, 3>,
}

struct SimParams {
    delta_time: f32,
    time: f32,
    max_particles: u32,
    _padding: u32,
}

struct DrawIndirect {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
}

@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<storage, read> emitters: array<EmitterData>;

@group(0) @binding(2)
var<uniform> params: SimParams;

@group(0) @binding(3)
var<storage, read_write> free_indices: array<u32>;

@group(0) @binding(4)
var<storage, read_write> free_count: atomic<u32>;

@group(0) @binding(5)
var<storage, read_write> alive_indices: array<u32>;

@group(0) @binding(6)
var<storage, read_write> alive_count: atomic<u32>;

@group(0) @binding(7)
var<storage, read_write> draw_indirect: DrawIndirect;

fn hash(seed: u32) -> u32 {
    var s = seed;
    s = s ^ 2747636419u;
    s = s * 2654435769u;
    s = s ^ (s >> 16u);
    s = s * 2654435769u;
    s = s ^ (s >> 16u);
    s = s * 2654435769u;
    return s;
}

fn random_float(seed: ptr<function, u32>) -> f32 {
    *seed = hash(*seed);
    return f32(*seed) / 4294967295.0;
}

fn random_range(seed: ptr<function, u32>, min_val: f32, max_val: f32) -> f32 {
    return min_val + random_float(seed) * (max_val - min_val);
}

fn random_unit_sphere(seed: ptr<function, u32>) -> vec3<f32> {
    let theta = random_float(seed) * 6.28318530718;
    let phi = acos(2.0 * random_float(seed) - 1.0);
    let sin_phi = sin(phi);
    return vec3<f32>(sin_phi * cos(theta), sin_phi * sin(theta), cos(phi));
}

fn random_cone_direction(seed: ptr<function, u32>, direction: vec3<f32>, angle: f32) -> vec3<f32> {
    let cos_angle = cos(angle);
    let z = random_range(seed, cos_angle, 1.0);
    let phi = random_float(seed) * 6.28318530718;
    let sin_theta = sqrt(1.0 - z * z);

    let local_dir = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), z);

    let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(direction.y) < 0.999);
    let tangent = normalize(cross(up, direction));
    let bitangent = cross(direction, tangent);

    return tangent * local_dir.x + bitangent * local_dir.y + direction * local_dir.z;
}

fn sample_value_curve(curve: array<vec4<f32>, 8>, count: u32, t: f32) -> f32 {
    if (count == 0u) {
        return 0.0;
    }
    var prev_time = -1.0;
    var prev_value = 0.0;
    for (var index = 0u; index < count; index++) {
        let slot_index = index / 2u;
        let lane = (index % 2u) * 2u;
        let entry = curve[slot_index];
        let curr_time = select(entry.z, entry.x, lane == 0u);
        let curr_value = select(entry.w, entry.y, lane == 0u);
        if (t <= curr_time) {
            if (prev_time < 0.0) {
                return curr_value;
            }
            let span = curr_time - prev_time;
            let blend = select(0.0, (t - prev_time) / span, span > 0.0);
            return mix(prev_value, curr_value, blend);
        }
        prev_time = curr_time;
        prev_value = curr_value;
    }
    return prev_value;
}

fn sample_gradient(emitter: EmitterData, t: f32) -> vec4<f32> {
    let count = emitter.gradient_count;
    if (count == 0u) {
        return vec4<f32>(1.0);
    }

    var prev_color = emitter.color_gradient[0];
    var prev_time = prev_color.x;
    prev_color = vec4<f32>(prev_color.yzw, emitter.color_gradient[1].x);

    for (var index = 0u; index < count; index++) {
        let entry_index = index * 2u;
        let time_and_rgb = emitter.color_gradient[entry_index];
        let a_and_padding = emitter.color_gradient[entry_index + 1u];

        let curr_time = time_and_rgb.x;
        let curr_color = vec4<f32>(time_and_rgb.yzw, a_and_padding.x);

        if (t <= curr_time) {
            let blend = select(0.0, (t - prev_time) / (curr_time - prev_time), curr_time > prev_time);
            return mix(prev_color, curr_color, blend);
        }

        prev_time = curr_time;
        prev_color = curr_color;
    }

    return prev_color;
}

fn simplex_noise_3d(position: vec3<f32>) -> f32 {
    let s = (position.x + position.y + position.z) / 3.0;
    let xs = position.x + s;
    let ys = position.y + s;
    let zs = position.z + s;

    let adjusted_pos = floor(vec3<f32>(xs, ys, zs));
    let g = (adjusted_pos.x + adjusted_pos.y + adjusted_pos.z) / 6.0;
    let x0 = position.x - adjusted_pos.x + g;
    let y0 = position.y - adjusted_pos.y + g;
    let z0 = position.z - adjusted_pos.z + g;

    var sum = x0 * x0 + y0 * y0 + z0 * z0;
    return sin(sum * 12.9898 + sum * 78.233) * 0.5 + 0.5;
}

fn curl_noise(position: vec3<f32>, time: f32) -> vec3<f32> {
    let epsilon = 0.01;
    let pos = position + vec3<f32>(time * 0.1);

    let dx = simplex_noise_3d(pos + vec3<f32>(epsilon, 0.0, 0.0)) - simplex_noise_3d(pos - vec3<f32>(epsilon, 0.0, 0.0));
    let dy = simplex_noise_3d(pos + vec3<f32>(0.0, epsilon, 0.0)) - simplex_noise_3d(pos - vec3<f32>(0.0, epsilon, 0.0));
    let dz = simplex_noise_3d(pos + vec3<f32>(0.0, 0.0, epsilon)) - simplex_noise_3d(pos - vec3<f32>(0.0, 0.0, epsilon));

    return vec3<f32>(dy - dz, dz - dx, dx - dy) / (2.0 * epsilon);
}

@compute @workgroup_size(256)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.max_particles) {
        return;
    }

    var particle = particles[index];

    let is_alive = particle.size_lifetime.w > 0.0;
    if (!is_alive) {
        return;
    }

    let age = particle.size_lifetime.z;
    let lifetime = particle.size_lifetime.w;
    let new_age = age + params.delta_time;

    if (new_age >= lifetime) {
        particle.size_lifetime.w = 0.0;
        let free_slot = atomicAdd(&free_count, 1u);
        free_indices[free_slot] = index;
        particles[index] = particle;
        return;
    }

    let gravity_y = particle.physics.x;
    let drag = particle.physics.y;
    let turbulence_strength = particle.physics.z;
    let turbulence_freq = particle.physics.w;
    let emissive_strength = particle.emitter_data.z;

    var turbulence = vec3<f32>(0.0);
    if (turbulence_strength > 0.0) {
        turbulence = curl_noise(particle.position.xyz * turbulence_freq, params.time) * turbulence_strength;
    }

    let drag_factor = 1.0 - drag * params.delta_time;
    var velocity = particle.velocity.xyz * drag_factor;
    velocity = velocity + vec3<f32>(0.0, gravity_y, 0.0) * params.delta_time;
    velocity = velocity + turbulence * params.delta_time;

    let new_position = particle.position.xyz + velocity * params.delta_time;

    let life_ratio = new_age / lifetime;
    let size_start = particle.size_lifetime.x;
    let size_end = particle.size_lifetime.y;
    let emitter_index = u32(particle.velocity.w);
    let emitter = emitters[emitter_index];
    var current_size = mix(size_start, size_end, life_ratio);
    if (emitter.size_curve_count > 0u) {
        current_size = sample_value_curve(emitter.size_curve, emitter.size_curve_count, life_ratio);
    }

    var color = mix(particle.color_start, particle.color_end, life_ratio);
    if (emitter.opacity_curve_count > 0u) {
        color.w = color.w
            * sample_value_curve(emitter.opacity_curve, emitter.opacity_curve_count, life_ratio);
    }

    particle.position = vec4<f32>(new_position, 1.0);
    particle.velocity = vec4<f32>(velocity, f32(emitter_index));
    particle.color = color;
    particle.size_lifetime.z = new_age;
    particle.emitter_data.y = current_size;

    let alive_slot = atomicAdd(&alive_count, 1u);
    alive_indices[alive_slot] = index;
    atomicAdd(&draw_indirect.instance_count, 1u);

    particles[index] = particle;
}

@compute @workgroup_size(1)
fn reset_counters() {
    atomicStore(&alive_count, 0u);
    atomicStore(&draw_indirect.instance_count, 0u);
}

@compute @workgroup_size(256)
fn spawn(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>
) {
    let emitter_index = workgroup_id.x;
    let spawn_index = local_id.x;

    let emitter = emitters[emitter_index];
    if (spawn_index >= emitter.spawn_count) {
        return;
    }

    let old_free_count = atomicSub(&free_count, 1u);
    if (old_free_count == 0u) {
        atomicAdd(&free_count, 1u);
        return;
    }

    let particle_index = free_indices[old_free_count - 1u];

    var seed = hash(particle_index * 1973u + bitcast<u32>(params.time) + spawn_index * 7919u + emitter_index * 6997u);

    var spawn_offset = vec3<f32>(0.0);
    var spawn_direction = emitter.direction.xyz;

    let shape_type = emitter.shape_type;
    if (shape_type == 1u) {
        let radius = emitter.shape_params.x;
        spawn_offset = random_unit_sphere(&seed) * radius * random_float(&seed);
        spawn_direction = random_unit_sphere(&seed);
    } else if (shape_type == 2u) {
        let angle = emitter.shape_params.x;
        let height = emitter.shape_params.y;
        spawn_direction = random_cone_direction(&seed, emitter.direction.xyz, angle);
        spawn_offset = emitter.direction.xyz * random_float(&seed) * height;
    } else if (shape_type == 3u) {
        let half_extents = emitter.shape_params.xyz;
        spawn_offset = vec3<f32>(
            random_range(&seed, -half_extents.x, half_extents.x),
            random_range(&seed, -half_extents.y, half_extents.y),
            random_range(&seed, -half_extents.z, half_extents.z)
        );
    }

    let spread = emitter.velocity_range.z;
    if (spread > 0.0) {
        spawn_direction = random_cone_direction(&seed, spawn_direction, spread);
    }

    let position = emitter.position.xyz + spawn_offset;
    let speed = random_range(&seed, emitter.velocity_range.x, emitter.velocity_range.y);
    let velocity = spawn_direction * speed;
    let lifetime = random_range(&seed, emitter.lifetime_range.x, emitter.lifetime_range.y);
    let size_start = emitter.size_range.x;
    let size_end = emitter.size_range.y;

    let color_start = sample_gradient(emitter, 0.15);
    let color_end = sample_gradient(emitter, 0.9);

    var particle: Particle;
    particle.position = vec4<f32>(position, 1.0);
    particle.velocity = vec4<f32>(velocity, f32(emitter_index));
    particle.color = color_start;
    particle.size_lifetime = vec4<f32>(size_start, size_end, 0.0, lifetime);
    particle.emitter_data = vec4<f32>(f32(emitter.texture_index), size_start, emitter.emissive_strength, f32(emitter.emitter_type));
    particle.physics = vec4<f32>(emitter.gravity.y, emitter.drag, emitter.turbulence.x, emitter.turbulence.y);
    particle.color_start = color_start;
    particle.color_end = color_end;

    particles[particle_index] = particle;
}
