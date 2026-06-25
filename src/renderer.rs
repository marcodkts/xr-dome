use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::{EulerRot, Mat4, Quat};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    window::Window,
};

use crate::{
    dome::Vertex,
    orientation::Orientation,
};

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

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = instance
            .create_surface(window)
            .expect("Não foi possível criar a superfície");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference:
                    wgpu::PowerPreference::HighPerformance,
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

        let vertex_buffer =
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Dome vertex buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );

        let index_buffer =
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Dome index buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                },
            );

        let camera_uniform = CameraUniform {
            view_projection: Mat4::IDENTITY.to_cols_array_2d(),
        };

        let camera_buffer =
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Camera buffer"),
                    contents: bytemuck::bytes_of(&camera_uniform),
                    usage: wgpu::BufferUsages::UNIFORM
                        | wgpu::BufferUsages::COPY_DST,
                },
            );

        let camera_bind_group_layout =
            device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility:
                            wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                },
            );

        let camera_bind_group =
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Camera bind group"),
                    layout: &camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource:
                            camera_buffer.as_entire_binding(),
                    }],
                },
            );

        let shader =
            device.create_shader_module(
                wgpu::ShaderModuleDescriptor {
                    label: Some("Dome shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("shader.wgsl").into(),
                    ),
                },
            );

        let pipeline_layout =
            device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Dome pipeline layout"),
                    bind_group_layouts: &[
                        &camera_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                },
            );

        let pipeline =
            device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Dome pipeline"),
                    layout: Some(&pipeline_layout),

                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vertex_main",
                        buffers: &[Vertex::descriptor()],
                    },

                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fragment_main",
                        targets: &[Some(
                            wgpu::ColorTargetState {
                                format,
                                blend: Some(
                                    wgpu::BlendState::REPLACE,
                                ),
                                write_mask:
                                    wgpu::ColorWrites::ALL,
                            },
                        )],
                    }),

                    primitive: wgpu::PrimitiveState {
                        topology:
                            wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode:
                            wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },

                    depth_stencil: None,
                    multisample:
                        wgpu::MultisampleState::default(),
                    multiview: None,
                },
            );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            camera_buffer,
            camera_bind_group,
        }
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

    fn update_camera(&self, orientation: Orientation) {
        let aspect =
            self.config.width as f32 / self.config.height as f32;

        let projection = Mat4::perspective_rh(
            60.0_f32.to_radians(),
            aspect,
            0.1,
            100.0,
        );

        let camera_rotation = Quat::from_euler(
            EulerRot::YXZ,
            orientation.yaw,
            orientation.pitch,
            orientation.roll,
        );

        let view = Mat4::from_quat(camera_rotation.conjugate());

        let camera_uniform = CameraUniform {
            view_projection:
                (projection * view).to_cols_array_2d(),
        };

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&camera_uniform),
        );
    }

    pub fn render(
        &mut self,
        orientation: Orientation,
    ) -> Result<(), wgpu::SurfaceError> {
        self.update_camera(orientation);

        let frame = self.surface.get_current_texture()?;

        let view = frame.texture.create_view(
            &wgpu::TextureViewDescriptor::default(),
        );

        let mut encoder =
            self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Render encoder"),
                },
            );

        {
            let mut render_pass =
                encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("Dome render pass"),

                        color_attachments: &[Some(
                            wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(
                                        wgpu::Color {
                                            r: 0.005,
                                            g: 0.008,
                                            b: 0.02,
                                            a: 1.0,
                                        },
                                    ),
                                    store:
                                        wgpu::StoreOp::Store,
                                },
                            },
                        )],

                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    },
                );

            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_bind_group(
                0,
                &self.camera_bind_group,
                &[],
            );

            render_pass.set_vertex_buffer(
                0,
                self.vertex_buffer.slice(..),
            );

            render_pass.set_index_buffer(
                self.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            render_pass.draw_indexed(
                0..self.index_count,
                0,
                0..1,
            );
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}