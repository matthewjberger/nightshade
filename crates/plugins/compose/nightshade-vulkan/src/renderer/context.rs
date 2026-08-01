use ash::{ext::debug_utils, khr::swapchain, vk};
use gpu_allocator::vulkan::Allocation;
use std::sync::{Arc, Mutex};

pub const FRAMES_IN_FLIGHT: usize = 3;

pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
    pub transfer: Option<u32>,
    pub compute: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Compression {
    pub bc: bool,
    pub astc: bool,
}

pub struct SentOnce {
    pub done_at: u64,
    pub command_buffer: vk::CommandBuffer,
    pub buffers: Vec<vk::Buffer>,
    pub allocations: Vec<Allocation>,
}

pub struct SentWork {
    pub timeline: vk::Semaphore,
    pub sent: u64,
    pub waiting: Vec<SentOnce>,
}

pub struct Lane {
    pub family: u32,
    pub queue: vk::Queue,
    pub pool: vk::CommandPool,
    pub work: Mutex<SentWork>,
}

pub struct Context {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub surface: ash::khr::surface::Instance,
    pub surface_khr: vk::SurfaceKHR,
    pub debug_utils: debug_utils::Instance,
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub families: QueueFamilies,
    pub graphics: Lane,
    pub transfer: Option<Lane>,
    pub compute: Option<Lane>,
    pub present_queue: vk::Queue,
    pub allocator: Option<Arc<Mutex<gpu_allocator::vulkan::Allocator>>>,
    pub mesh_shader: ash::ext::mesh_shader::Device,
    pub acceleration: Option<ash::khr::acceleration_structure::Device>,
    pub hdr_metadata: Option<ash::ext::hdr_metadata::Device>,
    pub compression: Compression,
    pub pipeline_cache: vk::PipelineCache,
    lost: std::sync::atomic::AtomicBool,
    pipeline_cache_path: Option<std::path::PathBuf>,
}

impl Context {
    pub fn note(&self, outcome: vk::Result) {
        if outcome == vk::Result::ERROR_DEVICE_LOST {
            self.lost.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn is_lost(&self) -> bool {
        self.lost.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn mark_lost(&self) {
        self.lost.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn allocator(&self) -> &Arc<Mutex<gpu_allocator::vulkan::Allocator>> {
        self.allocator
            .as_ref()
            .expect("the allocator is present until the context is dropped")
    }

    pub fn upload_lane(&self) -> &Lane {
        self.transfer.as_ref().unwrap_or(&self.graphics)
    }

    pub fn lanes(&self) -> impl Iterator<Item = &Lane> {
        std::iter::once(&self.graphics)
            .chain(self.transfer.as_ref())
            .chain(self.compute.as_ref())
    }

    pub fn families_in_use(&self) -> Vec<u32> {
        let mut families = vec![self.graphics.family];
        families.extend(self.transfer.as_ref().map(|lane| lane.family));
        families.extend(self.compute.as_ref().map(|lane| lane.family));
        families.sort_unstable();
        families.dedup();
        families
    }
}

fn create_lane(device: &ash::Device, family: u32) -> Result<Lane, Box<dyn std::error::Error>> {
    let queue = unsafe { device.get_device_queue(family, 0) };
    let pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(family)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    let pool = unsafe { device.create_command_pool(&pool_info, None)? };
    let mut timeline_info = vk::SemaphoreTypeCreateInfo::default()
        .semaphore_type(vk::SemaphoreType::TIMELINE)
        .initial_value(0);
    let semaphore_info = vk::SemaphoreCreateInfo::default().push_next(&mut timeline_info);
    let timeline = unsafe { device.create_semaphore(&semaphore_info, None)? };
    Ok(Lane {
        family,
        queue,
        pool,
        work: Mutex::new(SentWork {
            timeline,
            sent: 0,
            waiting: Vec::new(),
        }),
    })
}

pub fn create_context<W>(window_handle: &W) -> Result<Context, Box<dyn std::error::Error>>
where
    W: raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle,
{
    let entry = unsafe { ash::Entry::load()? };
    let instance = create_instance(window_handle, &entry)?;
    let (surface, surface_khr) = create_surface(window_handle, &entry, &instance)?;
    let (debug_utils, debug_messenger) = create_debug_utils(&entry, &instance)?;
    let (physical_device, families) = find_physical_device(&instance, &surface, surface_khr)?;
    let traces = can_trace(&instance, physical_device);
    let declares_light =
        supports_extension(&instance, physical_device, ash::ext::hdr_metadata::NAME)?;
    let compression = compression_of(&instance, physical_device);
    report_device(&instance, physical_device, traces);
    let device = create_device(
        &instance,
        physical_device,
        &families,
        traces,
        declares_light,
        compression,
    )?;

    let graphics = create_lane(&device, families.graphics)?;
    let transfer = match families.transfer {
        Some(family) => Some(create_lane(&device, family)?),
        None => None,
    };
    let compute = match families.compute {
        Some(family) => Some(create_lane(&device, family)?),
        None => None,
    };
    let present_queue = unsafe { device.get_device_queue(families.present, 0) };

    let allocator_info = gpu_allocator::vulkan::AllocatorCreateDesc {
        instance: instance.clone(),
        device: device.clone(),
        physical_device,
        debug_settings: gpu_allocator::AllocatorDebugSettings::default(),
        buffer_device_address: true,
        allocation_sizes: gpu_allocator::AllocationSizes::default(),
    };
    let allocator = gpu_allocator::vulkan::Allocator::new(&allocator_info)?;

    let mesh_shader = ash::ext::mesh_shader::Device::new(&instance, &device);
    let acceleration =
        traces.then(|| ash::khr::acceleration_structure::Device::new(&instance, &device));
    let hdr_metadata =
        declares_light.then(|| ash::ext::hdr_metadata::Device::new(&instance, &device));
    let (pipeline_cache, pipeline_cache_path) =
        create_pipeline_cache(&instance, physical_device, &device)?;

    Ok(Context {
        entry,
        instance,
        surface,
        surface_khr,
        debug_utils,
        debug_messenger,
        physical_device,
        device,
        families,
        graphics,
        transfer,
        compute,
        present_queue,
        allocator: Some(Arc::new(Mutex::new(allocator))),
        mesh_shader,
        acceleration,
        hdr_metadata,
        compression,
        pipeline_cache,
        pipeline_cache_path,
        lost: std::sync::atomic::AtomicBool::new(false),
    })
}

fn compression_of(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Compression {
    let features = unsafe { instance.get_physical_device_features(physical_device) };
    Compression {
        bc: features.texture_compression_bc == vk::TRUE,
        astc: features.texture_compression_astc_ldr == vk::TRUE,
    }
}

const CACHE_HEADER_SIZE: usize = 16 + vk::UUID_SIZE;

fn create_pipeline_cache(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
) -> Result<(vk::PipelineCache, Option<std::path::PathBuf>), Box<dyn std::error::Error>> {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let identity: String = properties
        .pipeline_cache_uuid
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let path = cache_directory().map(|directory| {
        directory
            .join("nightshade")
            .join(format!("pipelines-{identity}.bin"))
    });
    let saved = path
        .as_ref()
        .and_then(|path| std::fs::read(path).ok())
        .filter(|bytes| cache_fits_device(bytes, &properties));
    let create_info =
        vk::PipelineCacheCreateInfo::default().initial_data(saved.as_deref().unwrap_or(&[]));
    let cache = unsafe { device.create_pipeline_cache(&create_info, None)? };
    Ok((cache, path))
}

fn cache_fits_device(bytes: &[u8], properties: &vk::PhysicalDeviceProperties) -> bool {
    if bytes.len() < CACHE_HEADER_SIZE {
        return false;
    }
    let word =
        |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    word(0) as usize >= CACHE_HEADER_SIZE
        && word(4) == vk::PipelineCacheHeaderVersion::ONE.as_raw() as u32
        && word(8) == properties.vendor_id
        && word(12) == properties.device_id
        && bytes[16..CACHE_HEADER_SIZE] == properties.pipeline_cache_uuid
}

fn cache_directory() -> Option<std::path::PathBuf> {
    if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| {
            std::path::PathBuf::from(home)
                .join("Library")
                .join("Caches")
        })
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".cache"))
            })
    }
}

fn save_pipeline_cache(context: &Context) {
    let Some(path) = &context.pipeline_cache_path else {
        return;
    };
    let data = match unsafe {
        context
            .device
            .get_pipeline_cache_data(context.pipeline_cache)
    } {
        Ok(data) => data,
        Err(error) => {
            tracing::error!("the pipeline cache was not read back: {error}");
            return;
        }
    };
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        tracing::error!("the pipeline cache directory was not made: {error}");
        return;
    }
    if let Err(error) = std::fs::write(path, data) {
        tracing::error!("the pipeline cache was not saved: {error}");
    }
}

fn create_instance(
    window_handle: &impl raw_window_handle::HasDisplayHandle,
    entry: &ash::Entry,
) -> Result<ash::Instance, Box<dyn std::error::Error>> {
    let name = c"nightshade";
    let application_info = vk::ApplicationInfo::default()
        .application_name(name)
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(name)
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::make_api_version(0, 1, 3, 0));

    let requested = [c"VK_LAYER_KHRONOS_validation"];
    let available = unsafe { entry.enumerate_instance_layer_properties()? };
    let layers: Vec<*const std::os::raw::c_char> = requested
        .iter()
        .filter(|wanted| {
            available.iter().any(|layer| {
                let name = unsafe { std::ffi::CStr::from_ptr(layer.layer_name.as_ptr()) };
                name == **wanted
            })
        })
        .map(|name| name.as_ptr())
        .collect();

    let mut extension_names =
        ash_window::enumerate_required_extensions(window_handle.display_handle()?.as_raw())?
            .to_vec();
    extension_names.push(ash::ext::debug_utils::NAME.as_ptr());
    let offered = unsafe { entry.enumerate_instance_extension_properties(None)? };
    let names_a_color_space = offered.iter().any(|extension| {
        let name = unsafe { std::ffi::CStr::from_ptr(extension.extension_name.as_ptr()) };
        name == ash::ext::swapchain_colorspace::NAME
    });
    if names_a_color_space {
        extension_names.push(ash::ext::swapchain_colorspace::NAME.as_ptr());
    }

    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extension_names);

    Ok(unsafe { entry.create_instance(&create_info, None) }?)
}

fn create_surface<W>(
    window_handle: &W,
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Result<(ash::khr::surface::Instance, vk::SurfaceKHR), Box<dyn std::error::Error>>
where
    W: raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle,
{
    let surface = ash::khr::surface::Instance::new(entry, instance);
    let surface_khr = unsafe {
        ash_window::create_surface(
            entry,
            instance,
            window_handle.display_handle()?.as_raw(),
            window_handle.window_handle()?.as_raw(),
            None,
        )?
    };
    Ok((surface, surface_khr))
}

fn create_debug_utils(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Result<(debug_utils::Instance, vk::DebugUtilsMessengerEXT), Box<dyn std::error::Error>> {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(debug_callback));
    let debug_utils = debug_utils::Instance::new(entry, instance);
    let messenger = unsafe { debug_utils.create_debug_utils_messenger(&create_info, None)? };
    Ok((debug_utils, messenger))
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    kind: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let message = unsafe { std::ffi::CStr::from_ptr((*data).p_message) };
    if severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        tracing::error!("{kind:?} {message:?}");
    } else {
        tracing::warn!("{kind:?} {message:?}");
    }
    vk::FALSE
}

fn find_physical_device(
    instance: &ash::Instance,
    surface: &ash::khr::surface::Instance,
    surface_khr: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, QueueFamilies), Box<dyn std::error::Error>> {
    let devices = unsafe { instance.enumerate_physical_devices()? };
    let discrete = devices.iter().find(|device| {
        let properties = unsafe { instance.get_physical_device_properties(**device) };
        properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
    });
    let integrated = devices.iter().find(|device| {
        let properties = unsafe { instance.get_physical_device_properties(**device) };
        properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
    });
    let Some(physical_device) = discrete.or(integrated).copied() else {
        return Err("no discrete or integrated GPU is available".into());
    };

    if !supports_extension(instance, physical_device, swapchain::NAME)? {
        return Err("the physical device does not support swapchains".into());
    }
    if !supports_extension(instance, physical_device, ash::ext::mesh_shader::NAME)? {
        return Err(
            "the physical device does not support VK_EXT_mesh_shader, which this backend needs"
                .into(),
        );
    }

    let mut mesh_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default();
    let mut features13 = vk::PhysicalDeviceVulkan13Features::default();
    let mut features12 = vk::PhysicalDeviceVulkan12Features::default();
    let mut features = vk::PhysicalDeviceFeatures2::default()
        .push_next(&mut features12)
        .push_next(&mut features13)
        .push_next(&mut mesh_features);
    unsafe { instance.get_physical_device_features2(physical_device, &mut features) };
    let sampler_anisotropy = features.features.sampler_anisotropy;

    if features13.dynamic_rendering == vk::FALSE {
        return Err("the physical device does not support dynamic rendering".into());
    }
    if features13.synchronization2 == vk::FALSE {
        return Err("the physical device does not support synchronization2".into());
    }
    if mesh_features.mesh_shader == vk::FALSE {
        return Err("the physical device does not support mesh shaders".into());
    }
    if mesh_features.task_shader == vk::FALSE {
        return Err("the physical device does not support task shaders".into());
    }
    if features12.runtime_descriptor_array == vk::FALSE
        || features12.descriptor_binding_partially_bound == vk::FALSE
        || features12.shader_sampled_image_array_non_uniform_indexing == vk::FALSE
    {
        return Err("the physical device does not support bindless sampled images".into());
    }
    if features12.buffer_device_address == vk::FALSE {
        return Err("the physical device does not support buffer device addresses".into());
    }
    if sampler_anisotropy == vk::FALSE {
        return Err("the physical device does not support anisotropic sampling".into());
    }
    if features12.scalar_block_layout == vk::FALSE {
        return Err("the physical device does not support scalar block layout".into());
    }
    if features12.draw_indirect_count == vk::FALSE {
        return Err("the physical device does not support counted indirect draws".into());
    }

    let families = find_queue_families(instance, surface, surface_khr, physical_device)?;
    Ok((physical_device, families))
}

fn supports_extension(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    wanted: &std::ffi::CStr,
) -> Result<bool, Box<dyn std::error::Error>> {
    let properties = unsafe { instance.enumerate_device_extension_properties(physical_device)? };
    Ok(properties.iter().any(|extension| {
        let name = unsafe { std::ffi::CStr::from_ptr(extension.extension_name.as_ptr()) };
        name == wanted
    }))
}

fn find_queue_families(
    instance: &ash::Instance,
    surface: &ash::khr::surface::Instance,
    surface_khr: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<QueueFamilies, Box<dyn std::error::Error>> {
    let properties =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let mut graphics = None;
    let mut present = None;
    let mut transfer = None;
    let mut compute = None;
    for (index, family) in properties
        .iter()
        .enumerate()
        .filter(|(_, family)| family.queue_count > 0)
    {
        let index = index as u32;
        let capable = family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            && family.queue_flags.contains(vk::QueueFlags::COMPUTE);
        if graphics.is_none() && capable {
            graphics = Some(index);
        }
        let presentable = unsafe {
            surface.get_physical_device_surface_support(physical_device, index, surface_khr)?
        };
        if present.is_none() && presentable {
            present = Some(index);
        }
        let moves_only = family.queue_flags.contains(vk::QueueFlags::TRANSFER)
            && !family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            && !family.queue_flags.contains(vk::QueueFlags::COMPUTE);
        let computes_only = family.queue_flags.contains(vk::QueueFlags::COMPUTE)
            && !family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
        if computes_only {
            let carries = family.queue_flags.as_raw().count_ones();
            let better = compute
                .map(|(_, held): (u32, u32)| carries < held)
                .unwrap_or(true);
            if better {
                compute = Some((index, carries));
            }
        }
        if moves_only {
            let carries = family.queue_flags.as_raw().count_ones();
            let better = transfer
                .map(|(_, held): (u32, u32)| carries < held)
                .unwrap_or(true);
            if better {
                transfer = Some((index, carries));
            }
        }
    }
    Ok(QueueFamilies {
        graphics: graphics.ok_or("no graphics and compute queue family was found")?,
        present: present.ok_or("no present queue family was found")?,
        transfer: transfer.map(|(index, _)| index),
        compute: compute.map(|(index, _)| index),
    })
}

fn report_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, traces: bool) {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let name = unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) };
    let name = name.to_string_lossy();
    if traces {
        tracing::info!("drawing on {name}, which can trace a ray");
    } else {
        tracing::info!("drawing on {name}, which cannot trace a ray");
    }
}

fn can_trace(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> bool {
    let Ok(available) =
        (unsafe { instance.enumerate_device_extension_properties(physical_device) })
    else {
        return false;
    };
    let wanted = [
        ash::khr::acceleration_structure::NAME,
        ash::khr::ray_query::NAME,
        ash::khr::deferred_host_operations::NAME,
    ];
    wanted.iter().all(|name| {
        available.iter().any(|extension| {
            let held = unsafe { std::ffi::CStr::from_ptr(extension.extension_name.as_ptr()) };
            held == *name
        })
    })
}

fn create_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    families: &QueueFamilies,
    traces: bool,
    declares_light: bool,
    compression: Compression,
) -> Result<ash::Device, Box<dyn std::error::Error>> {
    let mut indices = vec![families.graphics, families.present];
    indices.extend(families.transfer);
    indices.extend(families.compute);
    indices.sort_unstable();
    indices.dedup();
    let priorities = [1.0f32];
    let queue_infos = indices
        .iter()
        .map(|index| {
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(*index)
                .queue_priorities(&priorities)
        })
        .collect::<Vec<_>>();

    let mut extensions = vec![
        swapchain::NAME.as_ptr(),
        ash::ext::mesh_shader::NAME.as_ptr(),
    ];
    if traces {
        extensions.push(ash::khr::acceleration_structure::NAME.as_ptr());
        extensions.push(ash::khr::ray_query::NAME.as_ptr());
        extensions.push(ash::khr::deferred_host_operations::NAME.as_ptr());
    }
    if declares_light {
        extensions.push(ash::ext::hdr_metadata::NAME.as_ptr());
    }

    let mut mesh_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default()
        .mesh_shader(true)
        .task_shader(true);
    let mut features13 = vk::PhysicalDeviceVulkan13Features::default()
        .dynamic_rendering(true)
        .synchronization2(true)
        .maintenance4(true);
    let mut features12 = vk::PhysicalDeviceVulkan12Features::default()
        .descriptor_indexing(true)
        .runtime_descriptor_array(true)
        .descriptor_binding_partially_bound(true)
        .descriptor_binding_sampled_image_update_after_bind(true)
        .descriptor_binding_update_unused_while_pending(true)
        .shader_sampled_image_array_non_uniform_indexing(true)
        .buffer_device_address(true)
        .timeline_semaphore(true)
        .scalar_block_layout(true)
        .draw_indirect_count(true);
    let core_features = vk::PhysicalDeviceFeatures::default()
        .sampler_anisotropy(true)
        .independent_blend(true)
        .texture_compression_bc(compression.bc)
        .texture_compression_astc_ldr(compression.astc);
    let mut features = vk::PhysicalDeviceFeatures2::default()
        .features(core_features)
        .push_next(&mut features12)
        .push_next(&mut features13)
        .push_next(&mut mesh_features);

    let mut structure_features =
        vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default().acceleration_structure(true);
    let mut query_features = vk::PhysicalDeviceRayQueryFeaturesKHR::default().ray_query(true);
    let mut create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&extensions)
        .push_next(&mut features);
    if traces {
        create_info = create_info
            .push_next(&mut structure_features)
            .push_next(&mut query_features);
    }

    Ok(unsafe { instance.create_device(physical_device, &create_info, None)? })
}

fn release_sent_work(context: &Context) {
    for lane in context.lanes() {
        let Ok(mut work) = lane.work.lock() else {
            continue;
        };
        let waiting = std::mem::take(&mut work.waiting);
        unsafe { context.device.destroy_semaphore(work.timeline, None) };
        let Ok(mut allocator) = context.allocator().lock() else {
            continue;
        };
        for entry in waiting {
            unsafe {
                context
                    .device
                    .free_command_buffers(lane.pool, std::slice::from_ref(&entry.command_buffer));
                for buffer in entry.buffers {
                    context.device.destroy_buffer(buffer, None);
                }
            }
            for allocation in entry.allocations {
                if let Err(error) = allocator.free(allocation) {
                    tracing::error!("the memory was not given back: {error}");
                }
            }
        }
    }
}

pub fn destroy_context(context: &mut Context) {
    unsafe {
        if let Err(error) = context.device.device_wait_idle() {
            tracing::error!("the device did not become idle before teardown: {error}");
        }
        release_sent_work(context);
        save_pipeline_cache(context);
        context
            .device
            .destroy_pipeline_cache(context.pipeline_cache, None);
        context.allocator = None;
        context
            .device
            .destroy_command_pool(context.graphics.pool, None);
        if let Some(transfer) = &context.transfer {
            context.device.destroy_command_pool(transfer.pool, None);
        }
        if let Some(compute) = &context.compute {
            context.device.destroy_command_pool(compute.pool, None);
        }
        context.device.destroy_device(None);
        context.surface.destroy_surface(context.surface_khr, None);
        context
            .debug_utils
            .destroy_debug_utils_messenger(context.debug_messenger, None);
        context.instance.destroy_instance(None);
    }
}
