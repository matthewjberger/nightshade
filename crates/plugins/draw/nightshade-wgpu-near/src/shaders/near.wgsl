struct Settings {
    origin: vec4<f32>,
    shape: vec4<f32>,
    counts: vec4<u32>,
}

@group(0) @binding(0) var<uniform> settings: Settings;
@group(0) @binding(1) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> weights: array<f32>;
@group(0) @binding(3) var<storage, read> asks: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> cell_counts: array<atomic<u32>>;
@group(0) @binding(5) var<storage, read_write> cell_items: array<u32>;
@group(0) @binding(6) var<storage, read_write> answers: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read_write> pushes: array<vec4<f32>>;

fn dimension() -> u32 {
    return settings.counts.y;
}

fn cell_axis(value: f32, origin: f32) -> u32 {
    let side = i32(dimension());
    let cell = i32(floor((value - origin) / settings.shape.x));
    return u32(clamp(cell, 0, side - 1));
}

fn cell_of(position: vec3<f32>) -> u32 {
    let side = dimension();
    let x = cell_axis(position.x, settings.origin.x);
    let y = cell_axis(position.y, settings.origin.y);
    let z = cell_axis(position.z, settings.origin.z);
    return (z * side + y) * side + x;
}

@compute @workgroup_size(64)
fn clear_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= settings.counts.z {
        return;
    }
    atomicStore(&cell_counts[id.x], 0u);
}

@compute @workgroup_size(64)
fn fill_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= settings.counts.x {
        return;
    }
    let cell = cell_of(points[id.x].xyz);
    let slot = atomicAdd(&cell_counts[cell], 1u);
    let capacity = settings.counts.w;
    if slot < capacity {
        cell_items[cell * capacity + slot] = id.x;
    }
}

@compute @workgroup_size(64)
fn answer_asks(@builtin(global_invocation_id) id: vec3<u32>) {
    let asked = arrayLength(&asks);
    if id.x >= asked {
        return;
    }
    let ask = asks[id.x];
    let point = ask.xyz;
    let radius = ask.w;
    let side = i32(dimension());
    let capacity = settings.counts.w;
    let reach = i32(ceil(radius / settings.shape.x));
    let home = vec3<i32>(
        i32(cell_axis(point.x, settings.origin.x)),
        i32(cell_axis(point.y, settings.origin.y)),
        i32(cell_axis(point.z, settings.origin.z)),
    );

    var count = 0u;
    var nearest = -1.0;
    var closest = radius;

    for (var z = home.z - reach; z <= home.z + reach; z = z + 1) {
        if z < 0 || z >= side { continue; }
        for (var y = home.y - reach; y <= home.y + reach; y = y + 1) {
            if y < 0 || y >= side { continue; }
            for (var x = home.x - reach; x <= home.x + reach; x = x + 1) {
                if x < 0 || x >= side { continue; }
                let cell = (u32(z) * u32(side) + u32(y)) * u32(side) + u32(x);
                let held = min(atomicLoad(&cell_counts[cell]), capacity);
                for (var slot = 0u; slot < held; slot = slot + 1u) {
                    let index = cell_items[cell * capacity + slot];
                    let there = points[index].xyz;
                    let apart = distance(point, there);
                    if apart > radius {
                        continue;
                    }
                    count = count + 1u;
                    if apart < closest || nearest < 0.0 {
                        closest = apart;
                        nearest = f32(index);
                    }
                }
            }
        }
    }

    answers[id.x] = vec4<f32>(f32(count), nearest, closest, 0.0);
}

@compute @workgroup_size(64)
fn push_apart(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= settings.counts.x {
        return;
    }
    let weight = weights[id.x];
    if weight <= 0.0 {
        pushes[id.x] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return;
    }
    let here = points[id.x].xyz;
    let room = settings.shape.y;
    let side = i32(dimension());
    let capacity = settings.counts.w;
    let reach = i32(ceil(room / settings.shape.x));
    let home = vec3<i32>(
        i32(cell_axis(here.x, settings.origin.x)),
        i32(cell_axis(here.y, settings.origin.y)),
        i32(cell_axis(here.z, settings.origin.z)),
    );

    var away = vec3<f32>(0.0, 0.0, 0.0);
    for (var z = home.z - reach; z <= home.z + reach; z = z + 1) {
        if z < 0 || z >= side { continue; }
        for (var y = home.y - reach; y <= home.y + reach; y = y + 1) {
            if y < 0 || y >= side { continue; }
            for (var x = home.x - reach; x <= home.x + reach; x = x + 1) {
                if x < 0 || x >= side { continue; }
                let cell = (u32(z) * u32(side) + u32(y)) * u32(side) + u32(x);
                let held = min(atomicLoad(&cell_counts[cell]), capacity);
                for (var slot = 0u; slot < held; slot = slot + 1u) {
                    let index = cell_items[cell * capacity + slot];
                    if index == id.x {
                        continue;
                    }
                    if weights[index] <= 0.0 {
                        continue;
                    }
                    let there = points[index].xyz;
                    let apart = distance(here, there);
                    if apart >= room || apart <= 0.0001 {
                        continue;
                    }
                    let share = (room - apart) / room;
                    away = away + normalize(here - there) * share;
                }
            }
        }
    }
    pushes[id.x] = vec4<f32>(away * weight, 0.0);
}
