use rotex_core::{DeviceDescriptor, GraphicsContext, InstanceDescriptor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device_descriptor = DeviceDescriptor::default();
    device_descriptor.enable_swapchain = false;
    let graphics_context = pollster::block_on(GraphicsContext::new(
        InstanceDescriptor::default(),
        device_descriptor,
    ))?;
    println!("Frontend headless graphics_context initialization succeeded.");
    graphics_context.destroy();
    Ok(())
}
