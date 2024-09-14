use glam::*;

use wgpu::{util::*, *};

use winit::{dpi::PhysicalSize, keyboard::KeyCode};

use std::collections::HashSet;

/// Represents a camera in 3D space.
#[derive(Debug, Clone)]
pub struct Camera {
    /// The position of the camera in the right handed coordinate system.
    pub eye: Vec3,
    /// The up vector of the camera (usually `Vec3::Y`, unless the roll angle is modified).
    pub up: Vec3,

    /// The euler-yaw angle of the camera in radians.
    pub yaw: f32,
    /// The euler-pitch angle of the camera in radians.
    pub pitch: f32,

    /// The aspect ratio of the rendering surface.
    aspect_ratio: f32,
}

/// Calculates the aspect ratio given a size.
fn calculate_aspect_ratio(size: PhysicalSize<u32>) -> f32 {
    let PhysicalSize { width, height } = size;
    width as f32 / height as f32
}

impl Camera {
    pub fn new(eye: Vec3, yaw: f32, pitch: f32, size: PhysicalSize<u32>) -> Self {
        let up = Vec3::Y;

        Self {
            eye,
            up,
            yaw,
            pitch,
            aspect_ratio: calculate_aspect_ratio(size),
        }
    }

    /// Returns the forward vector of the camera based on the `yaw` and `pitch`.
    pub fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }

    /// Returns the view-projection matrix of the camera.
    pub fn view_projection(&self) -> Mat4 {
        let forward = self.forward();

        let view = Mat4::look_at_rh(self.eye, forward + self.eye, self.up);
        let proj = Mat4::perspective_infinite_rh(45.0f32.to_radians(), self.aspect_ratio, 0.01);

        proj * view
    }

    /// Creates a new buffer, bind group layout, and bind group describing the camera's view-projection
    /// matrix.
    pub fn create_buffer(&self, device: &Device) -> (Buffer, BindGroupLayout, BindGroup) {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&self.view_projection().to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        (buffer, layout, bind_group)
    }

    /// Updates the aspect ratio of the camera given a new target size.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect_ratio = calculate_aspect_ratio(size);
    }

    /// Moves the camera in space based on which keys are currently being held down.
    /// Controls follow the default 'WASD' to move around the xz plane, `KeyCode::Space` to move up
    /// and `KeyCode::Shift` to move down.
    pub fn update_position(&mut self, keys_down: &HashSet<KeyCode>, dt: f32) {
        let mut delta = Vec3::ZERO;

        let forward = self.forward();
        let right = forward.cross(self.up);

        if keys_down.contains(&KeyCode::KeyW) {
            delta += forward;
        }

        if keys_down.contains(&KeyCode::KeyS) {
            delta -= forward;
        }

        if keys_down.contains(&KeyCode::KeyD) {
            delta += right;
        }
        if keys_down.contains(&KeyCode::KeyA) {
            delta -= right;
        }

        if keys_down.contains(&KeyCode::Space) {
            delta += self.up;
        }
        if keys_down.contains(&KeyCode::ShiftLeft) {
            delta -= self.up;
        }

        self.eye += 5.0 * delta.normalize_or_zero() * dt;
    }
}
