use std::{sync::Arc, usize};
use vulkano::{
    device::{Device, DeviceExtensions, Features, QueuesIter},
    instance::{Instance, PhysicalDevice, QueueFamily},
    swapchain::Surface,
};
use winit::window::Window;

struct QueueCollection {
    graphics_queue_id: Option<u32>,
    presentation_queue_id: Option<u32>,
}

impl QueueCollection {
    fn all_present(&self) -> bool {
        self.graphics_queue_id.is_some() && self.presentation_queue_id.is_some()
    }
}

pub fn create_device(
    surface: &Arc<Surface<Window>>,
    instance: &Arc<Instance>,
) -> (Arc<Device>, QueuesIter) {
    let device = pick_physical_device(surface, instance);
    let queue_collection = find_queue_families(surface, &device);

    let queues: Vec<_> = device
        .queue_families()
        .enumerate()
        .filter(|(i, _)| {
            *i == queue_collection.graphics_queue_id.unwrap() as usize
                || *i == queue_collection.presentation_queue_id.unwrap() as usize
        })
        .map(|(_, v)| (v, 1.0))
        .collect();

    Device::new(device, &Features::none(), &DeviceExtensions::required_extensions(device), queues).unwrap()
}

fn pick_physical_device<'a>(
    surface: &'a Arc<Surface<Window>>,
    instance: &'a Arc<Instance>,
) -> PhysicalDevice<'a> {
    PhysicalDevice::enumerate(&instance)
        .find(|device| find_queue_families(surface, &device).all_present())
        .expect("Could not find suitable physical device")
}

fn find_queue_families(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> QueueCollection {
    let mut collection = QueueCollection {
        graphics_queue_id: None,
        presentation_queue_id: None,
    };

    for queue_family in device.queue_families().into_iter() {
        if queue_family.supports_graphics() {
            collection.graphics_queue_id = Some(queue_family.id());
        }

        if surface.is_supported(queue_family).unwrap() {
            collection.presentation_queue_id = Some(queue_family.id());
        }
    }

    collection
}
