#[derive(Clone, Default)]
pub struct RenderMaterialEntry {
    pub material: crate::data::material::Material,
    pub texture_ids: crate::data::material::MaterialTextureIds,
}

#[derive(Clone, Default)]
pub struct RenderMaterials {
    pub entries: Vec<RenderMaterialEntry>,
    pub material_ids: std::collections::HashSet<u32>,
    pub transparent_ids: std::collections::HashSet<u32>,
    pub mask_ids: std::collections::HashSet<u32>,
    pub double_sided_ids: std::collections::HashSet<u32>,
    pub transmissive_ids: std::collections::HashSet<u32>,
    pub generation: u64,
    pub class_generation: u64,
}

impl RenderMaterials {
    pub fn add(&mut self, material: crate::data::material::Material) -> u32 {
        self.add_textured(
            material,
            crate::data::material::MaterialTextureIds::default(),
        )
    }

    pub fn add_textured(
        &mut self,
        material: crate::data::material::Material,
        texture_ids: crate::data::material::MaterialTextureIds,
    ) -> u32 {
        use crate::data::material::AlphaMode;
        let transparent = matches!(material.alpha_mode, AlphaMode::Blend | AlphaMode::Dither);
        let mask = matches!(material.alpha_mode, AlphaMode::Mask);
        let double_sided = material.double_sided;
        self.entries.push(RenderMaterialEntry {
            material,
            texture_ids,
        });
        let id = self.entries.len() as u32;
        self.material_ids.insert(id);
        if transparent {
            self.transparent_ids.insert(id);
        }
        if mask {
            self.mask_ids.insert(id);
        }
        if double_sided {
            self.double_sided_ids.insert(id);
        }
        self.generation += 1;
        id
    }
}
