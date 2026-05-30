use ash::vk;

use crate::error::vk_error;
use crate::{Device, Error, Instance};

pub struct ImageDescriptor {
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub usage: vk::ImageUsageFlags,
    pub properties: vk::MemoryPropertyFlags,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub image_type: vk::ImageType,
    pub view_type: vk::ImageViewType,
    pub tiling: vk::ImageTiling,
    pub samples: vk::SampleCountFlags,
}

impl ImageDescriptor {
    pub fn New(
        format: vk::Format,
        extent: vk::Extent3D,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Self {
        Self {
            format,
            extent,
            usage,
            properties,
            mip_levels: 1,
            array_layers: 1,
            image_type: vk::ImageType::TYPE_2D,
            view_type: vk::ImageViewType::TYPE_2D,
            tiling: vk::ImageTiling::OPTIMAL,
            samples: vk::SampleCountFlags::TYPE_1,
        }
    }

    pub fn WithMipLevels(mut self, levels: u32) -> Self {
        self.mip_levels = levels;
        self
    }

    pub fn WithArrayLayers(mut self, layers: u32, viewType: vk::ImageViewType) -> Self {
        self.array_layers = layers;
        self.view_type = viewType;
        self
    }
}

pub struct RotexImage {
    image_handle: vk::Image,
    device_memory: vk::DeviceMemory,
    image_view: vk::ImageView,
}

impl RotexImage {
    pub fn New(instance: &Instance, device: &Device, desc: ImageDescriptor) -> Result<Self, Error> {
        // 1. Create the Image using the descriptor's flexibility
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(desc.image_type)
            .format(desc.format)
            .extent(desc.extent)
            .mip_levels(desc.mip_levels)
            .array_layers(desc.array_layers)
            .samples(desc.samples)
            .tiling(desc.tiling)
            .usage(desc.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image_handle = unsafe {
            device.logical_device().create_image(&image_create_info, None)
        }
        .map_err(vk_error)?;

        // 2. Allocate and Bind Memory (Your logic here is already perfect)
        let mem_requirements = unsafe {
            device.logical_device().get_image_memory_requirements(image_handle)
        };

        let memory_type_index = device.find_memory_type(
            instance, 
            mem_requirements.memory_type_bits, 
            desc.properties
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let device_memory = unsafe {
            device.logical_device().allocate_memory(&alloc_info, None)
        }
        .map_err(vk_error)?;

        unsafe { device.logical_device().bind_image_memory(image_handle, device_memory, 0) }
            .map_err(vk_error)?;

        let aspect_mask = if desc.usage.contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
            vk::ImageAspectFlags::DEPTH
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let view_create_info = vk::ImageViewCreateInfo::default()
            .image(image_handle)
            .view_type(desc.view_type)
            .format(desc.format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: desc.mip_levels,
                base_array_layer: 0,
                layer_count: desc.array_layers,
            });

        let image_view = unsafe {
            device.logical_device().create_image_view(&view_create_info, None)
        }
        .map_err(vk_error)?;

        Ok(Self {
            image_handle,
            device_memory,
            image_view,
        })
    }
    
    pub fn handle(&self) -> vk::Image {
        self.image_handle
    }

    pub fn view(&self) -> vk::ImageView {
        self.image_view
    }
    
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.logical_device().destroy_image_view(self.image_view, None);
            device.logical_device().destroy_image(self.image_handle, None);
            device.logical_device().free_memory(self.device_memory, None);
        }
    }
}