use std::sync::Arc;

use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use anyhow::Result;

use crate::mesh::{Mesh, Vertex};

/// A wgpu-backend based renderer that holds a connection to the GPU, can create buffers, and render meshes.
#[derive(Debug)]
pub struct Renderer<'surface> {
    /// A handle to the rendering device, which in most cases will be a GPU.
    device: Device,
    /// A queue onto which messages can be passed to the `device` to be processed.
    queue: Queue,

    /// The pipeline through which data is transformed through the `device` to eventually be
    /// rendered onto the `surface`.
    pipeline: RenderPipeline,

    /// The surface onto which meshes will be rendered.
    surface: Surface<'surface>,
    /// The configuration of the `surface`.
    surface_config: SurfaceConfiguration,

    /// The mesh currently being rendered.
    mesh: Mesh,
}

impl<'surface> Renderer<'surface> {
    /// Creates a new open connection to the rendering device, and sets up a rendering pipeline.
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::empty(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: MemoryHints::Performance,
                },
                None,
            )
            .await?;

        let config = Self::create_surface_config(&surface, &adapter, window.inner_size());
        surface.configure(&device, &config);

        let pipeline = Self::create_render_pipeline(&device, config.format);

        let mesh = Mesh::new(
            &device,
            &[
                Vertex {
                    pos: [0.0, 0.5, 0.0],
                    color: [1.0, 0.0, 0.0],
                },
                Vertex {
                    pos: [-0.5, -0.5, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                Vertex {
                    pos: [0.5, -0.5, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
            ],
            &[0, 1, 2],
        );

        Ok(Self {
            device,
            queue,
            pipeline,
            surface,
            surface_config: config,
            mesh,
        })
    }

    /// Creates a configuration for a surface given the window size.
    fn create_surface_config(
        surface: &Surface,
        adapter: &Adapter,
        size: PhysicalSize<u32>,
    ) -> SurfaceConfiguration {
        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .iter()
            .cloned()
            .find(TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);

        let PhysicalSize { width, height } = size;

        SurfaceConfiguration {
            format,
            width,
            height,
            usage: TextureUsages::RENDER_ATTACHMENT,
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        }
    }

    /// Creates a render pipeline using the default shaders and settings.
    fn create_render_pipeline(device: &Device, texture_format: TextureFormat) -> RenderPipeline {
        let shader = device.create_shader_module(include_wgsl!("../assets/shader/main.wgsl"));

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipline Layout Descriptor"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    write_mask: ColorWrites::ALL,
                    blend: None,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState::default(),
            multisample: MultisampleState::default(),
            depth_stencil: None,
            multiview: None,
            cache: None,
        })
    }

    /// Reconfigures the target `surface` to the new rendering size.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = size;

        assert!(width > 0, "cannot resize to zero width");
        assert!(height > 0, "cannot resize to zero height");

        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Begins a render pass and renders the currently active meshes to the `surface`.
    pub fn render(&mut self) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint32);

            render_pass.draw(0..self.mesh.count, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
