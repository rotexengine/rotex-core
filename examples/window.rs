use rotex_core::{
    DeviceDescriptor, Extent2D, GraphicsContext, InstanceDescriptor, SurfaceDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Window>,
    engine: Option<GraphicsContext>,
}

fn window_extent(window: &Window) -> Extent2D {
    let size = window.inner_size();
    Extent2D {
        width: size.width.max(1),
        height: size.height.max(1),
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Rotex Frontend Window")
            .with_inner_size(LogicalSize::new(800.0, 600.0));
        let window = event_loop.create_window(attrs).expect("window");
        let display_handle = window.display_handle().expect("display").as_raw();
        let window_handle = window.window_handle().expect("window").as_raw();
        let mut instance_descriptor = InstanceDescriptor::default();
        #[cfg(not(target_arch = "wasm32"))]
        {
            instance_descriptor.required_instance_extensions = ash_window::enumerate_required_extensions(
                display_handle,
            )
            .expect("required instance extensions")
            .iter()
            .map(|extension| unsafe { std::ffi::CStr::from_ptr(*extension) })
            .map(|extension| extension.to_string_lossy().into_owned())
            .collect();
        }
        let mut engine =
            pollster::block_on(GraphicsContext::new(instance_descriptor, DeviceDescriptor::default()))
                .expect("frontend engine");
        engine
            .attach_surface(SurfaceDescriptor {
                display_handle,
                window_handle,
                extent: window_extent(&window),
            })
            .expect("attach surface");
        self.window = Some(window);
        self.engine = Some(engine);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    if let Some(engine) = app.engine.take() {
        engine.destroy();
    }
    Ok(())
}
