use crate::error::Error;
use rotex_types::{
    CreatedResources, Extent2D, RenderCommand, ResourceBatchCreate, ResourceBatchUpdate,
    SceneDescriptor, SurfaceDescriptor, TextureId, TextureReadback,
};

pub trait RenderBackend {
    fn attach_surface(&mut self, surface_descriptor: SurfaceDescriptor) -> Result<(), Error>;

    fn create_resources(
        &mut self,
        descriptor: ResourceBatchCreate,
    ) -> Result<CreatedResources, Error>;

    fn update_resources(&mut self, descriptor: ResourceBatchUpdate) -> Result<(), Error>;

    fn execute(&mut self, scene: &SceneDescriptor, commands: &[RenderCommand])
    -> Result<(), Error>;

    fn resize(&mut self, extent: Extent2D) -> Result<(), Error>;

    fn read_texture(&mut self, id: TextureId) -> Result<TextureReadback, Error>;

    fn destroy(self: Box<Self>);
}
