#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::BufWriter;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
use rotex_core::{
    CullMode, DeviceDescriptor, GraphicsContext, IndexFormat, InstanceDescriptor,
    MaterialDescriptor, MaterialId, MeshDescriptor, MeshId, MeshInstanceDescriptor,
    PassColorTarget, PassDescriptor, RenderCommand, ResourceBatchCreate, ResourceCreateDescriptor,
    ResourceHandle, SceneDescriptor, TextureDescriptor, TextureFormat, TextureId, VertexAttribute,
    VertexBufferLayout, VertexFormat,
};
use rotex_vulkan::VulkanBridge;

#[cfg(not(target_arch = "wasm32"))]
const OUTPUT_WIDTH: u32 = 1024;
#[cfg(not(target_arch = "wasm32"))]
const OUTPUT_HEIGHT: u32 = 1024;

#[cfg(not(target_arch = "wasm32"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct ColoredVertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_path_from_args();
    let mut device_descriptor = DeviceDescriptor::default();
    device_descriptor.enable_swapchain = false;

    let backend = VulkanBridge::new(InstanceDescriptor::default(), device_descriptor)?;
    let mut graphics_context = GraphicsContext::new(Box::new(backend));

    let (positions, colors, indices) = cube_geometry();
    let vertices = build_vertices(&positions, &colors, 0.9);
    let resources = graphics_context.create_resources(ResourceBatchCreate::new(vec![
        ResourceCreateDescriptor::Mesh(MeshDescriptor {
            vertex_data: vertices_as_bytes(&vertices),
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
        ResourceCreateDescriptor::Texture(TextureDescriptor {
            width: OUTPUT_WIDTH,
            height: OUTPUT_HEIGHT,
            format: TextureFormat::Rgba8Unorm,
            data: vec![0; (OUTPUT_WIDTH as usize) * (OUTPUT_HEIGHT as usize) * 4],
            render_attachment: true,
        }),
    ]))?;
    let mesh_id = expect_mesh(resources_handle(&resources.handles, 0)?)?;
    let albedo_texture = expect_texture(resources_handle(&resources.handles, 1)?)?;
    let render_target = expect_texture(resources_handle(&resources.handles, 2)?)?;

    let material_resources = graphics_context.create_resources(ResourceBatchCreate::new(vec![
        ResourceCreateDescriptor::Material(MaterialDescriptor {
            vertex_shader_spv: include_bytes!(concat!(env!("OUT_DIR"), "/cube.vert.spv")).to_vec(),
            vertex_entry: "main".to_string(),
            fragment_shader_spv: include_bytes!(concat!(env!("OUT_DIR"), "/cube.frag.spv"))
                .to_vec(),
            fragment_entry: "main".to_string(),
            enable_depth: true,
            cull_mode: CullMode::Back,
            texture: Some(albedo_texture),
        }),
    ]))?;
    let material_id = expect_material(resources_handle(&material_resources.handles, 0)?)?;

    let scene = SceneDescriptor::new(vec![MeshInstanceDescriptor::new(mesh_id, material_id)]);
    let commands = vec![RenderCommand::DrawGraphics(
        PassDescriptor::new("offscreen")
            .with_color_target(PassColorTarget::Texture(render_target))
            .with_clear_color([0.03, 0.03, 0.05, 1.0])
            .with_clear_depth(Some(1.0)),
    )];

    graphics_context.render(&scene, &commands)?;
    let readback = graphics_context.read_texture(render_target)?;
    if !matches!(readback.format, TextureFormat::Rgba8Unorm) {
        return Err(format!(
            "cube_to_texture_png expects Rgba8Unorm output, got {:?}",
            readback.format
        )
        .into());
    }
    write_png(
        &output_path,
        readback.width,
        readback.height,
        &readback.data,
    )?;
    graphics_context.destroy();

    println!(
        "Cube rendered to {} ({}x{})",
        output_path.display(),
        OUTPUT_WIDTH,
        OUTPUT_HEIGHT
    );
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("cube_to_texture_png is only available on native targets.");
}

#[cfg(not(target_arch = "wasm32"))]
fn output_path_from_args() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("cube.png"))
}

#[cfg(not(target_arch = "wasm32"))]
fn resources_handle(
    handles: &[ResourceHandle],
    index: usize,
) -> Result<ResourceHandle, Box<dyn std::error::Error>> {
    handles.get(index).copied().ok_or_else(|| {
        format!("resource handle at index {index} was not returned by the backend").into()
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_mesh(handle: ResourceHandle) -> Result<MeshId, Box<dyn std::error::Error>> {
    match handle {
        ResourceHandle::Mesh(id) => Ok(id),
        other => Err(format!("expected mesh handle, got {other:?}").into()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_material(handle: ResourceHandle) -> Result<MaterialId, Box<dyn std::error::Error>> {
    match handle {
        ResourceHandle::Material(id) => Ok(id),
        other => Err(format!("expected material handle, got {other:?}").into()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_texture(handle: ResourceHandle) -> Result<TextureId, Box<dyn std::error::Error>> {
    match handle {
        ResourceHandle::Texture(id) => Ok(id),
        other => Err(format!("expected texture handle, got {other:?}").into()),
    }
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
        0, 2, 1, 2, 0, 3, 4, 5, 6, 6, 7, 4, 0, 4, 7, 7, 3, 0, 1, 6, 5, 6, 1, 2, 3, 6, 2, 6, 3, 7,
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

#[cfg(not(target_arch = "wasm32"))]
fn write_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header()?;
    png_writer.write_image_data(rgba_data)?;
    Ok(())
}
