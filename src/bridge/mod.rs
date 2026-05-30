use crate::error::Error;
use rotex_types::{
    CreatedResources, DeviceDescriptor, Extent2D, FrameDescriptor, InstanceDescriptor, ResourceBatchCreate,
    ResourceBatchUpdate, SceneDescriptor, SurfaceDescriptor,
};

pub(crate) enum BackendBridge {
    #[cfg(not(target_arch = "wasm32"))]
    Vulkan(rotex_vulkan::VulkanBridge),
    #[cfg(target_arch = "wasm32")]
    Wgpu(rotex_wgpu::WgpuBridge),
}

impl BackendBridge {
    pub(crate) async fn new(
        instance_descriptor: InstanceDescriptor,
        device_descriptor: DeviceDescriptor,
    ) -> Result<Self, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return rotex_vulkan::VulkanBridge::new(instance_descriptor, device_descriptor)
                .map(Self::Vulkan)
                .map_err(Into::into);
        }

        #[cfg(target_arch = "wasm32")]
        {
            return rotex_wgpu::WgpuBridge::new(instance_descriptor, device_descriptor)
                .await
                .map(Self::Wgpu)
                .map_err(Into::into);
        }
    }

    pub(crate) fn attach_surface(
        &mut self,
        surface_descriptor: SurfaceDescriptor,
    ) -> Result<(), Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            BackendBridge::Vulkan(bridge) => bridge.attach_surface(surface_descriptor).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            BackendBridge::Wgpu(bridge) => bridge.attach_surface(surface_descriptor).map_err(Into::into),
        }
    }

    pub(crate) fn create_resources(
        &mut self,
        descriptor: ResourceBatchCreate,
    ) -> Result<CreatedResources, Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            BackendBridge::Vulkan(bridge) => bridge.create_resources(descriptor).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            BackendBridge::Wgpu(bridge) => bridge.create_resources(descriptor).map_err(Into::into),
        }
    }

    pub(crate) fn update_resources(
        &mut self,
        descriptor: ResourceBatchUpdate,
    ) -> Result<(), Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            BackendBridge::Vulkan(bridge) => bridge.update_resources(descriptor).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            BackendBridge::Wgpu(bridge) => bridge.update_resources(descriptor).map_err(Into::into),
        }
    }

    pub(crate) fn render(
        &mut self,
        scene: &SceneDescriptor,
        frame: &FrameDescriptor,
    ) -> Result<(), Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            BackendBridge::Vulkan(bridge) => bridge.render(scene, frame).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            BackendBridge::Wgpu(bridge) => bridge.render(scene, frame).map_err(Into::into),
        }
    }

    pub(crate) fn resize(&mut self, extent: Extent2D) -> Result<(), Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            BackendBridge::Vulkan(bridge) => bridge.resize(extent).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            BackendBridge::Wgpu(bridge) => bridge.resize(extent).map_err(Into::into),
        }
    }

    pub(crate) fn destroy(self) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            BackendBridge::Vulkan(bridge) => bridge.destroy(),
            #[cfg(target_arch = "wasm32")]
            BackendBridge::Wgpu(bridge) => bridge.destroy(),
        }
    }
}
