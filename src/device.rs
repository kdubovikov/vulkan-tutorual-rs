use std::{mem::needs_drop, sync::Arc, usize};
use vulkano::{device::{Device, DeviceExtensions, Features, Queue, QueuesIter}, instance::{Instance, PhysicalDevice, QueueFamily}, swapchain::Surface};
use winit::window::Window;

/// Structure that holds all necessary queue IDs for future reference
pub struct QueueCollection {
    pub graphics_queue_id: Option<u32>,
    pub presentation_queue_id: Option<u32>,
}

impl QueueCollection {
    fn all_present(&self) -> bool {
        self.graphics_queue_id.is_some() && self.presentation_queue_id.is_some()
    }

    pub fn is_shared(&self) -> bool {
        if self.all_present() {
            self.graphics_queue_id.unwrap() == self.presentation_queue_id.unwrap()
        } else {
            false
        }
    }
}

fn device_extensions(physical_device: PhysicalDevice) -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        .. DeviceExtensions::required_extensions(physical_device)
    }
}

pub fn create_device(
    surface: &Arc<Surface<Window>>,
    instance: &Arc<Instance>,
) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
    let device = pick_physical_device(surface, instance);
    let queue_collection = find_queue_families(surface, &device);

    if !queue_collection.all_present() {
        panic!("No suitable queue collections was found");
    }

    // find queues we need among all available queues
    let queues: Vec<_> = device
        .queue_families()
        .enumerate()
        .filter(|(i, _)| {
            *i == queue_collection.graphics_queue_id.unwrap() as usize
                || *i == queue_collection.presentation_queue_id.unwrap() as usize
        })
        .map(|(_, v)| (v, 1.0))
        .collect();

    let (device, mut queues) = Device::new(
        device,
        &Features::none(),
        &device_extensions(device),
        queues,
    )
    .unwrap();

    let graphics_queue = queues.next().unwrap();
    let presentation_queue =  queues.next().unwrap_or(graphics_queue.clone());

    (device, graphics_queue, presentation_queue)
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

fn check_device_extension_support(device: &PhysicalDevice) -> bool {
    let available_ext = DeviceExtensions::supported_by_device(*device);
    let needed_ext = device_extensions(*device);

    available_ext.intersection(&needed_ext) == needed_ext
}
