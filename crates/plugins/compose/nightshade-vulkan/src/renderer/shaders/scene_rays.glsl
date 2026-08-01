
layout(set = 1, binding = 0) uniform texture2D scene_textures[];
layout(set = 1, binding = 1) uniform sampler scene_samplers[];
layout(set = 2, binding = 4) uniform accelerationStructureEXT world;

vec2 slot_uv_of(GpuMaterial material, int slot, vec2 first, vec2 second) {
    vec4 row_u = material.transforms[slot * 2];
    vec4 row_v = material.transforms[slot * 2 + 1];
    vec2 uv = uint(row_u.w) == 1u ? second : first;
    vec3 homogeneous = vec3(uv, 1.0);
    return vec2(dot(row_u.xyz, homogeneous), dot(row_v.xyz, homogeneous));
}

vec4 sample_slot_flat(uint slot, vec2 uv) {
    uint index = slot & 0xFFFFu;
    uint wrap_u = (slot >> 16u) & 3u;
    uint wrap_v = (slot >> 18u) & 3u;
    uint nearest = (slot >> 20u) & 1u;
    uint sampler_slot = nearest * 9u + wrap_u * 3u + wrap_v;
    return textureLod(
        sampler2D(scene_textures[nonuniformEXT(index)], scene_samplers[sampler_slot]),
        uv,
        0.0
    );
}

bool cut_surface_covers(rayQueryEXT query) {
    uint drawn = uint(rayQueryGetIntersectionInstanceCustomIndexEXT(query, false));
    uint primitive = uint(rayQueryGetIntersectionPrimitiveIndexEXT(query, false));
    vec2 weights = rayQueryGetIntersectionBarycentricsEXT(query, false);
    Object object = objects[drawn];
    if (object.material_index == NO_MATERIAL) {
        return true;
    }
    GpuMaterial material = materials[object.material_index];
    MeshRange range = mesh_ranges[object.mesh_slot];
    uint base = (object.flags & OBJECT_SKINNED) != 0u ? object.vertex_offset : range.vertex_offset;
    uint first = range.index_offset + primitive * 3u;
    Vertex a = vertices[base + indices[first]];
    Vertex b = vertices[base + indices[first + 1u]];
    Vertex c = vertices[base + indices[first + 2u]];
    vec3 share = vec3(1.0 - weights.x - weights.y, weights.x, weights.y);
    vec2 uv = a.tex_coords * share.x + b.tex_coords * share.y + c.tex_coords * share.z;
    vec2 uv_1 = a.tex_coords_1 * share.x + b.tex_coords_1 * share.y + c.tex_coords_1 * share.z;
    vec4 color = a.color * share.x + b.color * share.y + c.color * share.z;
    vec4 albedo = material.base_color * surface_tint(material, color) * object.tint;
    uint base_slot = material.textures[TEXTURE_BASE];
    if (base_slot != NO_TEXTURE) {
        albedo *= sample_slot_flat(
            base_slot,
            atlas_uv(material, slot_uv_of(material, TEXTURE_BASE, uv, uv_1), uv_1.x)
        );
    }
    uint alpha_mode = uint(material.alpha.y);
    float cutoff = alpha_mode == ALPHA_MODE_MASK ? material.alpha.x : 0.5;
    return albedo.a >= cutoff;
}

bool sun_is_blocked(vec3 from, vec3 toward, float step_off) {
    rayQueryEXT query;
    rayQueryInitializeEXT(
        query,
        world,
        gl_RayFlagsTerminateOnFirstHitEXT,
        0xffu,
        from,
        step_off,
        toward,
        frame.sun_shadow.w
    );
    while (rayQueryProceedEXT(query)) {
        if (rayQueryGetIntersectionTypeEXT(query, false)
                == gl_RayQueryCandidateIntersectionTriangleEXT
            && cut_surface_covers(query)) {
            rayQueryConfirmIntersectionEXT(query);
        }
    }
    return rayQueryGetIntersectionTypeEXT(query, true)
        != gl_RayQueryCommittedIntersectionNoneEXT;
}
