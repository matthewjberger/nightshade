use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, NagaModuleDescriptor, ShaderDefValue, ShaderLanguage,
};
use std::collections::HashMap;
use std::sync::Mutex;

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
        let composer = Composer::default().with_capabilities(
            wgpu::naga::valid::Capabilities::default()
                | wgpu::naga::valid::Capabilities::TEXTURE_AND_SAMPLER_BINDING_ARRAY
                | wgpu::naga::valid::Capabilities::TEXTURE_AND_SAMPLER_BINDING_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::naga::valid::Capabilities::SHADER_INT64
                | wgpu::naga::valid::Capabilities::TEXTURE_INT64_ATOMIC,
        );
        Self {
            inner: Mutex::new(composer),
        }
    }

    pub fn publish_module(&self, source: &str, file_path: &str) {
        let mut composer = self.inner.lock().unwrap();
        add_module(&mut composer, source, file_path);
    }

    pub fn compose(
        &self,
        label: &str,
        source: &str,
        shader_defs: &[(&str, ShaderDefValue)],
    ) -> Result<wgpu::naga::Module, String> {
        let defs: HashMap<String, ShaderDefValue> = shader_defs
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();

        let mut composer = self.inner.lock().unwrap();
        composer
            .make_naga_module(NagaModuleDescriptor {
                source,
                file_path: label,
                shader_type: naga_oil::compose::ShaderType::Wgsl,
                shader_defs: defs,
                additional_imports: &[],
            })
            .map_err(|error| error.emit_to_string(&composer))
    }

    pub fn compile_wgsl(
        &self,
        device: &wgpu::Device,
        label: &str,
        source: &str,
    ) -> wgpu::ShaderModule {
        self.compile_wgsl_with_defs(device, label, source, &[])
    }

    pub fn compile_wgsl_with_defs(
        &self,
        device: &wgpu::Device,
        label: &str,
        source: &str,
        shader_defs: &[(&str, ShaderDefValue)],
    ) -> wgpu::ShaderModule {
        let module = self
            .compose(label, source, shader_defs)
            .unwrap_or_else(|diagnostics| {
                panic!("failed to compose shader {label}: {diagnostics}")
            });

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
        })
    }
}

fn add_module(composer: &mut Composer, source: &str, file_path: &str) {
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source,
            file_path,
            language: ShaderLanguage::Wgsl,
            as_name: None,
            additional_imports: &[],
            shader_defs: HashMap::new(),
        })
        .unwrap_or_else(|error| panic!("the shader module {file_path} was not published: {error}"));
}
