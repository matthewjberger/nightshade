#version 460
#extension GL_EXT_scalar_block_layout : require
#extension GL_GOOGLE_include_directive : require

#include "sky_common.glsl"
#include "sky_view.glsl"

layout(set = 0, binding = 1) uniform samplerCube environment;
layout(set = 0, binding = 2) uniform sampler3D air;

layout(location = 0) in vec3 world_direction;
layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_normal;

#include "sky_paint.glsl"

void main() {
    out_normal = vec4(0.0);
    vec3 direction = normalize(world_direction);
    float seconds = sky.settings.x;
    float hour = sky.settings.y;
    uint atmosphere = uint(sky.settings.z);

    vec3 color;
    if (atmosphere == ATMOSPHERE_CLOUDY) {
        color = cloudy_sky(direction, seconds);
    } else if (atmosphere == ATMOSPHERE_SPACE) {
        color = space_sky(direction, seconds);
    } else if (atmosphere == ATMOSPHERE_NEBULA) {
        color = nebula_sky(direction, seconds);
    } else if (atmosphere == ATMOSPHERE_SUNSET) {
        color = sunset_sky(direction, seconds);
    } else if (atmosphere == ATMOSPHERE_DAY_NIGHT) {
        color = day_night_sky(direction, seconds, hour);
    } else if (atmosphere == ATMOSPHERE_HDR) {
        color = textureLod(environment, direction, sky.settings.w).rgb;
    } else if (atmosphere == ATMOSPHERE_NONE) {
        color = sky.fog_color.rgb;
    } else {
        color = sky_gradient(direction);
    }

    if (sky.air.z > 0.0) {
        vec4 through = texture(air, vec3(gl_FragCoord.xy / sky.air.xy, 1.0));
        color = color * through.a + through.rgb;
    }

    out_color = vec4(clamp(color, vec3(0.0), vec3(65000.0)), 1.0);
}
