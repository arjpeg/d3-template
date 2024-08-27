use std::sync::Arc;

use winit::{application::*, dpi::*, event::*, event_loop::*, window::*};

use anyhow::Result;

use crate::renderer::Renderer;

/// The load-state of the application, whether the window has been created yet or not.
#[derive(Default, Debug)]
pub enum AppLoadState<'a> {
    /// The window and app have been initialized and are ready for updates.
    Loaded {
        /// The target surface for rendering.
        window: Arc<Window>,
        /// The actual application.
        app: App<'a>,
    },
    /// The window hasn't been created yet.
    #[default]
    Unloaded,
}

/// The application state.
#[derive(Debug)]
pub struct App<'w> {
    /// The main renderer for the application.
    renderer: Renderer<'w>,
}
impl<'a> App<'a> {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let renderer = pollster::block_on(Renderer::new(window))?;

        Ok(Self { renderer })
    }

    pub fn render(&mut self) -> Result<()> {
        self.renderer.render()
    }

    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => self.renderer.resize(size),
            _ => {}
        }
    }
}

impl<'a> ApplicationHandler for AppLoadState<'a> {
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

                window.request_redraw();
            }

            _ => {
                app.handle_window_event(event);
            }
        }
    }
}
