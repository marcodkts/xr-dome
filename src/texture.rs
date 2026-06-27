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
                    log::info!("loaded texture: {path}");
                    return texture;
                }

                Err(error) => {
                    log::warn!("could not load texture '{path}': {error}");
                    log::warn!("using generated fallback texture");
                }
            }
        }

        Self::generated_background(device, queue, 1024, 512, 140.0, 60.0)
    }

    pub fn generated_background(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        horizontal_degrees: f32,
        vertical_degrees: f32,
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

        draw_guides(
            &mut rgba,
            width,
            height,
            horizontal_degrees,
            vertical_degrees,
        );

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

fn draw_guides(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    horizontal_degrees: f32,
    vertical_degrees: f32,
) {
    let mut canvas = Canvas::new(rgba, width, height);
    canvas.draw_guides(horizontal_degrees, vertical_degrees);
}

struct Canvas<'a> {
    rgba: &'a mut [u8],
    width: u32,
    height: u32,
}

impl<'a> Canvas<'a> {
    fn new(rgba: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }

    fn draw_guides(&mut self, horizontal_degrees: f32, vertical_degrees: f32) {
        let cyan = [76, 235, 255, 255];
        let violet = [170, 106, 255, 255];
        let soft_white = [224, 238, 255, 255];

        let center_x = self.width as f32 * 0.5;
        let top_y = self.height as f32 * 0.16;
        let left_x = self.width as f32 * 0.10;
        let right_x = self.width as f32 * 0.90;

        self.dashed_arc(left_x, right_x, top_y, self.width as f32 * 0.024, cyan);
        self.dashed_line(
            left_x,
            self.height as f32 * 0.24,
            left_x,
            self.height as f32 * 0.78,
            cyan,
        );
        self.dashed_line(
            right_x,
            self.height as f32 * 0.24,
            right_x,
            self.height as f32 * 0.78,
            cyan,
        );

        self.text_centered(
            &format!("{horizontal_degrees:.0}°"),
            center_x,
            self.height as f32 * 0.06,
            5,
            soft_white,
        );

        self.text_centered(
            &format!("-{:.0}°", horizontal_degrees * 0.5),
            left_x + self.width as f32 * 0.02,
            self.height as f32 * 0.22,
            4,
            violet,
        );

        self.text_centered(
            &format!("+{:.0}°", horizontal_degrees * 0.5),
            right_x - self.width as f32 * 0.02,
            self.height as f32 * 0.22,
            4,
            violet,
        );

        self.text_centered(
            &format!("{vertical_degrees:.0}°"),
            left_x + self.width as f32 * 0.01,
            self.height as f32 * 0.50,
            4,
            cyan,
        );
    }

    fn dashed_arc(
        &mut self,
        left_x: f32,
        right_x: f32,
        anchor_y: f32,
        amplitude: f32,
        color: [u8; 4],
    ) {
        let steps = 220;
        let mut last_point: Option<(i32, i32)> = None;

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let x = mix(left_x, right_x, t);
            let curve = (t - 0.5).abs() * 2.0;
            let y = anchor_y - amplitude * (1.0 - curve * curve);

            let dash = ((t * 18.0).fract() < 0.62) || (step == 0) || (step == steps);

            let point = (x.round() as i32, y.round() as i32);

            if dash {
                if let Some((px, py)) = last_point {
                    self.line(px, py, point.0, point.1, color);
                }
                last_point = Some(point);
            } else {
                last_point = None;
            }
        }
    }

    fn dashed_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: [u8; 4]) {
        let steps = 80;
        let mut last_point: Option<(i32, i32)> = None;

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let dash = ((t * 12.0).fract() < 0.55) || (step == 0) || (step == steps);

            let x = mix(x0, x1, t);
            let y = mix(y0, y1, t);
            let point = (x.round() as i32, y.round() as i32);

            if dash {
                if let Some((px, py)) = last_point {
                    self.line(px, py, point.0, point.1, color);
                }
                last_point = Some(point);
            } else {
                last_point = None;
            }
        }
    }

    fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            self.set_pixel(x, y, color);

            if x == x1 && y == y1 {
                break;
            }

            let twice_err = err * 2;

            if twice_err >= dy {
                err += dy;
                x += sx;
            }

            if twice_err <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn text_centered(&mut self, text: &str, center_x: f32, top_y: f32, scale: i32, color: [u8; 4]) {
        let text_width = text.chars().count() as i32 * (6 * scale);
        let start_x = center_x.round() as i32 - text_width / 2;
        self.text(text, start_x, top_y.round() as i32, scale, color);
    }

    fn text(&mut self, text: &str, x: i32, y: i32, scale: i32, color: [u8; 4]) {
        let mut cursor_x = x;

        for ch in text.chars() {
            self.glyph(ch, cursor_x, y, scale, color);
            cursor_x += 6 * scale;
        }
    }

    fn glyph(&mut self, ch: char, x: i32, y: i32, scale: i32, color: [u8; 4]) {
        let glyph = match ch {
            '0' => [
                0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
            ],
            '1' => [
                0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
            '2' => [
                0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
            ],
            '3' => [
                0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
            ],
            '4' => [
                0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
            ],
            '5' => [
                0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
            ],
            '6' => [
                0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
            ],
            '7' => [
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
            ],
            '8' => [
                0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
            ],
            '9' => [
                0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b11100,
            ],
            '+' => [
                0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
            ],
            '-' => [
                0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
            ],
            '°' => [
                0b00110, 0b00110, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
            _ => [0, 0, 0, 0, 0, 0, 0],
        };

        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) == 0 {
                    continue;
                }

                for sy in 0..scale {
                    for sx in 0..scale {
                        self.set_pixel(
                            x + (col * scale) + sx,
                            y + (row as i32 * scale) + sy,
                            color,
                        );
                    }
                }
            }
        }
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 {
            return;
        }

        let x = x as u32;
        let y = y as u32;

        if x >= self.width || y >= self.height {
            return;
        }

        let index = ((y * self.width + x) * 4) as usize;

        let alpha = color[3] as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        self.rgba[index] = (self.rgba[index] as f32 * inv_alpha + color[0] as f32 * alpha) as u8;
        self.rgba[index + 1] =
            (self.rgba[index + 1] as f32 * inv_alpha + color[1] as f32 * alpha) as u8;
        self.rgba[index + 2] =
            (self.rgba[index + 2] as f32 * inv_alpha + color[2] as f32 * alpha) as u8;
        self.rgba[index + 3] = 255;
    }
}
