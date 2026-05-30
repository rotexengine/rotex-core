use rotex_core::{DeviceDescriptor, FrontendEngine, InstanceDescriptor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device_descriptor = DeviceDescriptor::default();
    device_descriptor.enable_swapchain = false;
    let engine = pollster::block_on(FrontendEngine::new(
        InstanceDescriptor::default(),
        device_descriptor,
    ))?;
    println!("Frontend headless engine initialization succeeded.");
    engine.destroy();
    Ok(())
}
