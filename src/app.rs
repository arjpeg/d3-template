use std::{collections::HashSet, f32::consts::FRAC_PI_2, sync::Arc, time::Instant};

use winit::{
    application::*,
    dpi::*,
    event::*,
    event_loop::*,
    keyboard::{KeyCode, PhysicalKey},
    window::*,
};

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

    /// The time of the last frame updated, a.k.a delta tima.
    last_frame: Instant,

    /// A collection of the keys currently being held down.
    keys_down: HashSet<KeyCode>,
}

impl App {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let camera = Camera::new(vec3(0.0, 0.0, 3.0), -FRAC_PI_2, 0.0, window.inner_size());
        let renderer = pollster::block_on(Renderer::new(window, &camera))?;

        Ok(Self {
            renderer,
            camera,
            keys_down: HashSet::new(),
            last_frame: Instant::now(),
        })
    }

    pub fn render(&mut self) -> Result<()> {
        self.renderer.render()
    }

    pub fn update(&mut self) -> Result<()> {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();

        self.last_frame = now;

        self.camera.update_position(&self.keys_down, dt);

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

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                let code = match physical_key {
                    PhysicalKey::Code(code) => code,
                    PhysicalKey::Unidentified(other) => {
                        eprintln!("warning: unrecognized key code, {other:?}");
                        return;
                    }
                };

                match state {
                    ElementState::Pressed => self.keys_down.insert(code),
                    ElementState::Released => self.keys_down.remove(&code),
                };
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
