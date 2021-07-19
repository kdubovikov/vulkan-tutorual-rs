use std::{sync::Arc, usize};

use vulkano::{device::{Device, Queue}, format::Format, image::{ImageUsage, SwapchainImage}, instance::{Instance, PhysicalDevice}, swapchain::{Capabilities, ColorSpace, PresentMode, SupportedPresentModes, Surface, Swapchain, SwapchainBuilder}, sync::SharingMode};
use winit::window::Window;

fn choose_swap_surface_format(available_formats: &[(Format, ColorSpace)]) -> (Format, ColorSpace) {
    *available_formats
        .iter()
        .find(|(format, color_space)| {
            *format == Format::B8G8R8A8Unorm && *color_space == ColorSpace::SrgbNonLinear
        })
        .unwrap_or_else(|| &available_formats[0])
}

fn choose_swap_present_mode(available_present_modes: SupportedPresentModes) -> PresentMode {
    if available_present_modes.mailbox {
        PresentMode::Mailbox
    } else if available_present_modes.immediate {
        PresentMode::Immediate
    } else {
        PresentMode::Fifo
    }
}

fn choose_swap_extent(
    capabilities: &Capabilities,
    desired_width: u32,
    desired_height: u32,
) -> [u32; 2] {
    if let Some(current_extent) = capabilities.current_extent {
        current_extent
    } else {
        let mut actual_extent = [desired_width, desired_height];
        actual_extent[0] = capabilities.min_image_extent[0]
            .max(capabilities.max_image_extent[0].min(actual_extent[0]));
        actual_extent[1] = capabilities.min_image_extent[1]
            .max(capabilities.max_image_extent[1].min(actual_extent[1]));

        actual_extent
    }
}

pub fn create_swap_chain(
    instance: &Arc<Instance>,
    surface: &Arc<Surface<Window>>,
    physical_device_index: usize,
    device: &Arc<Device>,
    graphics_queue: &Arc<Queue>,
    presentation_queue: &Arc<Queue>,
    old_swap_chain: Option<&Arc<Swapchain<Window>>>
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
    let mut builder: Option<SwapchainBuilder<Window>> = None;

    if let Some(swap_chain) = old_swap_chain {
        builder = Some(swap_chain.recreate()); // new feature in vulkako 0.24, breaks lesson 16
    } else {
        let physical_device = PhysicalDevice::from_index(instance, physical_device_index).unwrap();
        let capabilities = surface
            .capabilities(physical_device)
            .expect("failed to get surface capabilities");

        let (surface_format, color_space) = choose_swap_surface_format(&capabilities.supported_formats);
        let present_mode = choose_swap_present_mode(capabilities.present_modes);
        let extent = choose_swap_extent(&capabilities, 1024, 768);

        let mut image_count = capabilities.min_image_count + 1;

        if let Some(max_image_count) = capabilities.max_image_count {
            if image_count > max_image_count {
                image_count = max_image_count;
            }
        }

        let image_usage = ImageUsage {
            color_attachment: true,
            ..ImageUsage::none()
        };

        let sharing: SharingMode =
            if graphics_queue.id_within_family() == presentation_queue.id_within_family() {
                graphics_queue.into()
            } else {
                vec![graphics_queue, presentation_queue].as_slice().into()
            };

       builder = Some(Swapchain::start(device.clone(), surface.clone())
            .num_images(image_count)
            .sharing_mode(sharing)
            .usage(image_usage)
            .dimensions(extent)
            .present_mode(present_mode)
            .format(surface_format)
            .color_space(color_space)
            .layers(1)
            .transform(capabilities.current_transform)
            .clipped(true));

    }

    builder
        .expect("Failed to create swap chain builder")
        .build()
        .expect("Failed to build swap chain")
}
