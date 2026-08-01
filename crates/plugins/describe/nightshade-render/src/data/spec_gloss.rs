const DIELECTRIC_SPECULAR: f32 = 0.04;
const EPSILON: f32 = 1e-6;

fn solve_metallic(diffuse: f32, specular: f32, one_minus_specular_strength: f32) -> f32 {
    if specular < DIELECTRIC_SPECULAR {
        return 0.0;
    }
    let a = DIELECTRIC_SPECULAR;
    let b = diffuse * one_minus_specular_strength / (1.0 - DIELECTRIC_SPECULAR) + specular
        - 2.0 * DIELECTRIC_SPECULAR;
    let c = DIELECTRIC_SPECULAR - specular;
    let discriminant = (b * b - 4.0 * a * c).max(0.0);
    ((-b + discriminant.sqrt()) / (2.0 * a)).clamp(0.0, 1.0)
}

pub fn spec_gloss_to_metallic_roughness(diffuse: [f32; 4], specular: [f32; 3]) -> ([f32; 3], f32) {
    let max_specular = specular[0].max(specular[1]).max(specular[2]);
    let one_minus_specular_strength = 1.0 - max_specular;
    let max_diffuse = diffuse[0].max(diffuse[1]).max(diffuse[2]);
    let metallic = solve_metallic(max_diffuse, max_specular, one_minus_specular_strength);

    let base_from_diffuse = |channel: f32| -> f32 {
        channel * one_minus_specular_strength
            / (1.0 - DIELECTRIC_SPECULAR).max(EPSILON)
            / (1.0 - metallic).max(EPSILON)
    };
    let base_from_specular = |channel: f32| -> f32 {
        (channel - DIELECTRIC_SPECULAR * (1.0 - metallic)) / metallic.max(EPSILON)
    };

    let blend = metallic * metallic;
    let lerp = |start: f32, end: f32, factor: f32| start + (end - start) * factor;

    let base_color = [
        lerp(
            base_from_diffuse(diffuse[0]),
            base_from_specular(specular[0]),
            blend,
        )
        .clamp(0.0, 1.0),
        lerp(
            base_from_diffuse(diffuse[1]),
            base_from_specular(specular[1]),
            blend,
        )
        .clamp(0.0, 1.0),
        lerp(
            base_from_diffuse(diffuse[2]),
            base_from_specular(specular[2]),
            blend,
        )
        .clamp(0.0, 1.0),
    ];

    (base_color, metallic)
}
