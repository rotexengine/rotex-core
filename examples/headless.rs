use rotex_core::{DeviceDescriptor, GraphicsContext, InstanceDescriptor};
#[cfg(not(target_arch = "wasm32"))]
use rotex_vulkan::VulkanBridge;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device_descriptor = DeviceDescriptor::default();
    device_descriptor.enable_swapchain = false;
    let backend = VulkanBridge::new(InstanceDescriptor::default(), device_descriptor)?;
    let graphics_context = GraphicsContext::new(Box::new(backend));
    println!("Frontend headless graphics_context initialization succeeded.");
    graphics_context.destroy();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("headless example is only available on native targets.");
}
