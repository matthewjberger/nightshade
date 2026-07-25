//! The renderer's material table: one entry per registered material and
//! the registry that hands out material ids.

/// A resolved material for the renderer: the engine-side material data plus its
/// resolved texture ids. The renderer converts this to GPU material data with
/// its own bindless texture-layer map. Material id `N` maps to `entries[N - 1]`;
/// id 0 is the built-in default material.
#[derive(Clone, Default)]
pub struct RenderMaterialEntry {
    /// Engine-side material parameters.
    pub material: crate::material::Material,
    /// Resolved texture ids for the material's texture slots.
    pub texture_ids: crate::material::MaterialTextureIds,
}

/// The scene's materials resolved once per frame by the engine, so the render
/// passes read materials here instead of walking the material registry and each
/// entity's `MaterialRef` themselves.
#[derive(Clone, Default)]
pub struct RenderMaterials {
    /// Resolved materials, where id `N` is `entries[N - 1]`.
    pub entries: Vec<RenderMaterialEntry>,
    /// The set of live material ids (`entries` index + 1). Used to check whether
    /// a material an entity references is already known to the GPU-side cache.
    pub material_ids: std::collections::HashSet<u32>,
    /// Ids of materials with a blended or dithered alpha mode.
    pub transparent_ids: std::collections::HashSet<u32>,
    /// Ids of materials with an alpha-mask cutout mode.
    pub mask_ids: std::collections::HashSet<u32>,
    /// Ids of materials that render both faces.
    pub double_sided_ids: std::collections::HashSet<u32>,
    /// Bumped whenever the table changes, so consumers can gate their material
    /// conversions on a counter.
    pub generation: u64,
}

impl RenderMaterials {
    /// Adds a material and returns its id. Material id `N` is `entries[N - 1]`;
    /// id 0 is the renderer's built-in default. Updates the live / transparent /
    /// mask / double-sided id sets from the material and bumps `generation`.
    pub fn add(&mut self, material: crate::material::Material) -> u32 {
        self.add_textured(material, crate::material::MaterialTextureIds::default())
    }

    /// Like [`RenderMaterials::add`], with resolved texture ids.
    pub fn add_textured(
        &mut self,
        material: crate::material::Material,
        texture_ids: crate::material::MaterialTextureIds,
    ) -> u32 {
        use crate::material::AlphaMode;
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
