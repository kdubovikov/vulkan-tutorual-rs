mod device;
mod swapchain;
mod vertex;

use device::create_device;
use log::info;
use vertex::vertecies;
use std::{cmp::Ordering, iter::Inspect, ops::Bound, sync::Arc};
use swapchain::create_swap_chain;
use vulkano::{app_info_from_cargo_toml, buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer, ImmutableBuffer}, command_buffer::{
        AutoCommandBufferBuilder, DynamicState, PrimaryAutoCommandBuffer, SubpassContents,
    }, device::{Device, Queue, QueuesIter}, format::Format, image::{view::ImageView, SwapchainImage}, instance::{
        debug::{DebugCallback, MessageSeverity, MessageType},
        layers_list, ApplicationInfo, Instance, InstanceExtensions, Version,
    }, pipeline::{GraphicsPipeline, GraphicsPipelineAbstract, GraphicsPipelineBuilder, vertex::{BufferlessDefinition, BufferlessVertices, SingleBufferDefinition}, viewport::Viewport}, query::QueriesRange, render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass}, swapchain::{acquire_next_image, Surface, Swapchain}, sync::{self, GpuFuture}};
use vulkano_win::{required_extensions, VkSurfaceBuild};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_LUNARG_standard_validation"];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;

#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/triangle.vert"
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/triangle.frag"
    }
}

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
    render_pass: Arc<RenderPass>,
    graphics_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
    previous_frame_end: Option<Box<GpuFuture>>,
    recreate_swap_chain: bool,
    vertex_buffer: Arc<BufferAccess + Send + Sync>,
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
            None
        );

        let render_pass = Self::create_render_pass(&device, swap_chain.format());
        let graphics_pipeline =
            Self::create_graphics_pipeline(&device, swap_chain.dimensions(), &render_pass);
        let framebuffers = Self::create_framebuffers(&swap_chain_images, &render_pass);

        let vertex_buffer = Self::create_vertex_buffer(&graphics_queue);
        let command_buffers = framebuffers
            .iter()
            .map(|framebuffer| {
                let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
                    device.clone(),
                    graphics_queue.family(),
                    vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                command_buffer_builder
                    .begin_render_pass(
                        framebuffer.clone(),
                        SubpassContents::Inline,
                        vec![[0.0, 0.0, 0.0, 1.0].into()],
                    )
                    .unwrap()
                    .draw(
                        graphics_pipeline.clone(),
                        &DynamicState::none(),
                        vec![vertex_buffer.clone()],
                        (),
                        (),
                        vec![],
                    )
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                Arc::new(command_buffer_builder.build().unwrap())
            })
            .collect();

        let previous_frame_end = Some(Self::create_sync_objects(&device));

        Self {
            instance,
            debug_callback,
            device,
            graphics_queue,
            presentation_queue,
            event_loop: Some(event_loop),
            surface,
            swap_chain,
            swap_chain_images,
            render_pass,
            graphics_pipeline,
            framebuffers,
            command_buffers,
            previous_frame_end,
            recreate_swap_chain: false,
            vertex_buffer
        }
    }

    fn main_loop(&mut self) {
        let our_window_id = self.surface.window().id().clone();
        loop {
            self.draw_frame();

            self.event_loop
                .take()
                .unwrap()
                .run(move |event, _, control_flow| {
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

    fn create_vertex_buffer(queue: &Arc<Queue>) -> Arc<dyn BufferAccess + Send + Sync> {
        let vert = vertecies();
        let (buffer, future) = ImmutableBuffer::from_iter(vert.iter().cloned(), BufferUsage::vertex_buffer(), queue.clone()).unwrap();
        future.flush().unwrap();
        buffer
    }

    fn recreate_swap_chain(&mut self) {
        if self.recreate_swap_chain {
            print!("Recreating swap chain");
            let (swap_chain, swap_chain_images) = create_swap_chain(
                &self.instance,
                &self.surface,
                self.device.physical_device().index(),
                &self.device,
                &self.graphics_queue,
                &self.presentation_queue,
                Some(&self.swap_chain)
            );

            self.swap_chain = swap_chain;
            self.swap_chain_images = swap_chain_images;
            self.render_pass = Self::create_render_pass(&self.device, self.swap_chain.format());
            self.graphics_pipeline = Self::create_graphics_pipeline(&self.device, self.swap_chain.dimensions(), &self.render_pass);
            self.framebuffers = Self::create_framebuffers(&self.swap_chain_images, &self.render_pass);
            self.create_command_buffers();

            self.recreate_swap_chain = false;
        }
    }

    fn create_sync_objects(device: &Arc<Device>) -> Box<GpuFuture> {
        Box::new(sync::now(device.clone())) as Box<GpuFuture>
    }

    fn draw_frame(&mut self) {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        self.recreate_swap_chain();

        let (image_index, _, acquire_future) = match acquire_next_image(self.swap_chain.clone(), None) {
            Ok(result) => result,

            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                self.recreate_swap_chain = true;
                return;
            }

            Err(e) => panic!("{:?}", e)

        };
        let command_buffer = self.command_buffers[image_index].clone();

        let future = self.previous_frame_end.take().unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.presentation_queue.clone(), self.swap_chain.clone(), image_index)
            .then_signal_fence_and_flush();


        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(sync::FlushError::OutOfDate) => {
                self.recreate_swap_chain = true;
                self.previous_frame_end = Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end = Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
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

    fn create_graphics_pipeline(
        device: &Arc<Device>,
        swap_chain_extent: [u32; 2],
        render_pass: &Arc<RenderPass>,
    ) -> Arc<GraphicsPipelineAbstract + Send + Sync> {
        let vert_shader_module = vertex_shader::Shader::load(device.clone())
            .expect("Failed to create vertex shader module");
        let frag_shader_module = fragment_shader::Shader::load(device.clone())
            .expect("Failed to create fragment shader module");

        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0..1.0,
        };

        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<vertex::Vertex>()
                .vertex_shader(vert_shader_module.main_entry_point(), ())
                .triangle_list()
                .primitive_restart(false)
                .viewports(vec![viewport])
                .fragment_shader(frag_shader_module.main_entry_point(), ())
                .depth_clamp(false)
                .polygon_mode_fill()
                .line_width(1.0)
                .cull_mode_back()
                .front_face_clockwise()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .blend_pass_through()
                .build(device.clone())
                .unwrap(),
        )
    }

    fn create_render_pass(device: &Arc<Device>, color_format: Format) -> Arc<RenderPass> {
        Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: color_format,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        )
    }

    fn create_framebuffers(
        swap_chain_images: &[Arc<SwapchainImage<Window>>],
        render_pass: &Arc<RenderPass>,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        swap_chain_images
            .iter()
            .map(|image| {
                // creating a view is necessary in 0.24, but vulkano docs do not mention this
                let view = ImageView::new(image.clone()).unwrap();
                let framebuffer = Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(view)
                        .unwrap()
                        .build()
                        .unwrap(),
                );

                framebuffer as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect()
    }

    fn create_command_buffers(&mut self) {}
}

fn main() {
    let mut app = GraphicsApplication::new();
    app.main_loop();
}
