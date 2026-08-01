use super::*;

pub(super) fn build_materials(
    doc: &Document,
    namespace: &str,
    image_hashes: &[u64],
    image_index_to_id: &HashMap<usize, SourceImageId>,
    texture_specs: &mut TextureSpecMap,
) -> (Vec<Material>, HashMap<usize, usize>) {
    let mut materials = Vec::new();
    let mut gltf_material_index_to_deduplicated_index: HashMap<usize, usize> = HashMap::new();

    for material in doc.materials() {
        let pbr = material.pbr_metallic_roughness();
        let base_color = pbr.base_color_factor();
        let emissive = material.emissive_factor();
        let mut base_color_array = [base_color[0], base_color[1], base_color[2], base_color[3]];
        let mut roughness_value = pbr.roughness_factor();
        let mut metallic_value = pbr.metallic_factor();

        let (mut base_texture, mut base_texture_transform) = pbr
            .base_color_texture()
            .map(|tex| {
                let transform = texture_transform_from_info(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Color,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let (emissive_texture, emissive_texture_transform) = material
            .emissive_texture()
            .map(|tex| {
                let transform = texture_transform_from_info(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Color,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let (normal_texture, normal_texture_transform) = material
            .normal_texture()
            .map(|tex| {
                let transform = texture_transform_from_normal(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Linear,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let normal_scale = material
            .normal_texture()
            .map(|tex| tex.scale())
            .unwrap_or(1.0);

        let (mut metallic_roughness_texture, mut metallic_roughness_texture_transform) = pbr
            .metallic_roughness_texture()
            .map(|tex| {
                let transform = texture_transform_from_info(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Linear,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let (occlusion_texture, occlusion_texture_transform) = material
            .occlusion_texture()
            .map(|tex| {
                let transform = texture_transform_from_occlusion(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Linear,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let occlusion_strength = material
            .occlusion_texture()
            .map(|tex| tex.strength())
            .unwrap_or(1.0);

        let transmission_factor = material
            .transmission()
            .map(|t| t.transmission_factor())
            .unwrap_or(0.0);

        let (transmission_texture, transmission_texture_transform) = material
            .transmission()
            .and_then(|t| {
                t.transmission_texture().map(|tex| {
                    let transform = texture_transform_from_info(&tex);
                    (
                        register_texture(
                            tex.texture(),
                            TextureUsage::Linear,
                            image_hashes,
                            image_index_to_id,
                            texture_specs,
                        ),
                        transform,
                    )
                })
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let thickness = material
            .volume()
            .map(|v| v.thickness_factor())
            .unwrap_or(0.0);

        let (thickness_texture, thickness_texture_transform) = material
            .volume()
            .and_then(|v| {
                v.thickness_texture().map(|tex| {
                    let transform = texture_transform_from_info(&tex);
                    (
                        register_texture(
                            tex.texture(),
                            TextureUsage::Linear,
                            image_hashes,
                            image_index_to_id,
                            texture_specs,
                        ),
                        transform,
                    )
                })
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let attenuation_color = material
            .volume()
            .map(|v| v.attenuation_color())
            .unwrap_or([1.0, 1.0, 1.0]);

        let attenuation_distance = material
            .volume()
            .map(|v| v.attenuation_distance())
            .unwrap_or(f32::INFINITY);

        let ior = material.ior().unwrap_or(1.5);

        let specular_factor = material
            .specular()
            .map(|s| s.specular_factor())
            .unwrap_or(1.0);

        let specular_color_factor = material
            .specular()
            .map(|s| s.specular_color_factor())
            .unwrap_or([1.0, 1.0, 1.0]);

        let (specular_texture, specular_texture_transform) = material
            .specular()
            .and_then(|s| {
                s.specular_texture().map(|tex| {
                    let transform = texture_transform_from_info(&tex);
                    (
                        register_texture(
                            tex.texture(),
                            TextureUsage::Linear,
                            image_hashes,
                            image_index_to_id,
                            texture_specs,
                        ),
                        transform,
                    )
                })
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let (specular_color_texture, specular_color_texture_transform) = material
            .specular()
            .and_then(|s| {
                s.specular_color_texture().map(|tex| {
                    let transform = texture_transform_from_info(&tex);
                    (
                        register_texture(
                            tex.texture(),
                            TextureUsage::Color,
                            image_hashes,
                            image_index_to_id,
                            texture_specs,
                        ),
                        transform,
                    )
                })
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let emissive_strength = material.emissive_strength().unwrap_or(1.0);

        if let Some(spec_gloss) = material.pbr_specular_glossiness() {
            let diffuse_factor = spec_gloss.diffuse_factor();
            let specular_factor_sg = spec_gloss.specular_factor();
            let glossiness_factor = spec_gloss.glossiness_factor();

            let (converted_base_rgb, converted_metallic) =
                nightshade_render::data::spec_gloss::spec_gloss_to_metallic_roughness(
                    diffuse_factor,
                    specular_factor_sg,
                );

            base_color_array = [
                converted_base_rgb[0],
                converted_base_rgb[1],
                converted_base_rgb[2],
                diffuse_factor[3],
            ];
            metallic_value = converted_metallic;
            roughness_value = (1.0 - glossiness_factor).clamp(0.0, 1.0);

            let diffuse_texture_info = spec_gloss.diffuse_texture();
            let spec_gloss_texture_info = spec_gloss.specular_glossiness_texture();
            let diffuse_image = diffuse_texture_info.as_ref().and_then(|tex| {
                image_index_to_id
                    .get(&tex.texture().source()?.index())
                    .copied()
            });
            let spec_gloss_image = spec_gloss_texture_info.as_ref().and_then(|tex| {
                image_index_to_id
                    .get(&tex.texture().source()?.index())
                    .copied()
            });

            if diffuse_image.is_some() || spec_gloss_image.is_some() {
                let mat_index = material.index().unwrap_or(usize::MAX);
                let base_name = format!("{}::sg_converted_base_{}", namespace, mat_index);
                let mr_name = format!("{}::sg_converted_mr_{}", namespace, mat_index);

                let diffuse_sampler = diffuse_texture_info
                    .as_ref()
                    .map(|tex| sampler_settings_from_gltf(&tex.texture().sampler()))
                    .unwrap_or(SamplerSettings::DEFAULT);
                let spec_gloss_sampler = spec_gloss_texture_info
                    .as_ref()
                    .map(|tex| sampler_settings_from_gltf(&tex.texture().sampler()))
                    .unwrap_or(SamplerSettings::DEFAULT);

                texture_specs.insert(
                    base_name.clone(),
                    TextureSpec {
                        recipe: TextureRecipe::SpecGlossBase {
                            diffuse: diffuse_image,
                            diffuse_factor,
                            spec_gloss: spec_gloss_image,
                            specular_factor: specular_factor_sg,
                            glossiness_factor,
                        },
                        usage: TextureUsage::Color,
                        sampler: diffuse_sampler,
                    },
                );
                texture_specs.insert(
                    mr_name.clone(),
                    TextureSpec {
                        recipe: TextureRecipe::SpecGlossMr {
                            diffuse: diffuse_image,
                            diffuse_factor,
                            spec_gloss: spec_gloss_image,
                            specular_factor: specular_factor_sg,
                            glossiness_factor,
                        },
                        usage: TextureUsage::Linear,
                        sampler: spec_gloss_sampler,
                    },
                );

                base_texture = Some(base_name);
                base_texture_transform = diffuse_texture_info
                    .as_ref()
                    .map(texture_transform_from_info)
                    .unwrap_or(TextureTransform::IDENTITY);
                metallic_roughness_texture = Some(mr_name);
                metallic_roughness_texture_transform = spec_gloss_texture_info
                    .as_ref()
                    .map(texture_transform_from_info)
                    .unwrap_or(TextureTransform::IDENTITY);
            }
        }

        let diffuse_trans_ext = material.extension_value("KHR_materials_diffuse_transmission");
        let diffuse_transmission_factor = diffuse_trans_ext
            .and_then(|v| v.get("diffuseTransmissionFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let diffuse_transmission_color_factor = diffuse_trans_ext
            .and_then(|v| v.get("diffuseTransmissionColorFactor"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                [
                    arr.first().and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    arr.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    arr.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                ]
            })
            .unwrap_or([1.0, 1.0, 1.0]);
        let (diffuse_transmission_texture, diffuse_transmission_texture_transform) =
            diffuse_trans_ext
                .and_then(|v| v.get("diffuseTransmissionTexture"))
                .and_then(|info| {
                    resolve_extension_texture(
                        info,
                        doc,
                        TextureUsage::Linear,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    )
                })
                .map(|(name, transform)| (Some(name), transform))
                .unwrap_or((None, TextureTransform::IDENTITY));
        let (diffuse_transmission_color_texture_pre, diffuse_transmission_color_texture_transform) =
            diffuse_trans_ext
                .and_then(|v| v.get("diffuseTransmissionColorTexture"))
                .and_then(|info| {
                    resolve_extension_texture(
                        info,
                        doc,
                        TextureUsage::Color,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    )
                })
                .map(|(name, transform)| (Some(name), transform))
                .unwrap_or((None, TextureTransform::IDENTITY));
        let diffuse_transmission_color_texture = pack_textures_rgb_a(
            diffuse_transmission_color_texture_pre.as_deref(),
            diffuse_transmission_texture.as_deref(),
            "diffuse_transmission_packed",
            TextureUsage::Color,
            texture_specs,
        );
        let diffuse_transmission_texture: Option<String> = None;

        let dispersion = material
            .extension_value("KHR_materials_dispersion")
            .and_then(|v| v.get("dispersion"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        let anisotropy_ext = material.extension_value("KHR_materials_anisotropy");
        let anisotropy_strength = anisotropy_ext
            .and_then(|v| v.get("anisotropyStrength"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let anisotropy_rotation = anisotropy_ext
            .and_then(|v| v.get("anisotropyRotation"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let (anisotropy_texture, anisotropy_texture_transform) = anisotropy_ext
            .and_then(|v| v.get("anisotropyTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    doc,
                    TextureUsage::Linear,
                    image_hashes,
                    image_index_to_id,
                    texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));

        let iridescence_ext = material.extension_value("KHR_materials_iridescence");
        let iridescence_factor = iridescence_ext
            .and_then(|v| v.get("iridescenceFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let iridescence_ior = iridescence_ext
            .and_then(|v| v.get("iridescenceIor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.3) as f32;
        let iridescence_thickness_minimum = iridescence_ext
            .and_then(|v| v.get("iridescenceThicknessMinimum"))
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as f32;
        let iridescence_thickness_maximum = iridescence_ext
            .and_then(|v| v.get("iridescenceThicknessMaximum"))
            .and_then(|v| v.as_f64())
            .unwrap_or(400.0) as f32;
        let (iridescence_texture_pre, iridescence_texture_transform) = iridescence_ext
            .and_then(|v| v.get("iridescenceTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    doc,
                    TextureUsage::Linear,
                    image_hashes,
                    image_index_to_id,
                    texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let (iridescence_thickness_texture_pre, iridescence_thickness_texture_transform) =
            iridescence_ext
                .and_then(|v| v.get("iridescenceThicknessTexture"))
                .and_then(|info| {
                    resolve_extension_texture(
                        info,
                        doc,
                        TextureUsage::Linear,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    )
                })
                .map(|(name, transform)| (Some(name), transform))
                .unwrap_or((None, TextureTransform::IDENTITY));
        let iridescence_texture = pack_textures_rg(
            iridescence_texture_pre.as_deref(),
            iridescence_thickness_texture_pre.as_deref(),
            "iridescence_packed",
            TextureUsage::Linear,
            texture_specs,
        );
        let iridescence_thickness_texture: Option<String> = None;

        let sheen_ext = material.extension_value("KHR_materials_sheen");
        let sheen_color_factor = sheen_ext
            .and_then(|v| v.get("sheenColorFactor"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                [
                    arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                ]
            })
            .unwrap_or([0.0, 0.0, 0.0]);
        let sheen_roughness_factor = sheen_ext
            .and_then(|v| v.get("sheenRoughnessFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let (sheen_color_texture_pre, sheen_color_texture_transform) = sheen_ext
            .and_then(|v| v.get("sheenColorTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    doc,
                    TextureUsage::Color,
                    image_hashes,
                    image_index_to_id,
                    texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let (sheen_roughness_texture_pre, sheen_roughness_texture_transform) = sheen_ext
            .and_then(|v| v.get("sheenRoughnessTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    doc,
                    TextureUsage::Linear,
                    image_hashes,
                    image_index_to_id,
                    texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let sheen_color_texture = pack_textures_rgb_a(
            sheen_color_texture_pre.as_deref(),
            sheen_roughness_texture_pre.as_deref(),
            "sheen_packed",
            TextureUsage::Color,
            texture_specs,
        );
        let sheen_roughness_texture: Option<String> = None;

        let clearcoat_ext = material.extension_value("KHR_materials_clearcoat");
        let clearcoat_factor = clearcoat_ext
            .and_then(|v| v.get("clearcoatFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let clearcoat_roughness_factor = clearcoat_ext
            .and_then(|v| v.get("clearcoatRoughnessFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let (clearcoat_texture_pre, clearcoat_texture_transform) = clearcoat_ext
            .and_then(|v| v.get("clearcoatTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    doc,
                    TextureUsage::Linear,
                    image_hashes,
                    image_index_to_id,
                    texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let (clearcoat_roughness_texture_pre, clearcoat_roughness_texture_transform) =
            clearcoat_ext
                .and_then(|v| v.get("clearcoatRoughnessTexture"))
                .and_then(|info| {
                    resolve_extension_texture(
                        info,
                        doc,
                        TextureUsage::Linear,
                        image_hashes,
                        image_index_to_id,
                        texture_specs,
                    )
                })
                .map(|(name, transform)| (Some(name), transform))
                .unwrap_or((None, TextureTransform::IDENTITY));
        let (clearcoat_normal_texture, clearcoat_normal_texture_transform) = clearcoat_ext
            .and_then(|v| v.get("clearcoatNormalTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    doc,
                    TextureUsage::Linear,
                    image_hashes,
                    image_index_to_id,
                    texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let clearcoat_normal_scale = clearcoat_ext
            .and_then(|v| v.get("clearcoatNormalTexture"))
            .and_then(|info| info.get("scale"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        let clearcoat_texture = pack_textures_rg(
            clearcoat_texture_pre.as_deref(),
            clearcoat_roughness_texture_pre.as_deref(),
            "clearcoat_packed",
            TextureUsage::Linear,
            texture_specs,
        );
        let clearcoat_roughness_texture: Option<String> = clearcoat_texture.clone();

        let world_material = Material {
            base_color: base_color_array,
            emissive_factor: [emissive[0], emissive[1], emissive[2]],
            alpha_mode: match material.alpha_mode() {
                gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
                gltf::material::AlphaMode::Mask => AlphaMode::Mask,
                gltf::material::AlphaMode::Blend => AlphaMode::Blend,
            },
            alpha_cutoff: material.alpha_cutoff().unwrap_or(0.5),
            blend_opaque_alpha_threshold: 0.99,
            base_texture,
            base_texture_transform,
            emissive_texture,
            emissive_texture_transform,
            normal_texture,
            normal_texture_transform,
            normal_scale,
            normal_map_flip_y: false,
            normal_map_two_component: false,
            detail_base_texture: None,
            detail_normal_texture: None,
            detail_uv_scale: 4.0,
            height_texture: None,
            parallax_scale: 0.0,
            reflection_box_min: [0.0, 0.0, 0.0],
            reflection_box_max: [0.0, 0.0, 0.0],
            metallic_roughness_texture,
            metallic_roughness_texture_transform,
            occlusion_texture,
            occlusion_texture_transform,
            occlusion_strength,
            roughness: roughness_value,
            metallic: metallic_value,
            unlit: material.unlit(),
            double_sided: material.double_sided(),
            transmission_factor,
            transmission_texture,
            transmission_texture_transform,
            thickness,
            thickness_texture,
            thickness_texture_transform,
            attenuation_color,
            attenuation_distance,
            ior,
            specular_factor,
            specular_color_factor,
            specular_texture,
            specular_texture_transform,
            specular_color_texture,
            specular_color_texture_transform,
            emissive_strength,
            diffuse_transmission_factor,
            diffuse_transmission_texture,
            diffuse_transmission_texture_transform,
            diffuse_transmission_color_factor,
            diffuse_transmission_color_texture,
            diffuse_transmission_color_texture_transform,
            dispersion,
            anisotropy_strength,
            anisotropy_rotation,
            anisotropy_texture,
            anisotropy_texture_transform,
            iridescence_factor,
            iridescence_texture,
            iridescence_texture_transform,
            iridescence_ior,
            iridescence_thickness_minimum,
            iridescence_thickness_maximum,
            iridescence_thickness_texture,
            iridescence_thickness_texture_transform,
            sheen_color_factor,
            sheen_color_texture,
            sheen_color_texture_transform,
            sheen_roughness_factor,
            sheen_roughness_texture,
            sheen_roughness_texture_transform,
            clearcoat_factor,
            clearcoat_texture,
            clearcoat_texture_transform,
            clearcoat_roughness_factor,
            clearcoat_roughness_texture,
            clearcoat_roughness_texture_transform,
            clearcoat_normal_texture,
            clearcoat_normal_texture_transform,
            clearcoat_normal_scale,
            ..Default::default()
        };

        let deduplicated_index =
            if let Some(existing_index) = materials.iter().position(|m| m == &world_material) {
                if let Some(mat_index) = material.index() {
                    tracing::debug!(
                        "Material {} is identical to material {}, reusing",
                        mat_index,
                        existing_index
                    );
                }
                existing_index
            } else {
                let new_index = materials.len();
                materials.push(world_material);
                new_index
            };

        if let Some(mat_index) = material.index() {
            gltf_material_index_to_deduplicated_index.insert(mat_index, deduplicated_index);
        }
    }

    (materials, gltf_material_index_to_deduplicated_index)
}
