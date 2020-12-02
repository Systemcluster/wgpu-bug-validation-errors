use winit::{
    dpi::LogicalSize,
    window::{Window, WindowBuilder},
};


pub fn create_window(eventloop: &winit::event_loop::EventLoop<()>) -> Window {
    let builder = WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(1024, 1024))
        .with_min_inner_size(LogicalSize::new(1024, 1024))
        .with_title(env!("CARGO_PKG_NAME"))
        .with_transparent(false)
        .with_decorations(true);
    builder.build(&eventloop).unwrap()
}
