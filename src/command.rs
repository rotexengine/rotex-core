use ash::vk;

use crate::device::{QueueCategory, Device};
use crate::error::{vk_error, ErrorKind, Error, Severity};
use crate::pass::RenderPass;
use crate::framebuffer::Framebuffer;

pub struct CommandBuffer {
    pub(crate) handle: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    pub fn begin(
        &self,
        device: &Device,
        flags: vk::CommandBufferUsageFlags,
    ) -> Result<(), Error> {
        let begin_info = vk::CommandBufferBeginInfo::default().flags(flags);

        unsafe {
            device
                .logical_device()
                .begin_command_buffer(self.handle, &begin_info)
        }
        .map_err(vk_error)
    }

    pub fn begin_render_pass(
        &self,
        device: &Device,
        render_pass: &RenderPass,
        framebuffer: &Framebuffer,
        clear_values: &[vk::ClearValue],
    ) {
        debug_assert_eq!(
            clear_values.len() as u32,
            render_pass.attachments().len() as u32,
            "Rotex Core Panic: The number of clear values does not match the Render Pass attachment count!"
        );

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(render_pass.handle())
            .framebuffer(framebuffer.handle())
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: framebuffer.extent(),
            })
            .clear_values(clear_values);

        unsafe {
            device.logical_device().cmd_begin_render_pass(
                self.handle,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn end_render_pass(&self, device: &Device) {
        unsafe {
            device.logical_device().cmd_end_render_pass(self.handle);
        }
    }

    pub fn end(&self, device: &Device) -> Result<(), Error> {
        unsafe { device.logical_device().end_command_buffer(self.handle) }.map_err(vk_error)
    }

    pub fn bind_graphics_pipeline(&self, device: &Device, pipeline: vk::Pipeline) {
        unsafe {
            device.logical_device().cmd_bind_pipeline(
                self.handle,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
        }
    }

    pub fn bind_vertex_buffer(&self, device: &Device, buffer: vk::Buffer) {
        unsafe {
            device
                .logical_device()
                .cmd_bind_vertex_buffers(self.handle, 0, &[buffer], &[0]);
        }
    }

    pub fn draw(&self, device: &Device, vertex_count: u32) {
        unsafe {
            device
                .logical_device()
                .cmd_draw(self.handle, vertex_count, 1, 0, 0);
        }
    }

    pub fn set_viewport(&self, device: &Device, viewport: vk::Viewport) {
        unsafe {
            device.logical_device().cmd_set_viewport(self.handle, 0, &[viewport]);
        }
    }

    pub fn set_scissor(&self, device: &Device, scissor: vk::Rect2D) {
        unsafe {
            device.logical_device().cmd_set_scissor(self.handle, 0, &[scissor]);
        }
    }
}

pub struct CommandPool {
    pub(crate) handle: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: &Device) -> Result<Self, Error> {
        let graphics_queue = device
            .queues()
            .iter()
            .find(|q| q.category == QueueCategory::Graphics)
            .ok_or(Error {
                kind: ErrorKind::NoCompatibleDevice,
                severity: Severity::Fatal,
            })?;

        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_queue.family_index);

        let handle =
            unsafe { device.logical_device().create_command_pool(&pool_info, None) }
                .map_err(vk_error)?;

        Ok(Self { handle })
    }

    pub fn allocate_buffers(
        &self,
        device: &Device,
        count: u32,
    ) -> Result<Vec<CommandBuffer>, Error> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.handle)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let handles =
            unsafe { device.logical_device().allocate_command_buffers(&alloc_info) }
                .map_err(vk_error)?;

        Ok(handles
            .into_iter()
            .map(|handle| CommandBuffer { handle })
            .collect())
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.logical_device().destroy_command_pool(self.handle, None);
        }
    }
}
