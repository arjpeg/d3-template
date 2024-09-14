use std::{f32::consts::FRAC_PI_2, sync::Arc};

use winit::{application::*, dpi::*, event::*, event_loop::*, window::*};

use anyhow::Result;

use glam::*;

use crate::{camera::Camera, renderer::Renderer};

/// The load-state of the application, whether the window has been created yet or not.
#[derive(Default, Debug)]
pub enum AppLoadState {
    /// The window and app have been initialized and are ready for updates.
    Loaded {
        /// The target surface for rendering.
        window: Arc<Window>,
        /// The actual application.
        app: App,
    },
    /// The window hasn't been created yet.
    #[default]
    Unloaded,
}

/// The application state.
#[derive(Debug)]
pub struct App {
    /// The main renderer for the application.
    renderer: Renderer,
    /// The first-person camera used as the origin for rendering.
    camera: Camera,
}

impl App {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let camera = Camera::new(vec3(0.0, 0.0, 3.0), -FRAC_PI_2, 0.0, window.inner_size());
        let renderer = pollster::block_on(Renderer::new(window, &camera))?;

        Ok(Self { renderer, camera })
    }

    pub fn render(&mut self) -> Result<()> {
        self.renderer.render()
    }

    pub fn update(&mut self) -> Result<()> {
        self.renderer
            .update_camera_buffer(self.camera.view_projection());

        Ok(())
    }

    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.renderer.resize(size);
                self.camera.resize(size)
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for AppLoadState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("d3 template")
                    .with_inner_size(LogicalSize::new(1920, 1080)),
            )
            .expect("failed to create window");

        let window = Arc::new(window);

        window.request_redraw();

        *self = Self::Loaded {
            app: App::new(window.clone()).expect("failed to initialize app"),
            window,
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Self::Loaded { window, app } = self else { return; };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                app.render().unwrap();
                app.update().unwrap();

                window.request_redraw();
            }

            _ => {
                app.handle_window_event(event);
            }
        }
    }
}
