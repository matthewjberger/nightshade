use ash::vk::{self, Handle};
use nightshade_wgpu::backend::{DeviceConfig, negotiate_device_config};
use openxr as xr;
use std::ffi::{CString, c_char, c_void};

const VK_TARGET_VERSION: xr::Version = xr::Version::new(1, 1, 0);
const VK_TARGET_VERSION_ASH: u32 = vk::make_api_version(
    0,
    VK_TARGET_VERSION.major() as u32,
    VK_TARGET_VERSION.minor() as u32,
    VK_TARGET_VERSION.patch(),
);

pub struct XrGpu {
    pub depth_layer_supported: bool,
    pub local_floor_supported: bool,
    pub instance: nightshade_wgpu::wgpu::Instance,
    pub adapter: nightshade_wgpu::wgpu::Adapter,
    pub device: nightshade_wgpu::wgpu::Device,
    pub queue: nightshade_wgpu::wgpu::Queue,
    pub bindless: nightshade_wgpu::backend::BindlessConfig,
    pub session_create_info: xr::vulkan::SessionCreateInfo,
    _vk_entry: ash::Entry,
    _vk_instance: ash::Instance,
}

pub fn create_xr_gpu() -> Result<(xr::Instance, xr::SystemId, XrGpu), Box<dyn std::error::Error>> {
    let xr_entry = xr::Entry::linked();

    let available = xr_entry.enumerate_extensions()?;
    let mut required_extensions = xr::ExtensionSet::default();
    required_extensions.khr_vulkan_enable2 = true;
    required_extensions.khr_composition_layer_depth = available.khr_composition_layer_depth;
    required_extensions.ext_local_floor = available.ext_local_floor;
    if !available.khr_composition_layer_depth {
        tracing::info!(
            "runtime does not offer XR_KHR_composition_layer_depth: frames will reproject without              depth"
        );
    }

    let xr_instance = xr_entry.create_instance(
        &xr::ApplicationInfo {
            application_name: "nightshade",
            application_version: 1,
            engine_name: "nightshade",
            engine_version: 1,
            api_version: xr::Version::new(1, 0, 0),
        },
        &required_extensions,
        &[],
    )?;

    let system = xr_instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?;

    let requirements = xr_instance.graphics_requirements::<xr::Vulkan>(system)?;
    if VK_TARGET_VERSION < requirements.min_api_version_supported
        || VK_TARGET_VERSION.major() > requirements.max_api_version_supported.major()
    {
        return Err(format!(
            "the OpenXR runtime needs a Vulkan version between {} and {}.0.0, and this build \
             targets {VK_TARGET_VERSION}",
            requirements.min_api_version_supported,
            requirements.max_api_version_supported.major() + 1
        )
        .into());
    }

    let vk_entry = unsafe { ash::Entry::load()? };
    let flags = nightshade_wgpu::wgpu::InstanceFlags::default();

    let mut instance_extensions =
        <wgpu_hal::vulkan::Api as wgpu_hal::Api>::Instance::desired_extensions(
            &vk_entry,
            VK_TARGET_VERSION_ASH,
            flags,
        )?;

    for surface_extension in surface_instance_extensions() {
        if !instance_extensions.contains(&surface_extension) {
            instance_extensions.push(surface_extension);
        }
    }

    let vk_instance = unsafe {
        let extension_pointers: Vec<_> = instance_extensions
            .iter()
            .map(|name| name.as_ptr())
            .collect();
        let application_name = CString::new("nightshade")?;
        let application_info = vk::ApplicationInfo::default()
            .application_name(&application_name)
            .application_version(1)
            .engine_name(&application_name)
            .engine_version(1)
            .api_version(VK_TARGET_VERSION_ASH);

        let raw_instance = xr_instance
            .create_vulkan_instance(
                system,
                std::mem::transmute::<
                    unsafe extern "system" fn(
                        vk::Instance,
                        *const c_char,
                    )
                        -> Option<unsafe extern "system" fn()>,
                    unsafe extern "system" fn(
                        *const c_void,
                        *const c_char,
                    )
                        -> Option<unsafe extern "system" fn()>,
                >(vk_entry.static_fn().get_instance_proc_addr),
                &vk::InstanceCreateInfo::default()
                    .application_info(&application_info)
                    .enabled_extension_names(&extension_pointers) as *const _
                    as *const _,
            )?
            .map_err(vk::Result::from_raw)?;

        ash::Instance::load(
            vk_entry.static_fn(),
            vk::Instance::from_raw(raw_instance as _),
        )
    };

    let vk_physical_device = vk::PhysicalDevice::from_raw(unsafe {
        xr_instance.vulkan_graphics_device(system, vk_instance.handle().as_raw() as _)? as _
    });

    let device_properties =
        unsafe { vk_instance.get_physical_device_properties(vk_physical_device) };
    if device_properties.api_version < VK_TARGET_VERSION_ASH {
        return Err("the headset's GPU does not support Vulkan 1.1".into());
    }

    let hal_instance = unsafe {
        <wgpu_hal::vulkan::Api as wgpu_hal::Api>::Instance::from_raw(
            vk_entry.clone(),
            vk_instance.clone(),
            device_properties.api_version,
            0,
            None,
            instance_extensions,
            flags,
            nightshade_wgpu::wgpu::MemoryBudgetThresholds::default(),
            false,
            None,
        )?
    };

    let exposed_adapter = hal_instance
        .expose_adapter(vk_physical_device)
        .ok_or("wgpu could not use the GPU the OpenXR runtime selected")?;

    let DeviceConfig {
        required_features,
        required_limits,
        bindless,
    } = negotiate_device_config(
        exposed_adapter.features,
        &exposed_adapter.capabilities.limits,
        &exposed_adapter.info,
    );

    let mut device_extensions = exposed_adapter
        .adapter
        .required_device_extensions(required_features);
    if !device_extensions.contains(&ash::khr::swapchain::NAME) {
        device_extensions.push(ash::khr::swapchain::NAME);
    }

    let queue_family_index =
        unsafe { vk_instance.get_physical_device_queue_family_properties(vk_physical_device) }
            .iter()
            .position(|family| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .ok_or("the headset's GPU exposes no graphics queue family")? as u32;

    let (hal_device, vk_device_handle) = {
        let extension_pointers: Vec<_> =
            device_extensions.iter().map(|name| name.as_ptr()).collect();
        let mut physical_device_features = exposed_adapter
            .adapter
            .physical_device_features(&device_extensions, required_features);
        let queue_priorities = [1.0];
        let queue_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)];
        let create_info = physical_device_features
            .add_to_device_create(vk::DeviceCreateInfo::default().queue_create_infos(&queue_infos))
            .enabled_extension_names(&extension_pointers);

        let vk_device = unsafe {
            let raw_device = xr_instance
                .create_vulkan_device(
                    system,
                    std::mem::transmute::<
                        unsafe extern "system" fn(
                            vk::Instance,
                            *const c_char,
                        )
                            -> Option<unsafe extern "system" fn()>,
                        unsafe extern "system" fn(
                            *const c_void,
                            *const c_char,
                        )
                            -> Option<unsafe extern "system" fn()>,
                    >(vk_entry.static_fn().get_instance_proc_addr),
                    vk_physical_device.as_raw() as _,
                    &create_info as *const _ as *const _,
                )?
                .map_err(vk::Result::from_raw)?;

            ash::Device::load(vk_instance.fp_v1_0(), vk::Device::from_raw(raw_device as _))
        };

        let vk_device_handle = vk_device.handle().as_raw() as *const c_void;

        let hal_device = unsafe {
            exposed_adapter.adapter.device_from_raw(
                vk_device,
                None,
                &device_extensions,
                required_features,
                &required_limits,
                &nightshade_wgpu::wgpu::MemoryHints::Performance,
                queue_family_index,
                0,
            )
        }?;

        (hal_device, vk_device_handle)
    };

    let session_create_info = xr::vulkan::SessionCreateInfo {
        instance: vk_instance.handle().as_raw() as *const c_void,
        physical_device: vk_physical_device.as_raw() as *const c_void,
        device: vk_device_handle,
        queue_family_index,
        queue_index: 0,
    };

    let instance =
        unsafe { nightshade_wgpu::wgpu::Instance::from_hal::<wgpu_hal::api::Vulkan>(hal_instance) };
    let adapter = unsafe { instance.create_adapter_from_hal(exposed_adapter) };
    let (device, queue) = unsafe {
        adapter.create_device_from_hal(
            hal_device,
            &nightshade_wgpu::wgpu::DeviceDescriptor {
                label: Some("nightshade OpenXR device"),
                required_features,
                required_limits,
                memory_hints: nightshade_wgpu::wgpu::MemoryHints::Performance,
                experimental_features: nightshade_wgpu::wgpu::ExperimentalFeatures::disabled(),
                trace: nightshade_wgpu::wgpu::Trace::Off,
            },
        )
    }?;

    tracing::info!("adopted the OpenXR runtime's Vulkan device into wgpu");

    Ok((
        xr_instance,
        system,
        XrGpu {
            depth_layer_supported: available.khr_composition_layer_depth,
            local_floor_supported: available.ext_local_floor,
            instance,
            adapter,
            device,
            queue,
            bindless,
            session_create_info,
            _vk_entry: vk_entry,
            _vk_instance: vk_instance,
        },
    ))
}

fn surface_instance_extensions() -> Vec<&'static std::ffi::CStr> {
    let mut extensions = vec![ash::khr::surface::NAME];
    #[cfg(target_os = "windows")]
    extensions.push(ash::khr::win32_surface::NAME);
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        extensions.push(ash::khr::xlib_surface::NAME);
        extensions.push(ash::khr::xcb_surface::NAME);
        extensions.push(ash::khr::wayland_surface::NAME);
    }
    extensions
}
