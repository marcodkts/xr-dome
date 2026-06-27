use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::{EulerRot, Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::Window,
};

use crate::{
    dome::Vertex, dome_config::DomeConfig, orientation::Orientation, ray::Ray, texture::Texture,
};

const CAMERA_VERTICAL_FOV_DEGREES: f32 = 60.0;
const CURSOR_VERTEX_CAPACITY: usize = 8;
const CURSOR_INDEX_CAPACITY: usize = 12;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,

    pipeline: wgpu::RenderPipeline,

    dome_vertex_buffer: wgpu::Buffer,
    dome_index_buffer: wgpu::Buffer,
    dome_index_count: u32,

    surface_vertex_buffer: wgpu::Buffer,
    surface_index_buffer: wgpu::Buffer,
    surface_index_count: u32,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    _dome_texture: Texture,
    _surface_texture: Texture,

    dome_texture_bind_group: wgpu::BindGroup,
    surface_texture_bind_group: wgpu::BindGroup,

    cursor_vertex_buffer: wgpu::Buffer,
    cursor_index_buffer: wgpu::Buffer,
    cursor_index_count: u32,

    _cursor_texture: Texture,
    cursor_texture_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        dome_vertices: &[Vertex],
        dome_indices: &[u32],
        surface_vertices: &[Vertex],
        surface_indices: &[u32],
        dome_config: &DomeConfig,
        surface_texture_path: Option<&str>,
    ) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = instance
            .create_surface(window)
            .expect("Não foi possível criar a superfície");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Nenhuma GPU compatível foi encontrada");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("XR Dome Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Não foi possível criar o dispositivo gráfico");

        let capabilities = surface.get_capabilities(&adapter);

        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let dome_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dome vertex buffer"),
            contents: bytemuck::cast_slice(dome_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let dome_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dome index buffer"),
            contents: bytemuck::cast_slice(dome_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let surface_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("surface vertex buffer"),
            contents: bytemuck::cast_slice(surface_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let surface_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("surface index buffer"),
            contents: bytemuck::cast_slice(surface_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let cursor_dummy_vertices = [Vertex {
            position: [0.0, 0.0, 0.0],
            uv: [0.0, 0.0],
        }; CURSOR_VERTEX_CAPACITY];

        let cursor_dummy_indices = [0_u32; CURSOR_INDEX_CAPACITY];

        let cursor_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cursor vertex buffer"),
            contents: bytemuck::cast_slice(&cursor_dummy_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let cursor_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cursor index buffer"),
            contents: bytemuck::cast_slice(&cursor_dummy_indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let camera_uniform = CameraUniform {
            view_projection: Mat4::IDENTITY.to_cols_array_2d(),
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::bytes_of(&camera_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let dome_texture = Texture::generated_background(
            &device,
            &queue,
            2048,
            1024,
            dome_config.yaw_degrees,
            dome_config.max_pitch_degrees - dome_config.min_pitch_degrees,
        );

        let surface_texture =
            Texture::from_path_or_generated(&device, &queue, surface_texture_path);

        let cursor_texture = Texture::solid_rgba(&device, &queue, [230, 250, 255, 255]);

        let dome_texture_bind_group = Self::create_texture_bind_group(
            &device,
            &texture_bind_group_layout,
            &dome_texture,
            "Dome texture bind group",
        );

        let surface_texture_bind_group = Self::create_texture_bind_group(
            &device,
            &texture_bind_group_layout,
            &surface_texture,
            "surface texture bind group",
        );

        let cursor_texture_bind_group = Self::create_texture_bind_group(
            &device,
            &texture_bind_group_layout,
            &cursor_texture,
            "Cursor texture bind group",
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("XR Dome shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("XR Dome pipeline layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("XR Dome pipeline"),
            layout: Some(&pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &[Vertex::descriptor()],
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },

            depth_stencil: None,

            multisample: wgpu::MultisampleState::default(),

            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,

            pipeline,

            dome_vertex_buffer,
            dome_index_buffer,
            dome_index_count: dome_indices.len() as u32,

            surface_vertex_buffer,
            surface_index_buffer,
            surface_index_count: surface_indices.len() as u32,

            cursor_vertex_buffer,
            cursor_index_buffer,
            cursor_index_count: 0,

            camera_buffer,
            camera_bind_group,

            _dome_texture: dome_texture,
            _surface_texture: surface_texture,
            _cursor_texture: cursor_texture,

            dome_texture_bind_group,
            surface_texture_bind_group,
            cursor_texture_bind_group,
        }
    }

    fn create_texture_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        texture: &Texture,
        label: &str,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;

        self.reconfigure();
    }

    pub fn reconfigure(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn screen_ray(
        &self,
        cursor_position: PhysicalPosition<f64>,
        orientation: Orientation,
        camera_position: Vec3,
    ) -> Option<Ray> {
        if self.config.width == 0 || self.config.height == 0 {
            return None;
        }

        let width = self.config.width as f32;
        let height = self.config.height as f32;

        let x = cursor_position.x as f32;
        let y = cursor_position.y as f32;

        if x < 0.0 || y < 0.0 || x > width || y > height {
            return None;
        }

        let ndc_x = (x / width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / height) * 2.0;

        let aspect = width / height;

        let tan_half_fov = (CAMERA_VERTICAL_FOV_DEGREES.to_radians() * 0.5).tan();

        /*
         * Direção em espaço de câmera.
         * A câmera olha para -Z.
         */

        let camera_direction =
            Vec3::new(ndc_x * aspect * tan_half_fov, ndc_y * tan_half_fov, -1.0).normalize();

        let camera_rotation = Quat::from_euler(
            EulerRot::YXZ,
            orientation.yaw,
            orientation.pitch,
            orientation.roll,
        );

        let world_direction = camera_rotation * camera_direction;

        Some(Ray::new(camera_position, world_direction))
    }

    pub fn screen_center_ray(
        &self,
        orientation: Orientation,
        camera_position: Vec3,
    ) -> Option<Ray> {
        let center = PhysicalPosition::new(
            self.config.width as f64 * 0.5,
            self.config.height as f64 * 0.5,
        );

        self.screen_ray(center, orientation, camera_position)
    }

    fn update_camera(&self, orientation: Orientation, position: Vec3) {
        let aspect = self.config.width as f32 / self.config.height as f32;

        let projection =
            Mat4::perspective_rh(CAMERA_VERTICAL_FOV_DEGREES.to_radians(), aspect, 0.1, 100.0);

        let camera_rotation = Quat::from_euler(
            EulerRot::YXZ,
            orientation.yaw,
            orientation.pitch,
            orientation.roll,
        );

        let camera_transform = Mat4::from_rotation_translation(camera_rotation, position);

        let view = camera_transform.inverse();

        let camera_uniform = CameraUniform {
            view_projection: (projection * view).to_cols_array_2d(),
        };

        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera_uniform));
    }

    pub fn render(
        &mut self,
        orientation: Orientation,
        position: Vec3,
    ) -> Result<(), wgpu::SurfaceError> {
        self.update_camera(orientation, position);

        let frame = self.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("XR Dome render pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.005,
                            g: 0.008,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],

                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // 1. Desenha o domo/fundo
            render_pass.set_bind_group(1, &self.dome_texture_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.dome_vertex_buffer.slice(..));

            render_pass
                .set_index_buffer(self.dome_index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..self.dome_index_count, 0, 0..1);

            // 2. Desenha o painel central por cima do domo
            render_pass.set_bind_group(1, &self.surface_texture_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.surface_vertex_buffer.slice(..));

            render_pass.set_index_buffer(
                self.surface_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            render_pass.draw_indexed(0..self.surface_index_count, 0, 0..1);

            if self.cursor_index_count > 0 {
                render_pass.set_bind_group(1, &self.cursor_texture_bind_group, &[]);

                render_pass.set_vertex_buffer(0, self.cursor_vertex_buffer.slice(..));

                render_pass.set_index_buffer(
                    self.cursor_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(0..self.cursor_index_count, 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn update_cursor_mesh(&mut self, vertices: &[Vertex], indices: &[u32]) {
        if vertices.is_empty() || indices.is_empty() {
            self.cursor_index_count = 0;
            return;
        }

        debug_assert!(vertices.len() <= CURSOR_VERTEX_CAPACITY);
        debug_assert!(indices.len() <= CURSOR_INDEX_CAPACITY);

        self.queue.write_buffer(
            &self.cursor_vertex_buffer,
            0,
            bytemuck::cast_slice(vertices),
        );
        self.queue
            .write_buffer(&self.cursor_index_buffer, 0, bytemuck::cast_slice(indices));

        self.cursor_index_count = indices.len() as u32;
    }

    pub fn clear_cursor_mesh(&mut self) {
        self.cursor_index_count = 0;
    }
}
