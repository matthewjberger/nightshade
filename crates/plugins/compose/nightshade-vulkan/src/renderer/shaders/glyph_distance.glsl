const float SPREAD = 8.0;

const uint GLYPH_COLORED = 1u;

vec4 shade_distance(
    float distance,
    float pixel,
    vec4 base_color,
    vec4 outline_color,
    float outline_width,
    float smoothing
) {
    float soft = 0.5 * pixel + smoothing;
    float fill = smoothstep(-soft, soft, distance);
    if (outline_width > 0.0 && outline_color.a > 0.0) {
        float border = smoothstep(-soft, soft, distance + outline_width);
        return vec4(
            mix(outline_color.rgb, base_color.rgb, fill),
            mix(border * outline_color.a, base_color.a, fill)
        );
    }
    return vec4(base_color.rgb, fill * base_color.a);
}
