use crate::error::Error;
use crate::bridge::BackendBridge;
use rotex_types::{
    CreatedResources, DeviceDescriptor, Extent2D, FrameDescriptor, InstanceDescriptor, ResourceBatchCreate,
    ResourceBatchUpdate, SceneDescriptor, SurfaceDescriptor,
};

/// Active graphics backend and rendering session state.
pub struct GraphicsContext {
    backend: BackendBridge,
}

impl GraphicsContext {
    /// Initializes the platform backend from instance and device descriptors.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when backend creation fails.
    pub async fn new(
        instance_descriptor: InstanceDescriptor,
        device_descriptor: DeviceDescriptor,
    ) -> Result<Self, Error> {
        Ok(Self {
            backend: BackendBridge::new(instance_descriptor, device_descriptor).await?,
        })
    }

    /// Binds a window surface for presentation.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when surface creation or attachment fails.
    pub fn attach_surface(&mut self, surface_descriptor: SurfaceDescriptor) -> Result<(), Error> {
        self.backend.attach_surface(surface_descriptor)
    }

    /// Creates GPU resources described by `descriptor`.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when resource allocation or upload fails.
    pub fn create_resources(
        &mut self,
        descriptor: ResourceBatchCreate,
    ) -> Result<CreatedResources, Error> {
        self.backend.create_resources(descriptor)
    }

    /// Updates existing GPU resources described by `descriptor`.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when a resource update fails.
    pub fn update_resources(&mut self, descriptor: ResourceBatchUpdate) -> Result<(), Error> {
        self.backend.update_resources(descriptor)
    }

    /// Records and submits a frame from `scene_descriptor` and `frame_descriptor`.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when rendering or presentation fails.
    pub fn render(
        &mut self,
        scene_descriptor: &SceneDescriptor,
        frame_descriptor: &FrameDescriptor,
    ) -> Result<(), Error> {
        self.backend.render(scene_descriptor, frame_descriptor)
    }

    /// Resizes the swapchain and dependent targets to `extent`.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when swapchain recreation fails.
    pub fn resize(&mut self, extent: Extent2D) -> Result<(), Error> {
        self.backend.resize(extent)
    }

    /// Releases backend resources and consumes the context.
    pub fn destroy(self) {
        self.backend.destroy();
    }
}