// rotex_core/src/renderer.rs (Formerly engine.rs)

use crate::error::Error;
use crate::bridge::BackendBridge;
use rotex_types::{
    CreatedResources, DeviceDescriptor, Extent2D, FrameDescriptor, InstanceDescriptor, ResourceBatchCreate,
    ResourceBatchUpdate, SceneDescriptor, SurfaceDescriptor,
};

pub struct GraphicsContext {
    backend: BackendBridge,
}

impl GraphicsContext {
    pub async fn new(
        instance_descriptor: InstanceDescriptor,
        device_descriptor: DeviceDescriptor,
    ) -> Result<Self, Error> {
        Ok(Self {
            backend: BackendBridge::new(instance_descriptor, device_descriptor).await?,
        })
    }

    pub fn attach_surface(&mut self, surface_descriptor: SurfaceDescriptor) -> Result<(), Error> {
        self.backend.attach_surface(surface_descriptor)
    }

    pub fn create_resources(
        &mut self,
        descriptor: ResourceBatchCreate,
    ) -> Result<CreatedResources, Error> {
        self.backend.create_resources(descriptor)
    }

    pub fn update_resources(&mut self, descriptor: ResourceBatchUpdate) -> Result<(), Error> {
        self.backend.update_resources(descriptor)
    }

    pub fn render(
        &mut self,
        scene_descriptor: &SceneDescriptor,
        frame_descriptor: &FrameDescriptor,
    ) -> Result<(), Error> {
        self.backend.render(scene_descriptor, frame_descriptor)
    }

    pub fn resize(&mut self, extent: Extent2D) -> Result<(), Error> {
        self.backend.resize(extent)
    }

    pub fn destroy(self) {
        self.backend.destroy();
    }
}