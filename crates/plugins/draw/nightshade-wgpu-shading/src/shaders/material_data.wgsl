#define_import_path nightshade_wgpu::material_data


const MATERIAL_REVEAL_DISABLED: f32 = 1.0e30;

struct TextureTransform {
    row_u: vec4<f32>,
    row_v: vec4<f32>,
    drift: vec4<f32>,
};

struct Material {
    base_color: vec4<f32>,
    emissive_factor: vec3<f32>,
    alpha_mode: u32,
    alpha_cutoff: f32,
    base_layer: u32,
    emissive_layer: u32,
    normal_layer: u32,
    metallic_roughness_layer: u32,
    occlusion_layer: u32,
    normal_scale: f32,
    occlusion_strength: f32,
    roughness: f32,
    metallic: f32,
    unlit: u32,
    normal_map_flags: u32,
    transmission_factor: f32,
    transmission_layer: u32,
    thickness: f32,
    thickness_layer: u32,
    attenuation_color: vec3<f32>,
    attenuation_distance: f32,
    ior: f32,
    specular_factor: f32,
    sky_light: u32,
    vertex_occlusion: u32,
    specular_color_factor: vec3<f32>,
    specular_layer: u32,
    specular_color_layer: u32,
    emissive_strength: f32,
    atlas_cells: vec2<f32>,
    base_transform: TextureTransform,
    emissive_transform: TextureTransform,
    normal_transform: TextureTransform,
    metallic_roughness_transform: TextureTransform,
    occlusion_transform: TextureTransform,
    transmission_transform: TextureTransform,
    thickness_transform: TextureTransform,
    specular_transform: TextureTransform,
    specular_color_transform: TextureTransform,
    clearcoat_factor: f32,
    clearcoat_roughness_factor: f32,
    clearcoat_normal_scale: f32,
    clearcoat_layer: u32,
    clearcoat_roughness_layer: u32,
    clearcoat_normal_layer: u32,
    _pad_clearcoat: vec2<u32>,
    clearcoat_transform: TextureTransform,
    clearcoat_roughness_transform: TextureTransform,
    clearcoat_normal_transform: TextureTransform,
    sheen_color_factor: vec3<f32>,
    sheen_roughness_factor: f32,
    sheen_color_layer: u32,
    _pad_sheen_word: u32,
    _pad_sheen: vec2<u32>,
    sheen_color_transform: TextureTransform,
    sheen_roughness_transform: TextureTransform,
    iridescence_factor: f32,
    iridescence_ior: f32,
    iridescence_thickness_minimum: f32,
    iridescence_thickness_maximum: f32,
    iridescence_layer: u32,
    _pad_iridescence_word: u32,
    _pad_iridescence: vec2<u32>,
    iridescence_transform: TextureTransform,
    iridescence_thickness_transform: TextureTransform,
    anisotropy_strength: f32,
    anisotropy_rotation_cos: f32,
    anisotropy_rotation_sin: f32,
    anisotropy_layer: u32,
    anisotropy_transform: TextureTransform,
    dispersion: f32,
    diffuse_transmission_factor: f32,
    _pad_diffuse_transmission_word: u32,
    diffuse_transmission_color_layer: u32,
    diffuse_transmission_color_factor: vec3<f32>,
    blend_opaque_alpha_threshold: f32,
    diffuse_transmission_transform: TextureTransform,
    diffuse_transmission_color_transform: TextureTransform,
    detail_base_layer: u32,
    detail_normal_layer: u32,
    height_layer: u32,
    detail_uv_scale: f32,
    parallax_scale: f32,
    _pad_detail_a: u32,
    _pad_detail_b: u32,
    _pad_detail_c: u32,
    reflection_box_min: vec3<f32>,
    _pad_box_min: f32,
    reflection_box_max: vec3<f32>,
    _pad_box_max: f32,
    reveal_normal: vec3<f32>,
    reveal_front: f32,
    reveal_noise_scale: vec3<f32>,
    reveal_noise_amplitude: f32,
    reveal_seam_color: vec3<f32>,
    reveal_seam_width: f32,
    reveal_seam_strength: f32,
    _pad_reveal_a: f32,
    _pad_reveal_b: f32,
    _pad_reveal_c: f32,
};

fn reveal_mod289(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn reveal_mod289_4(x: vec4<f32>) -> vec4<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn reveal_permute(x: vec4<f32>) -> vec4<f32> {
    return reveal_mod289_4(((x * 34.0) + 1.0) * x);
}

fn reveal_taylor_inv_sqrt(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn reveal_noise(v: vec3<f32>) -> f32 {
    let C = vec2<f32>(1.0 / 6.0, 1.0 / 3.0);
    let D = vec4<f32>(0.0, 0.5, 1.0, 2.0);

    var i = floor(v + dot(v, vec3<f32>(C.y, C.y, C.y)));
    let x0 = v - i + dot(i, vec3<f32>(C.x, C.x, C.x));

    let g = step(x0.yzx, x0.xyz);
    let l = 1.0 - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    let x1 = x0 - i1 + C.x;
    let x2 = x0 - i2 + C.y;
    let x3 = x0 - D.yyy;

    i = reveal_mod289(i);
    let p = reveal_permute(reveal_permute(reveal_permute(
        i.z + vec4<f32>(0.0, i1.z, i2.z, 1.0))
        + i.y + vec4<f32>(0.0, i1.y, i2.y, 1.0))
        + i.x + vec4<f32>(0.0, i1.x, i2.x, 1.0));

    let n_ = 0.142857142857;
    let ns = n_ * D.wyz - D.xzx;

    let j = p - 49.0 * floor(p * ns.z * ns.z);

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7.0 * x_);

    let x = x_ * ns.x + ns.y;
    let y = y_ * ns.x + ns.y;
    let h = 1.0 - abs(x) - abs(y);

    let b0 = vec4<f32>(x.xy, y.xy);
    let b1 = vec4<f32>(x.zw, y.zw);

    let s0 = floor(b0) * 2.0 + 1.0;
    let s1 = floor(b1) * 2.0 + 1.0;
    let sh = -step(h, vec4<f32>(0.0));

    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3<f32>(a0.xy, h.x);
    var p1 = vec3<f32>(a0.zw, h.y);
    var p2 = vec3<f32>(a1.xy, h.z);
    var p3 = vec3<f32>(a1.zw, h.w);

    let norm = reveal_taylor_inv_sqrt(vec4<f32>(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;

    var m = max(0.6 - vec4<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), vec4<f32>(0.0));
    m = m * m;
    return 42.0 * dot(m * m, vec4<f32>(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}

fn material_reveal_distance(material: Material, world_position: vec3<f32>) -> f32 {
    if material.reveal_front >= MATERIAL_REVEAL_DISABLED {
        return -1.0;
    }
    let axis = normalize(material.reveal_normal);
    let wobble = reveal_noise(world_position * material.reveal_noise_scale)
        * material.reveal_noise_amplitude;
    return dot(world_position, axis) - (material.reveal_front + wobble);
}

fn material_reveal_seam(material: Material, reveal_distance: f32) -> vec3<f32> {
    if material.reveal_seam_width <= 0.0 || material.reveal_seam_strength <= 0.0 {
        return vec3<f32>(0.0);
    }
    let falloff = 1.0 - smoothstep(0.0, material.reveal_seam_width, -reveal_distance);
    return material.reveal_seam_color * (falloff * falloff * material.reveal_seam_strength);
}

