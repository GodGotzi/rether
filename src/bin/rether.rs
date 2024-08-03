use std::sync::Arc;

use wgpu::InstanceDescriptor;
use winit::{
    application::ApplicationHandler,
    error::{EventLoopError, OsError},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

fn main() -> Result<(), EventLoopError> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("Fiberslice-5D v{}", VERSION);

    let event_loop: EventLoop<()> = EventLoop::new().unwrap();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut application = Runner::Idle;

    event_loop.run_app(&mut application)
}

enum Runner {
    Idle,
    Running {
        window: Arc<winit::window::Window>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        adapter: wgpu::Adapter,

        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
        surface_format: wgpu::TextureFormat,
    },
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(create_window(event_loop).expect("Failed to create window"));

        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();

        // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        println!("Adapter: {:?}", adapter.get_info());

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits {
                    max_buffer_size: u32::MAX as u64,
                    ..Default::default()
                },
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .unwrap();

        let size = window.inner_size();
        let surface_format = surface.get_capabilities(&adapter).formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![surface_format],
        };
        surface.configure(&device, &surface_config);

        *self = Runner::Running {
            window,
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter,
            surface,
            surface_config,
            surface_format,
        };
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Runner::Running { window, .. } = self {
            match event {
                winit::event::WindowEvent::RedrawRequested => {}
                winit::event::WindowEvent::Resized(size) => {
                    self.resize_surface(size);
                }
                winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                    let size = window.inner_size();

                    self.resize_surface(size);
                }
                winit::event::WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                _ => {
                    window.request_redraw();
                }
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
    }
}

impl Runner {
    fn resize_surface(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            match self {
                Runner::Running {
                    device,
                    surface_config,
                    surface,
                    ..
                } => {
                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(&device, &surface_config);
                }
                _ => {}
            }
        }
    }
}

fn create_window(event_loop: &ActiveEventLoop) -> Result<Window, OsError> {
    let attributes = WindowAttributes::default()
        .with_title("Fiberslice-5D")
        .with_visible(false)
        .with_resizable(true)
        .with_decorations(true)
        .with_active(true);

    event_loop.create_window(attributes)
}
