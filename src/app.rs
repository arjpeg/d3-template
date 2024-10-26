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
#[derive(Debug)]
pub enum AppLoadState {
    /// The window and app have been initialized and are ready for updates.
    Loaded {
        /// The target surface for rendering.
        window: Arc<Window>,
        /// The actual application.
        app: App,
    },
    /// The window hasn't been created yet.
    Unloaded,
}

/// The application state.
#[derive(Debug)]
pub struct App {
    /// The main renderer for the application.
    renderer: Renderer,
    /// The first-person camera used as the origin for rendering.
    camera: Camera,
    /// The window onto which the app is rendered.
    window: Arc<Window>,

    /// The time of the last updated frame.
    last_frame: Instant,
    /// The time since, the last frame (always one frame behind). Also known as `dt`.
    delta_time: f32,

    /// Represents if the cursor is currently locked onto the window.
    has_focus: bool,

    /// A collection of the keys currently being held down.
    keys_down: HashSet<KeyCode>,
}

impl App {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let camera = Camera::new(vec3(0.0, 0.0, 3.0), -FRAC_PI_2, 0.0, window.inner_size());
        let renderer = pollster::block_on(Renderer::new(window.clone(), &camera))?;

        Ok(Self {
            renderer,
            camera,
            window,
            keys_down: HashSet::new(),
            last_frame: Instant::now(),
            delta_time: 0.0,
            has_focus: false,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        self.renderer.render()
    }

    pub fn update(&mut self) -> Result<()> {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();

        self.last_frame = now;
        self.delta_time = dt;

        if self.has_focus {
            self.camera.update_position(&self.keys_down, dt);
        }

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
                        eprintln!("unrecognized key code, {other:?}");
                        return;
                    }
                };

                if code == KeyCode::Escape {
                    self.has_focus = false;
                    self.set_cursor_state();
                    return;
                }

                match state {
                    ElementState::Pressed => self.keys_down.insert(code),
                    ElementState::Released => self.keys_down.remove(&code),
                };
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                self.has_focus = true;
                self.set_cursor_state()
            }

            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } if self.has_focus => {
                self.camera.update_rotation_angles(delta, self.delta_time)
            }

            _ => {}
        }
    }

    fn set_cursor_state(&mut self) {
        if self.has_focus {
            self.window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
            self.window.set_cursor_visible(false);
        } else {
            self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
            self.window.set_cursor_visible(true);
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
        let Self::Loaded { window, app } = self else {
            return;
        };

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

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        let Self::Loaded { app, .. } = self else {
            return;
        };

        app.handle_device_event(event);
    }
}
