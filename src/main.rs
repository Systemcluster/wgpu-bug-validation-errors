use futures::executor::block_on;
use tracing::metadata::LevelFilter;
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;
use wgpu::{
    BackendBit, DeviceDescriptor, Features, Instance, Limits, PowerPreference,
    RequestAdapterOptions,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};


mod components;
mod graphics;
mod resources;
mod setup;
mod shaders;
mod systems;
mod universe;

use setup::*;
use universe::*;


fn main() {
    LogTracer::init().unwrap();
    let collector = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::default().add_directive(LevelFilter::WARN.into()))
        .compact()
        .finish();
    tracing::subscriber::set_global_default(collector).unwrap();

    let eventloop = EventLoop::new();
    let window = create_window(&eventloop);

    let backend = BackendBit::VULKAN;
    let instance = Instance::new(backend);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter_options = RequestAdapterOptions {
        power_preference:   PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    };
    let adapter = block_on(instance.request_adapter(&adapter_options)).unwrap();
    let device_limits = Limits {
        max_push_constant_size: 128,
        ..Limits::default()
    };
    let device_features = Features::default() | Features::PUSH_CONSTANTS;
    let device_descriptor = DeviceDescriptor {
        limits:   device_limits,
        features: device_features,
        label:    None,
    };
    let (device, queue) = block_on(
        adapter.request_device(&device_descriptor, Some(&std::path::Path::new("./trace"))),
    )
    .unwrap();

    window.set_visible(true);

    let mut universe = Universe::new(device, queue, surface);
    universe.resize(window.inner_size().width, window.inner_size().height);

    eventloop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                if *control_flow == ControlFlow::Exit {
                    return;
                }
                universe.render();
            }
            Event::RedrawRequested(_) => {}
            Event::WindowEvent {
                event: window_event,
                window_id,
            } => {
                if window_id == window.id() {
                    match window_event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {
                            universe.event(window_event);
                        }
                    }
                }
            }
            _ => {}
        }
    });
}
