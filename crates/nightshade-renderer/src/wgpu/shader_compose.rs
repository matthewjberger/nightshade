use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, NagaModuleDescriptor, ShaderDefValue, ShaderLanguage,
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub struct ShaderComposer {
    inner: Mutex<Composer>,
}

impl Default for ShaderComposer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderComposer {
    pub fn new() -> Self {
        // These say what the composer will validate, not what any device can do.
        // A capability granted here only permits a construct to be written; the
        // device's own features still decide at module creation, and a shader
        // that needs one is only ever compiled where those features exist. So
        // this list is deliberately the union of everything any path may use.
        let mut composer = Composer::default().with_capabilities(
            wgpu::naga::valid::Capabilities::default()
                | wgpu::naga::valid::Capabilities::TEXTURE_AND_SAMPLER_BINDING_ARRAY
                | wgpu::naga::valid::Capabilities::TEXTURE_AND_SAMPLER_BINDING_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::naga::valid::Capabilities::SHADER_INT64
                | wgpu::naga::valid::Capabilities::TEXTURE_INT64_ATOMIC,
        );
        composer
            .add_composable_module(ComposableModuleDescriptor {
                source: include_str!("shaders/pbr_brdf.wgsl"),
                file_path: "pbr_brdf.wgsl",
                language: ShaderLanguage::Wgsl,
                as_name: None,
                additional_imports: &[],
                shader_defs: HashMap::new(),
            })
            .expect("failed to register pbr_brdf shader module");
        composer
            .add_composable_module(ComposableModuleDescriptor {
                source: include_str!("shaders/material_sampling.wgsl"),
                file_path: "material_sampling.wgsl",
                language: ShaderLanguage::Wgsl,
                as_name: None,
                additional_imports: &[],
                shader_defs: HashMap::new(),
            })
            .expect("failed to register material_sampling shader module");
        composer
            .add_composable_module(ComposableModuleDescriptor {
                source: include_str!("shaders/cull_common.wgsl"),
                file_path: "cull_common.wgsl",
                language: ShaderLanguage::Wgsl,
                as_name: None,
                additional_imports: &[],
                shader_defs: HashMap::new(),
            })
            .expect("failed to register cull_common shader module");
        composer
            .add_composable_module(ComposableModuleDescriptor {
                source: include_str!("shaders/material_data.wgsl"),
                file_path: "material_data.wgsl",
                language: ShaderLanguage::Wgsl,
                as_name: None,
                additional_imports: &[],
                shader_defs: HashMap::new(),
            })
            .expect("failed to register material_data shader module");
        composer
            .add_composable_module(ComposableModuleDescriptor {
                source: include_str!("shaders/area_lighting.wgsl"),
                file_path: "area_lighting.wgsl",
                language: ShaderLanguage::Wgsl,
                as_name: None,
                additional_imports: &[],
                shader_defs: HashMap::new(),
            })
            .expect("failed to register area_lighting shader module");
        #[cfg(feature = "meshlet")]
        {
            composer
                .add_composable_module(ComposableModuleDescriptor {
                    source: include_str!("shaders/meshlet_data.wgsl"),
                    file_path: "meshlet_data.wgsl",
                    language: ShaderLanguage::Wgsl,
                    as_name: None,
                    additional_imports: &[],
                    shader_defs: crate::wgpu::passes::geometry::meshlet::meshlet_shader_defs()
                        .into_iter()
                        .map(|(name, value)| (name.to_string(), value))
                        .collect(),
                })
                .expect("failed to register meshlet_data shader module");
            composer
                .add_composable_module(ComposableModuleDescriptor {
                    source: include_str!("shaders/meshlet_streams.wgsl"),
                    file_path: "meshlet_streams.wgsl",
                    language: ShaderLanguage::Wgsl,
                    as_name: None,
                    additional_imports: &[],
                    shader_defs: HashMap::new(),
                })
                .expect("failed to register meshlet_streams shader module");
        }
        Self {
            inner: Mutex::new(composer),
        }
    }

    pub fn compile_wgsl(
        &self,
        device: &wgpu::Device,
        label: &str,
        source: &str,
        shader_defs: &[(&str, ShaderDefValue)],
    ) -> wgpu::ShaderModule {
        let defs: HashMap<String, ShaderDefValue> = shader_defs
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();

        let mut composer = self.inner.lock().unwrap();
        let module = composer
            .make_naga_module(NagaModuleDescriptor {
                source,
                file_path: label,
                shader_type: naga_oil::compose::ShaderType::Wgsl,
                shader_defs: defs,
                additional_imports: &[],
            })
            .unwrap_or_else(|error| {
                panic!(
                    "failed to compose shader {label}: {}",
                    error.emit_to_string(&composer)
                )
            });

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
        })
    }
}

static GLOBAL_COMPOSER: OnceLock<ShaderComposer> = OnceLock::new();

pub fn global() -> &'static ShaderComposer {
    GLOBAL_COMPOSER.get_or_init(ShaderComposer::new)
}

pub fn compile_wgsl(device: &wgpu::Device, label: &str, source: &str) -> wgpu::ShaderModule {
    global().compile_wgsl(device, label, source, &[])
}

pub fn compile_wgsl_with_defs(
    device: &wgpu::Device,
    label: &str,
    source: &str,
    shader_defs: &[(&str, ShaderDefValue)],
) -> wgpu::ShaderModule {
    global().compile_wgsl(device, label, source, shader_defs)
}
