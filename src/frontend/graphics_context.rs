use crate::backend::GpuBackend;
use crate::error::Error;
use rotex_types::{
    CreatedResources, Extent2D, RenderCommand, ResourceBatchCreate, ResourceBatchUpdate,
    SceneDescriptor, SurfaceDescriptor, TextureId, TextureReadback,
};

pub struct GraphicsContext {
    backend: Box<dyn GpuBackend>,
}

impl GraphicsContext {
    pub fn new(backend: Box<dyn GpuBackend>) -> Self {
        Self { backend }
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
        commands: &[RenderCommand],
    ) -> Result<(), Error> {
        self.backend.execute(scene_descriptor, commands)
    }

    pub fn resize(&mut self, extent: Extent2D) -> Result<(), Error> {
        self.backend.resize(extent)
    }

    pub fn read_texture(&mut self, id: TextureId) -> Result<TextureReadback, Error> {
        self.backend.read_texture(id)
    }

    pub fn destroy(self) {
        self.backend.destroy();
    }
}
