use log::info;
use std::{iter::Inspect, sync::Arc};
use vulkano::{
    app_info_from_cargo_toml,
    instance::{ApplicationInfo, Instance, InstanceExtensions, Version},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct GraphicsApplication {
    instance: Arc<Instance>,
}

impl GraphicsApplication {
    pub fn new() -> Self {
        let instance = Self::create_vk_instance();
        Self { instance }
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

        Instance::new(Some(&app_info), Version::V1_1, &required_extensions, None).unwrap()
    }
}

fn main() {
    let mut app = GraphicsApplication::new();
    app.main_loop();
}
