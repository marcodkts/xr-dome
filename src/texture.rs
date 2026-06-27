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
        let mut rgba = vec![0_u8; (width * height * 4) as usize];

        for y in 0..height {
            let v = y as f32 / (height - 1) as f32;

            for x in 0..width {
                let u = x as f32 / (width - 1) as f32;

                let v_center = v - 0.5;

                /*
                 * Base quase totalmente preta.
                 * Pequeníssima variação só para não ficar "chapado".
                 */
                let horizon_fade = 1.0 - (v_center.abs() * 2.0).clamp(0.0, 1.0);

                let mut r = 1.5 + 2.0 * horizon_fade;
                let mut g = 1.5 + 2.5 * horizon_fade;
                let mut b = 2.0 + 4.0 * horizon_fade;

                /*
                 * Grade fina e discreta.
                 * Menos linhas e menos intensidade.
                 */
                let minor_vertical = repeated_line(u, 48.0, 0.0045);

                let major_vertical = repeated_line(u, 12.0, 0.0075);

                let minor_horizontal = repeated_line(v, 24.0, 0.0045);

                let major_horizontal = repeated_line(v, 6.0, 0.0075);

                let minor_grid = minor_vertical.max(minor_horizontal);

                let major_grid = major_vertical.max(major_horizontal);

                /*
                 * Horizonte sutil para orientação.
                 */
                let horizon = line_at(v, 0.5, 0.0035);

                /*
                 * Eixo frontal bem discreto.
                 */
                let front_axis = wrapped_line_at(u, 0.5, 0.0025);

                /*
                 * Mistura extremamente contida.
                 * A ideia é parecer uma malha técnica fina,
                 * não uma interface brilhante.
                 */
                r = mix(r, 12.0, minor_grid * 0.14);
                g = mix(g, 18.0, minor_grid * 0.16);
                b = mix(b, 26.0, minor_grid * 0.18);

                r = mix(r, 18.0, major_grid * 0.24);
                g = mix(g, 28.0, major_grid * 0.28);
                b = mix(b, 42.0, major_grid * 0.32);

                r = mix(r, 18.0, horizon * 0.25);
                g = mix(g, 42.0, horizon * 0.35);
                b = mix(b, 60.0, horizon * 0.45);

                r = mix(r, 24.0, front_axis * 0.16);
                g = mix(g, 20.0, front_axis * 0.14);
                b = mix(b, 34.0, front_axis * 0.18);

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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            _texture: texture,
            view,
            sampler,
        }
    }

    pub fn solid_rgba(device: &wgpu::Device, queue: &wgpu::Queue, rgba: [u8; 4]) -> Self {
        Self::from_rgba(device, queue, 1, 1, &rgba)
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
