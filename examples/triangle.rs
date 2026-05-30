use std::mem::size_of;

use ash::vk;
use rotex_core::{
    ColorBlendAttachmentState, ColorBlendState, CommandBuffer, CommandPool, DebugMessenger, Device,
    DeviceDescriptor, Error, ErrorKind, Fence, Framebuffer, FramebufferBuilder, GraphicsPipeline,
    GraphicsPipelineBuilder, GraphicsPipelineLayout, Instance, QueueCategory, QueueRequest,
    RasterizationState, RenderPass, RenderPassBuilder, Semaphore, Severity, ShaderModule,
    ShaderStageDescriptor, SubpassBlueprint, Surface, Swapchain, Vertex, VertexInputDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};

#[repr(C)]
#[derive(Clone, Copy)]
struct TriangleVertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex for TriangleVertex {
    fn descriptor() -> VertexInputDescriptor {
        VertexInputDescriptor::new()
            .with_binding(
                vk::VertexInputBindingDescription::default()
                    .binding(0)
                    .stride(size_of::<TriangleVertex>() as u32)
                    .input_rate(vk::VertexInputRate::VERTEX),
            )
            .with_attribute(
                vk::VertexInputAttributeDescription::default()
                    .binding(0)
                    .location(0)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(0),
            )
            .with_attribute(
                vk::VertexInputAttributeDescription::default()
                    .binding(0)
                    .location(1)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(8),
            )
    }
}

const TRIANGLE_VERTICES: [TriangleVertex; 3] = [
    TriangleVertex {
        position: [0.0, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    TriangleVertex {
        position: [0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    TriangleVertex {
        position: [-0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

struct App {
    instance: Option<Instance>,
    debug_messenger: Option<DebugMessenger>,
    window: Option<Window>,
    renderer: Option<Renderer>,
}

struct VertexBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl VertexBuffer {
    fn new(
        instance: &Instance,
        device: &Device,
        vertices: &[TriangleVertex],
    ) -> Result<Self, Error> {
        let size = size_of_val(vertices) as vk::DeviceSize;

        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.logical_device().create_buffer(&buffer_info, None) }
            .map_err(vulkan_error)?;

        let requirements = unsafe {
            device
                .logical_device()
                .get_buffer_memory_requirements(buffer)
        };

        let memory_type = find_memory_type(
            instance,
            device.physical_device(),
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type);

        let memory = unsafe { device.logical_device().allocate_memory(&alloc_info, None) }
            .map_err(vulkan_error)?;

        unsafe {
            device
                .logical_device()
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(vulkan_error)?;

            let data = device
                .logical_device()
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(vulkan_error)?;

            std::ptr::copy_nonoverlapping(vertices.as_ptr(), data.cast(), vertices.len());
            device.logical_device().unmap_memory(memory);
        }

        Ok(Self { buffer, memory })
    }

    fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    fn destroy(self, device: &Device) {
        unsafe {
            device.logical_device().destroy_buffer(self.buffer, None);
            device.logical_device().free_memory(self.memory, None);
        }
    }
}

struct Renderer {
    instance: Instance,
    debug_messenger: Option<DebugMessenger>,
    device: Device,
    surface: Surface,
    swapchain: Swapchain,
    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,
    pipeline_layout: GraphicsPipelineLayout,
    pipeline: GraphicsPipeline,
    vert_shader: ShaderModule,
    frag_shader: ShaderModule,
    vertex_buffer: VertexBuffer,
    command_pool: CommandPool,
    command_buffer: CommandBuffer,
    image_available: Semaphore,
    render_finished: Vec<Semaphore>,
    in_flight_fence: Fence,
    graphics_queue_index: u32,
    extent_hint: vk::Extent2D,
    swapchain_outdated: bool,
}

fn vulkan_error(err: vk::Result) -> Error {
    Error {
        kind: ErrorKind::Vulkan(err),
        severity: Severity::Fatal,
    }
}

fn is_swapchain_out_of_date(err: &Error) -> bool {
    matches!(
        err.vk_result_code(),
        Some(code) if code == vk::Result::ERROR_OUT_OF_DATE_KHR.as_raw()
    )
}

fn spirv_words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("valid spv word")))
        .collect()
}

fn load_shaders(device: &Device) -> Result<(ShaderModule, ShaderModule), Error> {
    let vert_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/triangle.vert.spv"));
    let frag_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/triangle.frag.spv"));

    let vert_words = spirv_words(vert_bytes);
    let frag_words = spirv_words(frag_bytes);

    let vert_shader = ShaderModule::new(device, &vert_words)?;
    let frag_shader = ShaderModule::new(device, &frag_words)?;
    Ok((vert_shader, frag_shader))
}

fn find_memory_type(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Result<u32, Error> {
    let memory_properties = unsafe {
        instance
            .instance()
            .get_physical_device_memory_properties(physical_device)
    };

    memory_properties
        .memory_types
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (type_filter & (1 << index)) != 0 && memory_type.property_flags.contains(properties)
        })
        .map(|(index, _)| index as u32)
        .ok_or(Error {
            kind: ErrorKind::NoCompatibleDevice,
            severity: Severity::Fatal,
        })
}

fn window_extent(window: &Window) -> vk::Extent2D {
    let size = window.inner_size();
    vk::Extent2D {
        width: size.width.max(1),
        height: size.height.max(1),
    }
}

fn graphics_queue_index(device: &Device) -> u32 {
    device
        .queues()
        .iter()
        .find(|queue| queue.category == QueueCategory::Graphics)
        .expect("graphics queue must exist")
        .family_index
}

fn build_framebuffers(
    device: &Device,
    swapchain: &Swapchain,
    render_pass: vk::RenderPass,
) -> Result<Vec<Framebuffer>, Error> {
    swapchain
        .image_views()
        .iter()
        .map(|view| {
            FramebufferBuilder::new()
                .with_attachment(*view)
                .with_extent(swapchain.extent().width, swapchain.extent().height)
                .build(device, render_pass)
        })
        .collect()
}

fn create_render_pass(device: &Device, swapchain: &Swapchain) -> Result<RenderPass, Error> {
    RenderPassBuilder::new()
        .with_attachment(
            vk::AttachmentDescription::default()
                .format(swapchain.format())
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
        )
        .with_subpass(SubpassBlueprint {
            color_attachments: vec![0],
            depth_attachment: None,
        })
        .build(device)
        .map_err(vulkan_error)
}

fn create_render_finished_semaphores(
    device: &Device,
    count: usize,
) -> Result<Vec<Semaphore>, Error> {
    (0..count).map(|_| Semaphore::new(device)).collect()
}

fn create_pipeline(
    device: &Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    vert_shader: &ShaderModule,
    frag_shader: &ShaderModule,
    pipeline_layout: &GraphicsPipelineLayout,
) -> Result<GraphicsPipeline, Error> {
    let color_blend = ColorBlendState::new().with_attachment(
        ColorBlendAttachmentState::new().with_color_write_mask(vk::ColorComponentFlags::RGBA),
    );

    GraphicsPipelineBuilder::new()
        .with_shader_stage(ShaderStageDescriptor::new(
            vk::ShaderStageFlags::VERTEX,
            vert_shader,
        ))
        .with_shader_stage(ShaderStageDescriptor::new(
            vk::ShaderStageFlags::FRAGMENT,
            frag_shader,
        ))
        .with_render_pass(render_pass)
        .with_layout(pipeline_layout.handle())
        .with_vertex_input_state(TriangleVertex::descriptor())
        .with_extent(extent.width, extent.height)
        .with_rasterization_state(RasterizationState::new().with_cull_mode(vk::CullModeFlags::NONE))
        .with_color_blend_state(color_blend)
        .build(device)
}

impl Renderer {
    fn new(instance: Instance, window: &Window) -> Result<Self, Error> {
        let adapter = instance
            .enumerate_adapters()
            .into_iter()
            .next()
            .ok_or(Error {
                kind: ErrorKind::NoCompatibleDevice,
                severity: Severity::Fatal,
            })?;

        let device = adapter.request_device(
            &instance,
            DeviceDescriptor {
                required_features: vk::PhysicalDeviceFeatures::default(),
                enable_swapchain: true,
                queues: vec![
                    QueueRequest {
                        category: QueueCategory::Graphics,
                        count: 1,
                    },
                    QueueRequest {
                        category: QueueCategory::Transfer,
                        count: 1,
                    },
                ],
            },
        )?;

        let graphics_queue_index = graphics_queue_index(&device);

        let raw_surface = unsafe {
            ash_window::create_surface(
                instance.entry(),
                instance.instance(),
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
        }
        .map_err(vulkan_error)?;

        let surface = Surface::new(&instance, raw_surface);
        let extent_hint = window_extent(window);
        let swapchain = Swapchain::new(&instance, &device, &surface, extent_hint)?;
        let render_pass = create_render_pass(&device, &swapchain)?;
        let framebuffers = build_framebuffers(&device, &swapchain, render_pass.handle())?;

        let (vert_shader, frag_shader) = load_shaders(&device)?;
        let pipeline_layout = GraphicsPipelineLayout::new(&device, &[], &[])?;
        let pipeline = create_pipeline(
            &device,
            render_pass.handle(),
            swapchain.extent(),
            &vert_shader,
            &frag_shader,
            &pipeline_layout,
        )?;
        let vertex_buffer = VertexBuffer::new(&instance, &device, &TRIANGLE_VERTICES)?;

        let command_pool = CommandPool::new(&device)?;
        let mut command_buffers = command_pool.allocate_buffers(&device, 1)?;
        let command_buffer = command_buffers
            .pop()
            .expect("one command buffer was requested");

        Ok(Self {
            instance,
            debug_messenger: None,
            graphics_queue_index,
            render_finished: create_render_finished_semaphores(&device, swapchain.images().len())?,
            image_available: Semaphore::new(&device)?,
            in_flight_fence: Fence::new(&device, true)?,
            device,
            surface,
            swapchain,
            render_pass,
            framebuffers,
            pipeline_layout,
            pipeline,
            vert_shader,
            frag_shader,
            vertex_buffer,
            command_pool,
            command_buffer,
            extent_hint,
            swapchain_outdated: false,
        })
    }

    fn recreate_swapchain(&mut self) -> Result<(), Error> {
        unsafe { self.device.logical_device().device_wait_idle() }.map_err(vulkan_error)?;

        for framebuffer in self.framebuffers.drain(..) {
            framebuffer.destroy(&self.device);
        }
        for semaphore in self.render_finished.drain(..) {
            semaphore.destroy(&self.device);
        }

        self.pipeline.destroy(&self.device);
        self.swapchain.destroy(&mut self.device);

        self.swapchain = Swapchain::new(
            &self.instance,
            &self.device,
            &self.surface,
            self.extent_hint,
        )?;
        self.framebuffers =
            build_framebuffers(&self.device, &self.swapchain, self.render_pass.handle())?;
        self.render_finished =
            create_render_finished_semaphores(&self.device, self.swapchain.images().len())?;
        self.pipeline = create_pipeline(
            &self.device,
            self.render_pass.handle(),
            self.swapchain.extent(),
            &self.vert_shader,
            &self.frag_shader,
            &self.pipeline_layout,
        )?;
        self.swapchain_outdated = false;
        Ok(())
    }

    fn record_triangle(&self, framebuffer: &Framebuffer) -> Result<(), Error> {
        self.command_buffer
            .begin(&self.device, vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)?;

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.05, 0.05, 0.1, 1.0],
            },
        }];

        self.command_buffer.begin_render_pass(
            &self.device,
            &self.render_pass,
            framebuffer,
            &clear_values,
        );
        self.command_buffer
            .bind_graphics_pipeline(&self.device, self.pipeline.handle());
        self.command_buffer
            .bind_vertex_buffer(&self.device, self.vertex_buffer.buffer());
        self.command_buffer
            .draw(&self.device, TRIANGLE_VERTICES.len() as u32);
        self.command_buffer.end_render_pass(&self.device);
        self.command_buffer.end(&self.device)
    }

    fn draw(&mut self) -> Result<(), Error> {
        if self.swapchain_outdated {
            self.recreate_swapchain()?;
        }

        self.in_flight_fence.wait(&self.device, u64::MAX)?;

        let image_index = match self.swapchain.acquire_next_image(&self.image_available) {
            Ok((index, _)) => index,
            Err(err) if is_swapchain_out_of_date(&err) => {
                self.swapchain_outdated = true;
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        let render_finished = &self.render_finished[image_index as usize];
        self.in_flight_fence.reset(&self.device)?;

        unsafe {
            self.device.logical_device().reset_command_buffer(
                self.command_buffer.handle(),
                vk::CommandBufferResetFlags::empty(),
            )
        }
        .map_err(vulkan_error)?;

        self.record_triangle(&self.framebuffers[image_index as usize])?;

        let wait_semaphores = [self.image_available.handle()];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.command_buffer.handle()];
        let signal_semaphores = [render_finished.handle()];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        let queue = self.device.get_queue(self.graphics_queue_index, 0);

        unsafe {
            self.device.logical_device().queue_submit(
                queue,
                &[submit_info],
                self.in_flight_fence.handle(),
            )
        }
        .map_err(vulkan_error)?;

        match self.swapchain.present(queue, image_index, render_finished) {
            Ok(_) => Ok(()),
            Err(err) if is_swapchain_out_of_date(&err) => {
                self.swapchain_outdated = true;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn destroy(mut self) {
        let _ = self.in_flight_fence.wait(&self.device, u64::MAX);
        unsafe {
            let _ = self.device.logical_device().device_wait_idle();
        }

        self.command_pool.destroy(&self.device);
        self.image_available.destroy(&self.device);
        for semaphore in self.render_finished {
            semaphore.destroy(&self.device);
        }
        self.in_flight_fence.destroy(&self.device);

        self.pipeline.destroy(&self.device);
        self.pipeline_layout.destroy(&self.device);
        self.vert_shader.destroy(&self.device);
        self.frag_shader.destroy(&self.device);
        self.vertex_buffer.destroy(&self.device);

        for framebuffer in self.framebuffers {
            framebuffer.destroy(&self.device);
        }
        self.render_pass.destroy(&self.device);
        self.swapchain.destroy(&self.device);
        self.surface.destroy();

        if let Some(debug_messenger) = self.debug_messenger {
            debug_messenger.destroy();
        }

        let mut device = self.device;
        device.destroy();
        let mut instance = self.instance;
        instance.destroy();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = match event_loop.create_window(
                Window::default_attributes()
                    .with_title("Rotex Triangle")
                    .with_inner_size(LogicalSize::new(800.0, 600.0)),
            ) {
                Ok(window) => window,
                Err(error) => {
                    eprintln!("failed to create window: {error}");
                    event_loop.exit();
                    return;
                }
            };
            self.window = Some(window);
        }

        if self.renderer.is_some() {
            return;
        }

        let window = self.window.as_ref().expect("window exists after resumed");
        let instance = self.instance.take().expect("instance set in main");
        let debug_messenger = self.debug_messenger.take();

        match Renderer::new(instance, window) {
            Ok(mut renderer) => {
                renderer.debug_messenger = debug_messenger;
                self.renderer = Some(renderer);
                window.request_redraw();
            }
            Err(error) => {
                eprintln!("failed to initialize renderer: {error}");
                if let Some(debug_messenger) = debug_messenger {
                    debug_messenger.destroy();
                }
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().map(|window| window.id()) != Some(id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.extent_hint = vk::Extent2D {
                        width: size.width.max(1),
                        height: size.height.max(1),
                    };
                    renderer.swapchain_outdated = true;
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(renderer) = self.renderer.as_mut() else {
                    return;
                };

                if let Err(error) = renderer.draw() {
                    eprintln!("failed to draw frame: {error}");
                    event_loop.exit();
                    return;
                }

                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let extensions =
        ash_window::enumerate_required_extensions(event_loop.display_handle()?.as_raw())?;
    let (instance, debug_messenger) = Instance::new(extensions).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to initialize Vulkan instance: {err}"),
        )
    })?;

    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        instance: Some(instance),
        debug_messenger,
        window: None,
        renderer: None,
    };
    event_loop.run_app(&mut app)?;

    if let Some(renderer) = app.renderer.take() {
        renderer.destroy();
    } else if let Some(debug_messenger) = app.debug_messenger.take() {
        debug_messenger.destroy();
    } else if let Some(mut instance) = app.instance.take() {
        instance.destroy();
    }

    Ok(())
}
