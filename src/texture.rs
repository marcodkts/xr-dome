use std::path::Path;

pub struct Texture {
    _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_path_or_generated(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: Option<&str>,
    ) -> Self {
        if let Some(path) = path {
            match Self::from_path(device, queue, path) {
                Ok(texture) => {
                    println!("loaded texture: {path}");
                    return texture;
                }

                Err(error) => {
                    eprintln!("could not load texture '{path}': {error}");
                    eprintln!("using generated fallback texture");
                }
            }
        }

        Self::generated_background(device, queue, 1024, 512)
    }

    pub fn generated_background(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Self {
        let mut rgba =
            vec![0_u8; (width * height * 4) as usize];

        for y in 0..height {
            let v = y as f32 / (height - 1) as f32;

            for x in 0..width {
                let u = x as f32 / (width - 1) as f32;

                /*
                * Coordenadas centradas.
                * u_center representa yaw.
                * v_center representa pitch visual.
                */
                let u_center = u - 0.5;
                let v_center = v - 0.5;

                /*
                * Base escura com leve foco no horizonte.
                */
                let horizon_fade =
                    1.0 - (v_center.abs() * 2.0).clamp(0.0, 1.0);

                let vignette =
                    (u_center.abs() * 1.15 + v_center.abs() * 0.9)
                        .clamp(0.0, 1.0);

                let mut r = 4.0 + 12.0 * horizon_fade;
                let mut g = 8.0 + 22.0 * horizon_fade;
                let mut b = 24.0 + 58.0 * horizon_fade;

                r *= 1.0 - vignette * 0.35;
                g *= 1.0 - vignette * 0.30;
                b *= 1.0 - vignette * 0.18;

                /*
                * Área central um pouco mais limpa,
                * para o painel não competir com a grid.
                */
                let center_clear =
                    1.0 - ((u_center / 0.23).powi(2)
                        + (v_center / 0.30).powi(2))
                        .sqrt()
                        .clamp(0.0, 1.0);

                r *= 1.0 - center_clear * 0.22;
                g *= 1.0 - center_clear * 0.20;
                b *= 1.0 - center_clear * 0.10;

                /*
                * Grid vertical: linhas de yaw.
                */
                let minor_vertical =
                    repeated_line(u, 64.0, 0.006);

                let major_vertical =
                    repeated_line(u, 16.0, 0.011);

                /*
                * Grid horizontal: linhas de pitch.
                * Ficam um pouco mais presentes abaixo do horizonte,
                * criando uma sensação de chão/mesa.
                */
                let lower_weight =
                    smoothstep(0.46, 0.82, v);

                let minor_horizontal =
                    repeated_line(v, 32.0, 0.006) * lower_weight;

                let major_horizontal =
                    repeated_line(v, 8.0, 0.014) * lower_weight;

                /*
                * Linhas diagonais sutis para dar estilo de cockpit/workspace.
                */
                let diagonal_a =
                    repeated_line(u + v * 0.35, 18.0, 0.005)
                        * lower_weight
                        * 0.45;

                let diagonal_b =
                    repeated_line(u - v * 0.35, 18.0, 0.005)
                        * lower_weight
                        * 0.35;

                /*
                * Horizonte e eixo frontal.
                */
                let horizon =
                    line_at(v, 0.5, 0.004);

                let front_axis =
                    wrapped_line_at(u, 0.5, 0.003);

                /*
                * Moldura angular sutil ao redor da área principal.
                */
                let focus_left =
                    wrapped_line_at(u, 0.5 - 0.165, 0.0025);

                let focus_right =
                    wrapped_line_at(u, 0.5 + 0.165, 0.0025);

                let focus_top =
                    line_at(v, 0.5 - 0.145, 0.0025);

                let focus_bottom =
                    line_at(v, 0.5 + 0.145, 0.0025);

                let focus_frame =
                    (focus_left + focus_right)
                        .min(1.0)
                        * smoothstep(0.34, 0.43, v)
                        * (1.0 - smoothstep(0.57, 0.66, v))
                        + (focus_top + focus_bottom)
                            .min(1.0)
                            * smoothstep(0.28, 0.42, u)
                            * (1.0 - smoothstep(0.58, 0.72, u));

                /*
                * Composição das linhas.
                */
                let minor_grid =
                    minor_vertical.max(minor_horizontal);

                let major_grid =
                    major_vertical.max(major_horizontal);

                let diagonal_grid =
                    diagonal_a.max(diagonal_b);

                r = mix(r, 28.0, minor_grid * 0.22);
                g = mix(g, 70.0, minor_grid * 0.28);
                b = mix(b, 110.0, minor_grid * 0.36);

                r = mix(r, 54.0, major_grid * 0.42);
                g = mix(g, 132.0, major_grid * 0.55);
                b = mix(b, 188.0, major_grid * 0.65);

                r = mix(r, 74.0, diagonal_grid * 0.28);
                g = mix(g, 82.0, diagonal_grid * 0.25);
                b = mix(b, 160.0, diagonal_grid * 0.45);

                r = mix(r, 44.0, horizon * 0.75);
                g = mix(g, 190.0, horizon * 0.85);
                b = mix(b, 230.0, horizon * 0.95);

                r = mix(r, 145.0, front_axis * 0.38);
                g = mix(g, 72.0, front_axis * 0.30);
                b = mix(b, 190.0, front_axis * 0.45);

                r = mix(r, 92.0, focus_frame * 0.38);
                g = mix(g, 170.0, focus_frame * 0.45);
                b = mix(b, 230.0, focus_frame * 0.55);

                /*
                * Pequeno glow no centro inferior.
                */
                let lower_center_glow =
                    1.0 - ((u_center / 0.32).powi(2)
                        + ((v - 0.68) / 0.22).powi(2))
                        .sqrt()
                        .clamp(0.0, 1.0);

                r = mix(r, 38.0, lower_center_glow * 0.20);
                g = mix(g, 92.0, lower_center_glow * 0.25);
                b = mix(b, 160.0, lower_center_glow * 0.32);

                let index = ((y * width + x) * 4) as usize;

                rgba[index] = r.clamp(0.0, 255.0) as u8;
                rgba[index + 1] = g.clamp(0.0, 255.0) as u8;
                rgba[index + 2] = b.clamp(0.0, 255.0) as u8;
                rgba[index + 3] = 255;
            }
        }

        Self::from_rgba(device, queue, width, height, &rgba)
    }

    fn from_path(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
    ) -> Result<Self, image::ImageError> {
        let image = image::open(path)?;
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();

        Ok(Self::from_rgba(device, queue, width, height, &rgba))
    }

    fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(
            &wgpu::TextureViewDescriptor::default(),
        );

        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                label: Some("Texture sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        );

        Self {
            _texture: texture,
            view,
            sampler,
        }
    }
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn line_at(value: f32, anchor: f32, width: f32) -> f32 {
    let d = (value - anchor).abs();
    1.0 - smoothstep(width, width * 1.8, d)
}

fn repeated_line(value: f32, divisions: f32, width: f32) -> f32 {
    let cell = (value * divisions).fract();
    let d = (cell - 0.5).abs();
    1.0 - smoothstep(width, width * 1.8, d)
}

fn wrapped_line_at(value: f32, anchor: f32, width: f32) -> f32 {
    let d0 = (value - anchor).abs();
    let d1 = (value - anchor + 1.0).abs();
    let d2 = (value - anchor - 1.0).abs();

    let d = d0.min(d1.min(d2));

    1.0 - smoothstep(width, width * 1.8, d)
}