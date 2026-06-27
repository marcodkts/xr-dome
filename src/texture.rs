use crate::settings::WorkstationVisualConfig;

pub struct Texture {
    _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn generated_background(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        horizontal_degrees: f32,
        vertical_degrees: f32,
    ) -> Self {
        let mut rgba = vec![0_u8; (width * height * 4) as usize];

        draw_guides(
            &mut rgba,
            width,
            height,
            horizontal_degrees,
            vertical_degrees,
            2.5,
        );

        Self::from_rgba(device, queue, width, height, &rgba)
    }

    pub fn generated_workstation(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        config: &WorkstationVisualConfig,
    ) -> Self {
        let mut rgba = vec![0_u8; (width * height * 4) as usize];

        {
            let mut canvas = Canvas::new(&mut rgba, width, height);
            canvas.draw_guides(
                config.horizontal_fov_degrees,
                config.vertical_fov_degrees,
                config.observer_distance_m,
            );
        }

        Self::from_rgba(device, queue, width, height, &rgba)
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

fn draw_guides(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    horizontal_degrees: f32,
    vertical_degrees: f32,
    observer_distance_m: f32,
) {
    let mut canvas = Canvas::new(rgba, width, height);
    canvas.draw_guides(horizontal_degrees, vertical_degrees, observer_distance_m);
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

    fn draw_guides(
        &mut self,
        horizontal_degrees: f32,
        vertical_degrees: f32,
        observer_distance_m: f32,
    ) {
        let cyan = [74, 223, 255, 255];
        let violet = [170, 106, 255, 255];
        let soft_white = [223, 236, 255, 255];

        let center_x = self.width as f32 * 0.5;
        let top_y = self.height as f32 * 0.16;
        let arc_y = self.height as f32 * 0.28;
        let left_x = self.width as f32 * 0.08;
        let right_x = self.width as f32 * 0.92;
        let inner_left_x = self.width as f32 * 0.34;
        let inner_right_x = self.width as f32 * 0.66;
        let marker_y = self.height as f32 * 0.58;
        let observer_y = self.height as f32 * 0.80;

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
        self.dashed_line(
            inner_left_x,
            self.height as f32 * 0.26,
            inner_left_x,
            self.height as f32 * 0.74,
            violet,
        );
        self.dashed_line(
            inner_right_x,
            self.height as f32 * 0.26,
            inner_right_x,
            self.height as f32 * 0.74,
            violet,
        );

        self.line(
            (self.width as f32 * 0.16).round() as i32,
            arc_y.round() as i32,
            (self.width as f32 * 0.84).round() as i32,
            arc_y.round() as i32,
            [42, 70, 102, 160],
        );

        self.line(
            center_x.round() as i32 - 18,
            observer_y.round() as i32,
            center_x.round() as i32 + 18,
            observer_y.round() as i32,
            cyan,
        );
        self.line(
            center_x.round() as i32,
            observer_y.round() as i32 - 18,
            center_x.round() as i32,
            observer_y.round() as i32 + 18,
            cyan,
        );

        self.line(
            center_x.round() as i32,
            observer_y.round() as i32,
            center_x.round() as i32,
            marker_y.round() as i32,
            [60, 112, 160, 160],
        );
        self.line(
            center_x.round() as i32,
            observer_y.round() as i32,
            (self.width as f32 * 0.80).round() as i32,
            (self.height as f32 * 0.72).round() as i32,
            [60, 112, 160, 120],
        );

        self.text_centered(
            &format!("~{horizontal_degrees:.0}° horizontal"),
            center_x,
            self.height as f32 * 0.06,
            5,
            soft_white,
        );
        self.text_centered(
            &format!("~{vertical_degrees:.0}° vertical"),
            self.width as f32 * 0.08,
            self.height as f32 * 0.46,
            4,
            cyan,
        );
        self.text_centered(
            "-70°",
            left_x + self.width as f32 * 0.01,
            self.height as f32 * 0.22,
            4,
            violet,
        );
        self.text_centered("-35°", inner_left_x, self.height as f32 * 0.24, 4, violet);
        self.text_centered("0°", center_x, self.height as f32 * 0.60, 4, soft_white);
        self.text_centered("+35°", inner_right_x, self.height as f32 * 0.24, 4, violet);
        self.text_centered(
            "+70°",
            right_x - self.width as f32 * 0.01,
            self.height as f32 * 0.22,
            4,
            violet,
        );
        self.text_centered(
            &format!("observer ~{observer_distance_m:.1} m"),
            center_x,
            observer_y + self.height as f32 * 0.05,
            4,
            soft_white,
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
        let glyph = match ch.to_ascii_uppercase() {
            'A' => [
                0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
            'B' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
            ],
            'C' => [
                0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
            ],
            'D' => [
                0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
            ],
            'E' => [
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ],
            'F' => [
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
            'G' => [
                0b01110, 0b10001, 0b10000, 0b10000, 0b10011, 0b10001, 0b01110,
            ],
            'H' => [
                0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
            'I' => [
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
            ],
            'J' => [
                0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
            ],
            'K' => [
                0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
            ],
            'L' => [
                0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
            ],
            'M' => [
                0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
            ],
            'N' => [
                0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
            ],
            'O' => [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'P' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
            'Q' => [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
            ],
            'R' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
            ],
            'S' => [
                0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
            ],
            'T' => [
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
            'U' => [
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'V' => [
                0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
            ],
            'W' => [
                0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
            ],
            'X' => [
                0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001,
            ],
            'Y' => [
                0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
            'Z' => [
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
            ],
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
            '/' => [
                0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
            ],
            ':' => [
                0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
            ],
            '.' => [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110,
            ],
            '(' => [
                0b00011, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00011,
            ],
            ')' => [
                0b11000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b11000,
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
