use ash::vk;
use rotex_core::{
    Adapter, DebugMessenger, Device, DeviceDescriptor, Instance, QueueCategory, QueueRequest,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut instance, debug_messenger): (Instance, Option<DebugMessenger>) =
        Instance::new(&[]).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to initialize Vulkan instance: {err}"),
        )
    })?;

    let adapter: Adapter = instance
        .enumerate_adapters()
        .into_iter()
        .next()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "no Vulkan adapters found")
        })?;

    let descriptor = DeviceDescriptor {
        required_features: vk::PhysicalDeviceFeatures::default(),
        enable_swapchain: false,
        queues: vec![
            QueueRequest {
                category: QueueCategory::Compute,
                count: 1,
            },
            QueueRequest {
                category: QueueCategory::Transfer,
                count: 1,
            },
        ],
    };

    let mut device: Device = adapter.request_device(&instance, descriptor).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to initialize logical device: {err}"),
        )
    })?;

    println!("Headless device initialized on adapter: {}", adapter.name());

    device.destroy();
    if let Some(debug_messenger) = debug_messenger {
        debug_messenger.destroy();
    }
    instance.destroy();
    Ok(())
}
