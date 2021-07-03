use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use std::sync::Arc;

pub fn pick_physical_device(instance: &Arc<Instance>) -> usize { 
   PhysicalDevice::enumerate(&instance)
      .position(|device| {
         find_queue_families(&device).is_some()
      })
      .expect("Could not find suitable physical device")
}

fn find_queue_families<'a>(device: &'a PhysicalDevice) -> Option<QueueFamily<'a>> {
   device.queue_families()
      .find(|queue_family| queue_family.supports_graphics())
}