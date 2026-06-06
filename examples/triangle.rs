#[cfg(not(target_arch = "wasm32"))]
use rotex_core::{
    CullMode, DeviceDescriptor, Extent2D, GraphicsContext, IndexFormat, InstanceDescriptor,
    MaterialDescriptor, MaterialId, MeshDescriptor, MeshId, MeshInstanceDescriptor, PassDescriptor,
    RenderCommand, ResourceBatchCreate, ResourceCreateDescriptor, ResourceHandle, SceneDescriptor,
    SurfaceDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat,
};
#[cfg(not(target_arch = "wasm32"))]
use rotex_vulkan::VulkanBridge;
#[cfg(not(target_arch = "wasm32"))]
use rotex_window::{EngineApp, EngineEvent, WindowBridge, WindowDescriptor, WindowOrchestrator};
#[cfg(not(target_arch = "wasm32"))]
use rotex_winit::WinitBackend;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct App {
    graphics_context: Option<GraphicsContext>,
    scene: Option<SceneDescriptor>,
    commands: Option<Vec<RenderCommand>>,
}

#[cfg(not(target_arch = "wasm32"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct ColoredVertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[cfg(not(target_arch = "wasm32"))]
impl EngineApp for App {
    fn on_event(&mut self, event: EngineEvent, window: &dyn WindowBridge) {
        match event {
            EngineEvent::Init => {
                let display_handle = match window.display_handle() {
                    Ok(handle) => handle.as_raw(),
                    Err(err) => {
                        eprintln!("display handle unavailable: {err}");
                        return;
                    }
                };
                let window_handle = match window.window_handle() {
                    Ok(handle) => handle.as_raw(),
                    Err(err) => {
                        eprintln!("window handle unavailable: {err}");
                        return;
                    }
                };

                let mut instance_descriptor = InstanceDescriptor::default();
                instance_descriptor.required_instance_extensions =
                    match ash_window::enumerate_required_extensions(display_handle) {
                        Ok(extensions) => extensions
                            .iter()
                            .map(|extension| unsafe { std::ffi::CStr::from_ptr(*extension) })
                            .map(|extension| extension.to_string_lossy().into_owned())
                            .collect(),
                        Err(err) => {
                            eprintln!("failed to enumerate required instance extensions: {err}");
                            return;
                        }
                    };
                let backend =
                    match VulkanBridge::new(instance_descriptor, DeviceDescriptor::default()) {
                        Ok(backend) => backend,
                        Err(err) => {
                            eprintln!("triangle backend initialization failed: {err}");
                            return;
                        }
                    };
                let mut graphics_context = GraphicsContext::new(Box::new(backend));
                let (width, height) = window.extent();
                if let Err(err) = graphics_context.attach_surface(SurfaceDescriptor {
                    display_handle,
                    window_handle,
                    extent: Extent2D { width, height },
                }) {
                    eprintln!("attach surface failed: {err}");
                    return;
                }

                let resources =
                    match graphics_context.create_resources(ResourceBatchCreate::new(vec![
                        ResourceCreateDescriptor::Mesh(mesh_from_vertices(
                            &[
                                ColoredVertex {
                                    position: [0.0, -0.6, 0.5],
                                    color: [1.0, 0.2, 0.3],
                                },
                                ColoredVertex {
                                    position: [0.6, 0.6, 0.5],
                                    color: [0.2, 1.0, 0.3],
                                },
                                ColoredVertex {
                                    position: [-0.6, 0.6, 0.5],
                                    color: [0.2, 0.4, 1.0],
                                },
                            ],
                            &[0, 1, 2],
                        )),
                        ResourceCreateDescriptor::Material(MaterialDescriptor {
                            vertex_shader_spv: include_bytes!(concat!(
                                env!("OUT_DIR"),
                                "/triangle.vert.spv"
                            ))
                            .to_vec(),
                            vertex_entry: "main".to_string(),
                            fragment_shader_spv: include_bytes!(concat!(
                                env!("OUT_DIR"),
                                "/triangle.frag.spv"
                            ))
                            .to_vec(),
                            fragment_entry: "main".to_string(),
                            enable_depth: false,
                            cull_mode: CullMode::None,
                            texture: None,
                        }),
                    ])) {
                        Ok(resources) => resources,
                        Err(err) => {
                            eprintln!("triangle resources creation failed: {err}");
                            return;
                        }
                    };
                let mesh_id = expect_mesh(resources.handles[0]);
                let material_id = expect_material(resources.handles[1]);
                let scene =
                    SceneDescriptor::new(vec![MeshInstanceDescriptor::new(mesh_id, material_id)]);
                let commands = vec![RenderCommand::DrawGraphics(
                    PassDescriptor::new("main").with_clear_color([0.06, 0.06, 0.09, 1.0]),
                )];

                self.graphics_context = Some(graphics_context);
                self.scene = Some(scene);
                self.commands = Some(commands);
            }
            EngineEvent::Resized(width, height) => {
                if let Some(graphics_context) = self.graphics_context.as_mut() {
                    if let Err(err) = graphics_context.resize(Extent2D { width, height }) {
                        eprintln!("resize failed: {err}");
                    }
                }
            }
            EngineEvent::Render => {
                if let (Some(graphics_context), Some(scene), Some(commands)) = (
                    self.graphics_context.as_mut(),
                    self.scene.as_ref(),
                    self.commands.as_ref(),
                ) {
                    if let Err(err) = graphics_context.render(scene, commands) {
                        eprintln!("render failed: {err}");
                    }
                }
            }
            EngineEvent::CloseRequested => {
                if let Some(graphics_context) = self.graphics_context.take() {
                    graphics_context.destroy();
                }
            }
            _ => {}
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor = WindowDescriptor {
        title: "Rotex Frontend Triangle".to_string(),
        width: 900,
        height: 600,
        ..Default::default()
    };
    let orchestrator = WindowOrchestrator::new(Box::new(WinitBackend));
    orchestrator.run(descriptor, Box::new(App::default()))?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("triangle example is only available on native targets.");
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_mesh(handle: ResourceHandle) -> MeshId {
    match handle {
        ResourceHandle::Mesh(id) => id,
        _ => panic!("expected mesh handle"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_material(handle: ResourceHandle) -> MaterialId {
    match handle {
        ResourceHandle::Material(id) => id,
        _ => panic!("expected material handle"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn mesh_from_vertices(vertices: &[ColoredVertex], indices: &[u32]) -> MeshDescriptor {
    MeshDescriptor {
        vertex_data: vertices_as_bytes(vertices),
        vertex_layout: colored_vertex_layout(),
        index_data: indices_as_bytes(indices),
        index_format: IndexFormat::Uint32,
        index_count: indices.len() as u32,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn colored_vertex_layout() -> VertexBufferLayout {
    VertexBufferLayout {
        array_stride: 24,
        attributes: vec![
            VertexAttribute {
                location: 0,
                format: VertexFormat::Float32x3,
                offset: 0,
            },
            VertexAttribute {
                location: 1,
                format: VertexFormat::Float32x3,
                offset: 12,
            },
        ],
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn vertices_as_bytes(vertices: &[ColoredVertex]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vertices.len() * 24);
    for vertex in vertices {
        for component in vertex.position {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
        for component in vertex.color {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
    }
    bytes
}

#[cfg(not(target_arch = "wasm32"))]
fn indices_as_bytes(indices: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(indices.len() * 4);
    for index in indices {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes
}
