use log::info;
use vulkano_win::required_extensions;
use std::{iter::Inspect, sync::Arc};
use vulkano::{app_info_from_cargo_toml, instance::{ApplicationInfo, Instance, InstanceExtensions, Version, debug::{DebugCallback, MessageSeverity, MessageType}, layers_list}};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const VALIDATION_LAYERS: &[&str] = &[
    "VK_LAYER_LUNARG_standard_validation"
];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;

#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

struct GraphicsApplication {
    instance: Arc<Instance>,
    debug_callback: Option<DebugCallback>,
}

impl GraphicsApplication {
    pub fn new() -> Self {
        let instance = Self::create_vk_instance();
        let debug_callback = Self::create_debug_callback(&instance);
        Self { instance, debug_callback }
    }

    fn main_loop(&self) {
        let (event_loop, window) = Self::create_window();
        loop {
            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                    _ => (),
                }
            });
        }
    }

    fn create_window() -> (EventLoop<()>, Window) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Vulkan")
            .build(&event_loop)
            .unwrap();

        (event_loop, window)
    }

    fn create_vk_instance() -> Arc<Instance> {
        let supported_extensions =
            InstanceExtensions::supported_by_core().expect("Failed to get supported extensions");

        info!("Supported extensions: {:?}", supported_extensions);

        let app_info = app_info_from_cargo_toml!();
        let required_extensions = vulkano_win::required_extensions();

        if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
            Instance::new(Some(&app_info), Version::V1_1, &required_extensions, VALIDATION_LAYERS.iter().cloned())
                .expect("failed to create Vulkan instance")
        } else {
            Instance::new(Some(&app_info), Version::V1_1, &required_extensions, None)
                .expect("failed to create Vulkan instance")
        }
    }

    fn check_validation_layer_support() -> bool {
            let layers: Vec<_> = layers_list()
                .unwrap()
                .map(|layer| 
                    layer.name().to_owned()
                ).collect();
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
            verbose: false
        };

        DebugCallback::new(&instance, severipy, msg_types, |msg| {
            println!("validation layer: {:?}", msg.description)
        }).ok()
    }
}

fn main() {
    let mut app = GraphicsApplication::new();
    app.main_loop();
}
