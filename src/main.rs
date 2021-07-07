mod device;
mod swapchain;

use device::create_device;
use log::info;
use swapchain::create_swap_chain;
use std::{iter::Inspect, sync::Arc};
use vulkano::{app_info_from_cargo_toml, device::{Device, Queue, QueuesIter}, image::SwapchainImage, instance::{
        debug::{DebugCallback, MessageSeverity, MessageType},
        layers_list, ApplicationInfo, Instance, InstanceExtensions, Version,
    }, swapchain::{Surface, Swapchain}};
use vulkano_win::{required_extensions, VkSurfaceBuild};
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, platform::run_return::EventLoopExtRunReturn, window::{Window, WindowBuilder}};

const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_LUNARG_standard_validation"];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;

#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

struct GraphicsApplication {
    instance: Arc<Instance>,
    debug_callback: Option<DebugCallback>,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    presentation_queue: Arc<Queue>,
    event_loop: Option<EventLoop<()>>,
    surface: Arc<Surface<Window>>,
    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
}

impl GraphicsApplication {
    pub fn new() -> Self {
        let instance = Self::create_vk_instance();
        let debug_callback = Self::create_debug_callback(&instance);
        let (event_loop, surface) = Self::create_surface(&instance);
        let (device, graphics_queue, presentation_queue) = create_device(&surface, &instance);
        let (swap_chain, swap_chain_images) = create_swap_chain(
            &instance, 
            &surface,
            device.physical_device().index(), 
            &device, 
            &graphics_queue, 
            &presentation_queue, 
            800, 
            600);

        Self {
            instance,
            debug_callback,
            device,
            graphics_queue,
            presentation_queue,
            event_loop: Some(event_loop),
            surface,
            swap_chain,
            swap_chain_images
        }
    }

    fn main_loop(&mut self) {
        let our_window_id = self.surface.window().id().clone();
        loop {
            self.event_loop.take().unwrap().run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == our_window_id => *control_flow = ControlFlow::Exit,
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } => {
                        println!("{:?} {:?}", window_id, our_window_id)
                    }
                    _ => (),
                }
            });
        }
    }

    fn create_surface(instance: &Arc<Instance>) -> (EventLoop<()>, Arc<Surface<Window>>) {
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_title("Vulkan")
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        (event_loop, surface)
    }

    fn create_vk_instance() -> Arc<Instance> {
        let supported_extensions =
            InstanceExtensions::supported_by_core().expect("Failed to get supported extensions");

        info!("Supported extensions: {:?}", supported_extensions);

        let app_info = app_info_from_cargo_toml!();
        let required_extensions = vulkano_win::required_extensions();

        if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
            Instance::new(
                Some(&app_info),
                Version::V1_1,
                &required_extensions,
                VALIDATION_LAYERS.iter().cloned(),
            )
            .expect("failed to create Vulkan instance")
        } else {
            Instance::new(Some(&app_info), Version::V1_1, &required_extensions, None)
                .expect("failed to create Vulkan instance")
        }
    }

    fn check_validation_layer_support() -> bool {
        let layers: Vec<_> = layers_list()
            .unwrap()
            .map(|layer| layer.name().to_owned())
            .collect();
        VALIDATION_LAYERS
            .iter()
            .all(|layer_name| layers.contains(&layer_name.to_string()))
    }

    fn get_required_extensions() -> InstanceExtensions {
        let mut extensions = required_extensions();

        if ENABLE_VALIDATION_LAYERS {
            extensions.ext_debug_utils = true;
        }

        extensions
    }

    fn create_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }

        let msg_types = MessageType::all();
        let severipy = MessageSeverity {
            error: true,
            warning: true,
            information: true,
            verbose: false,
        };

        DebugCallback::new(&instance, severipy, msg_types, |msg| {
            println!("validation layer: {:?}", msg.description)
        })
        .ok()
    }
}

fn main() {
    let mut app = GraphicsApplication::new();
    app.main_loop();
}
