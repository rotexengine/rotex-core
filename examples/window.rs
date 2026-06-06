#[cfg(not(target_arch = "wasm32"))]
use rotex_core::{
    DeviceDescriptor, Extent2D, GraphicsContext, InstanceDescriptor, SurfaceDescriptor,
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
                            eprintln!("frontend backend initialization failed: {err}");
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
                self.graphics_context = Some(graphics_context);
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
        title: "Rotex Frontend Window".to_string(),
        width: 800,
        height: 600,
        ..Default::default()
    };
    let orchestrator = WindowOrchestrator::new(Box::new(WinitBackend));
    orchestrator.run(descriptor, Box::new(App::default()))?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("window example is only available on native targets.");
}
