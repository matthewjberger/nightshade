use crate::renderer::context::Context;
use crate::renderer::memory::{Image, create_color_image, create_depth_image, destroy_image};
use ash::{khr::swapchain, vk};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encoding {
    Srgb,
    ScRgb,
    Pq,
}

impl Encoding {
    fn of(color_space: vk::ColorSpaceKHR) -> Self {
        match color_space {
            vk::ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT => Encoding::ScRgb,
            vk::ColorSpaceKHR::HDR10_ST2084_EXT => Encoding::Pq,
            _ => Encoding::Srgb,
        }
    }
}

fn choose_format(formats: &[vk::SurfaceFormatKHR], hdr: bool) -> Option<vk::SurfaceFormatKHR> {
    let wanted: &[(vk::Format, vk::ColorSpaceKHR)] = if hdr {
        &[
            (
                vk::Format::R16G16B16A16_SFLOAT,
                vk::ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT,
            ),
            (
                vk::Format::A2B10G10R10_UNORM_PACK32,
                vk::ColorSpaceKHR::HDR10_ST2084_EXT,
            ),
            (
                vk::Format::B8G8R8A8_UNORM,
                vk::ColorSpaceKHR::SRGB_NONLINEAR,
            ),
        ]
    } else {
        &[(
            vk::Format::B8G8R8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        )]
    };
    wanted
        .iter()
        .find_map(|(format, color_space)| {
            formats
                .iter()
                .find(|candidate| {
                    candidate.format == *format && candidate.color_space == *color_space
                })
                .copied()
        })
        .or_else(|| formats.first().copied())
}

pub struct Swapchain {
    pub loader: swapchain::Device,
    pub handle: vk::SwapchainKHR,
    pub extent: vk::Extent2D,
    pub render_extent: vk::Extent2D,
    pub format: vk::SurfaceFormatKHR,
    pub encoding: Encoding,
    pub images: Vec<vk::Image>,
    pub views: Vec<vk::ImageView>,
    pub depth: Image,
    pub hdr: Image,
    pub normal: Image,
    pub motion: Image,
    pub entity_id: Image,
    pub accumulated: Image,
    pub revealed: Image,
}

pub const HDR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

pub fn declare_display_light(
    context: &Context,
    swapchain: &Swapchain,
    display: Option<crate::renderer::display::DisplayLight>,
) {
    let (Some(metadata), Some(display)) = (context.hdr_metadata.as_ref(), display) else {
        return;
    };
    let (red, green, blue) = match swapchain.encoding {
        Encoding::Srgb => return,
        Encoding::ScRgb => ((0.64, 0.33), (0.30, 0.60), (0.15, 0.06)),
        Encoding::Pq => ((0.708, 0.292), (0.170, 0.797), (0.131, 0.046)),
    };
    let declared = vk::HdrMetadataEXT::default()
        .display_primary_red(vk::XYColorEXT { x: red.0, y: red.1 })
        .display_primary_green(vk::XYColorEXT {
            x: green.0,
            y: green.1,
        })
        .display_primary_blue(vk::XYColorEXT {
            x: blue.0,
            y: blue.1,
        })
        .white_point(vk::XYColorEXT {
            x: 0.3127,
            y: 0.3290,
        })
        .max_luminance(display.peak_nits.max(1.0))
        .min_luminance(display.floor_nits.max(0.0))
        .max_content_light_level(display.peak_nits.max(1.0))
        .max_frame_average_light_level(display.frame_nits.max(1.0));
    unsafe { metadata.set_hdr_metadata(&[swapchain.handle], &[declared]) };
}
pub const NORMAL_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
pub const MOTION_FORMAT: vk::Format = vk::Format::R16G16_SFLOAT;
pub const ENTITY_ID_FORMAT: vk::Format = vk::Format::R32_UINT;
pub const ACCUMULATED_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
pub const REVEALED_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

pub fn create_swapchain(
    context: &Context,
    width: u32,
    height: u32,
    vsync: bool,
    scale: f32,
    hdr: bool,
) -> Result<Swapchain, Box<dyn std::error::Error>> {
    let formats = unsafe {
        context
            .surface
            .get_physical_device_surface_formats(context.physical_device, context.surface_khr)?
    };
    let format = choose_format(&formats, hdr).ok_or("the surface reports no formats")?;
    let encoding = Encoding::of(format.color_space);
    if hdr && encoding == Encoding::Srgb {
        tracing::info!(
            "the display offers no high dynamic range format, so the frame is shown as usual"
        );
    }

    let present_modes = unsafe {
        context.surface.get_physical_device_surface_present_modes(
            context.physical_device,
            context.surface_khr,
        )?
    };
    let present_mode = if vsync {
        vk::PresentModeKHR::FIFO
    } else if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
        vk::PresentModeKHR::IMMEDIATE
    } else {
        vk::PresentModeKHR::FIFO
    };

    let capabilities = unsafe {
        context.surface.get_physical_device_surface_capabilities(
            context.physical_device,
            context.surface_khr,
        )?
    };

    let extent = if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D {
            width: width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    };

    let mut image_count = capabilities.min_image_count + 1;
    if capabilities.max_image_count > 0 {
        image_count = image_count.min(capabilities.max_image_count);
    }

    let families = [context.families.graphics, context.families.present];
    let mut create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(context.surface_khr)
        .min_image_count(image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);
    create_info = if context.families.graphics != context.families.present {
        create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&families)
    } else {
        create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    };
    let create_info = create_info
        .pre_transform(capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let loader = swapchain::Device::new(&context.instance, &context.device);
    let handle = unsafe { loader.create_swapchain(&create_info, None)? };
    let images = unsafe { loader.get_swapchain_images(handle)? };
    let views = images
        .iter()
        .map(|image| {
            let view_info = vk::ImageViewCreateInfo::default()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            unsafe { context.device.create_image_view(&view_info, None) }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let scale = scale.clamp(0.25, 4.0);
    let render_extent = vk::Extent2D {
        width: ((extent.width as f32 * scale).round() as u32).max(1),
        height: ((extent.height as f32 * scale).round() as u32).max(1),
    };
    let depth = create_depth_image(context, render_extent)?;
    let hdr = create_color_image(context, render_extent, HDR_FORMAT, "scene color")?;
    let normal = create_color_image(context, render_extent, NORMAL_FORMAT, "scene normal")?;
    let motion = create_color_image(context, render_extent, MOTION_FORMAT, "scene motion")?;
    let entity_id =
        create_color_image(context, render_extent, ENTITY_ID_FORMAT, "scene entity id")?;
    let accumulated =
        create_color_image(context, render_extent, ACCUMULATED_FORMAT, "blended sum")?;
    let revealed = create_color_image(context, render_extent, REVEALED_FORMAT, "blended showing")?;

    Ok(Swapchain {
        loader,
        handle,
        extent,
        render_extent,
        format,
        encoding,
        images,
        views,
        depth,
        hdr,
        normal,
        motion,
        entity_id,
        accumulated,
        revealed,
    })
}

pub fn destroy_swapchain(context: &Context, swapchain: &mut Swapchain) {
    destroy_image(context, &mut swapchain.depth);
    destroy_image(context, &mut swapchain.hdr);
    destroy_image(context, &mut swapchain.normal);
    destroy_image(context, &mut swapchain.motion);
    destroy_image(context, &mut swapchain.entity_id);
    destroy_image(context, &mut swapchain.accumulated);
    destroy_image(context, &mut swapchain.revealed);
    unsafe {
        for view in swapchain.views.drain(..) {
            context.device.destroy_image_view(view, None);
        }
        swapchain.loader.destroy_swapchain(swapchain.handle, None);
    }
}
