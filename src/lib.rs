mod app;
mod camera;
mod mesh;
mod renderer;

use winit::event_loop::*;

use crate::app::AppLoadState;

/// Entry point into the application, sets up an event loop and runs the app.
pub fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AppLoadState::Unloaded;
    event_loop.run_app(&mut app).unwrap();
}
