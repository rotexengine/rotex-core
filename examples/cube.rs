#[cfg(not(target_arch = "wasm32"))]
use rotex_core::{
    CullMode, DeviceDescriptor, Extent2D, GraphicsContext, IndexFormat, InstanceDescriptor,
    MaterialDescriptor, MaterialId, MeshDescriptor, MeshId, MeshInstanceDescriptor, PassDescriptor,
    RenderCommand, ResourceBatchCreate, ResourceBatchUpdate, ResourceCreateDescriptor,
    ResourceHandle, ResourceUpdateDescriptor, SceneDescriptor, SurfaceDescriptor,
    TextureDescriptor, TextureFormat, TextureId, VertexAttribute, VertexBufferLayout, VertexFormat,
};
#[cfg(not(target_arch = "wasm32"))]
use rotex_vulkan::VulkanBridge;
#[cfg(not(target_arch = "wasm32"))]
use rotex_window::{EngineApp, EngineEvent, WindowBridge, WindowDescriptor, WindowOrchestrator};
#[cfg(not(target_arch = "wasm32"))]
use rotex_winit::WinitBackend;

#[cfg(not(target_arch = "wasm32"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct ColoredVertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct App {
    graphics_context: Option<GraphicsContext>,
    scene: Option<SceneDescriptor>,
    commands: Option<Vec<RenderCommand>>,
    mesh_id: Option<MeshId>,
    base_positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 3]>,
    indices: Vec<u32>,
    elapsed_time: f32,
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
                            eprintln!("cube backend initialization failed: {err}");
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

                let (positions, colors, indices) = cube_geometry();
                let initial_vertices = build_vertices(&positions, &colors, 0.0);
                let resources =
                    match graphics_context.create_resources(ResourceBatchCreate::new(vec![
                        ResourceCreateDescriptor::Mesh(MeshDescriptor {
                            vertex_data: vertices_as_bytes(&initial_vertices),
                            vertex_layout: colored_vertex_layout(),
                            index_data: indices_as_bytes(&indices),
                            index_format: IndexFormat::Uint32,
                            index_count: indices.len() as u32,
                        }),
                        ResourceCreateDescriptor::Texture(TextureDescriptor {
                            width: 1,
                            height: 1,
                            format: TextureFormat::Rgba8Unorm,
                            data: vec![255, 255, 255, 255],
                            render_attachment: false,
                        }),
                    ])) {
                        Ok(resources) => resources,
                        Err(err) => {
                            eprintln!("cube mesh/texture creation failed: {err}");
                            return;
                        }
                    };
                let Some(mesh_id) = expect_mesh(resources.handles[0]) else {
                    eprintln!("expected mesh handle at resources[0]");
                    return;
                };
                let Some(texture_id) = expect_texture(resources.handles[1]) else {
                    eprintln!("expected texture handle at resources[1]");
                    return;
                };

                let material_resources =
                    match graphics_context.create_resources(ResourceBatchCreate::new(vec![
                        ResourceCreateDescriptor::Material(MaterialDescriptor {
                            vertex_shader_spv: include_bytes!(concat!(
                                env!("OUT_DIR"),
                                "/cube.vert.spv"
                            ))
                            .to_vec(),
                            vertex_entry: "main".to_string(),
                            fragment_shader_spv: include_bytes!(concat!(
                                env!("OUT_DIR"),
                                "/cube.frag.spv"
                            ))
                            .to_vec(),
                            fragment_entry: "main".to_string(),
                            enable_depth: true,
                            cull_mode: CullMode::Back,
                            texture: Some(texture_id),
                        }),
                    ])) {
                        Ok(resources) => resources,
                        Err(err) => {
                            eprintln!("cube material creation failed: {err}");
                            return;
                        }
                    };
                let Some(material_id) = expect_material(material_resources.handles[0]) else {
                    eprintln!("expected material handle at material_resources[0]");
                    return;
                };

                let scene =
                    SceneDescriptor::new(vec![MeshInstanceDescriptor::new(mesh_id, material_id)]);
                let commands = vec![RenderCommand::DrawGraphics(
                    PassDescriptor::new("main")
                        .with_clear_color([0.02, 0.02, 0.04, 1.0])
                        .with_clear_depth(Some(1.0)),
                )];

                self.graphics_context = Some(graphics_context);
                self.scene = Some(scene);
                self.commands = Some(commands);
                self.mesh_id = Some(mesh_id);
                self.base_positions = positions;
                self.colors = colors;
                self.indices = indices;
                self.elapsed_time = 0.0;
            }
            EngineEvent::Update(delta_time) => {
                self.elapsed_time += delta_time;
                if let (Some(graphics_context), Some(mesh_id)) =
                    (self.graphics_context.as_mut(), self.mesh_id)
                {
                    let vertices =
                        build_vertices(&self.base_positions, &self.colors, self.elapsed_time);
                    let update = ResourceBatchUpdate::new(vec![ResourceUpdateDescriptor::Mesh {
                        id: mesh_id,
                        vertex_data: vertices_as_bytes(&vertices),
                        vertex_layout: colored_vertex_layout(),
                        index_data: indices_as_bytes(&self.indices),
                        index_format: IndexFormat::Uint32,
                        index_count: self.indices.len() as u32,
                    }]);
                    if let Err(err) = graphics_context.update_resources(update) {
                        eprintln!("resource update failed: {err}");
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
            EngineEvent::Resized(width, height) => {
                if let Some(graphics_context) = self.graphics_context.as_mut() {
                    if let Err(err) = graphics_context.resize(Extent2D { width, height }) {
                        eprintln!("resize failed: {err}");
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
        title: "Rotex Frontend Cube".to_string(),
        width: 900,
        height: 700,
        ..Default::default()
    };
    let orchestrator = WindowOrchestrator::new(Box::new(WinitBackend));
    orchestrator.run(descriptor, Box::new(App::default()))?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("cube example is only available on native targets.");
}

#[cfg(not(target_arch = "wasm32"))]
fn cube_geometry() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let positions = vec![
        [-0.6, -0.6, -0.6],
        [0.6, -0.6, -0.6],
        [0.6, 0.6, -0.6],
        [-0.6, 0.6, -0.6],
        [-0.6, -0.6, 0.6],
        [0.6, -0.6, 0.6],
        [0.6, 0.6, 0.6],
        [-0.6, 0.6, 0.6],
    ];
    let colors = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 0.5, 0.0],
        [0.4, 0.2, 1.0],
    ];
    let indices = vec![
        // back (-Z)
        0, 2, 1, 2, 0, 3, // front (+Z)
        4, 5, 6, 6, 7, 4, // left (-X)
        0, 4, 7, 7, 3, 0, // right (+X)
        1, 6, 5, 6, 1, 2, // top (+Y)
        3, 6, 2, 6, 3, 7, // bottom (-Y)
        0, 1, 5, 5, 4, 0,
    ];
    (positions, colors, indices)
}

#[cfg(not(target_arch = "wasm32"))]
fn build_vertices(
    base_positions: &[[f32; 3]],
    colors: &[[f32; 3]],
    time: f32,
) -> Vec<ColoredVertex> {
    base_positions
        .iter()
        .zip(colors.iter())
        .map(|(position, color)| {
            let mut p = *position;
            rotate_vertex(&mut p, time);
            ColoredVertex {
                position: p,
                color: *color,
            }
        })
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn rotate_vertex(position: &mut [f32; 3], time: f32) {
    let angle_y = time;
    let angle_x = time * 0.7;
    let (sy, cy) = angle_y.sin_cos();
    let (sx, cx) = angle_x.sin_cos();

    let x = position[0] * cy + position[2] * sy;
    let z = -position[0] * sy + position[2] * cy;
    let y = position[1] * cx - z * sx;
    let z2 = position[1] * sx + z * cx;

    position[0] = x * 0.7;
    position[1] = y * 0.7;
    position[2] = (z2 * 0.45) + 0.5;
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_mesh(handle: ResourceHandle) -> Option<MeshId> {
    match handle {
        ResourceHandle::Mesh(id) => Some(id),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_material(handle: ResourceHandle) -> Option<MaterialId> {
    match handle {
        ResourceHandle::Material(id) => Some(id),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_texture(handle: ResourceHandle) -> Option<TextureId> {
    match handle {
        ResourceHandle::Texture(id) => Some(id),
        _ => None,
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
