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

    pub fn generated_workstation(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        config: &WorkstationVisualConfig,
    ) -> Self {
        let mut rgba = vec![0_u8; (width * height * 4) as usize];

        for y in 0..height {
            let v = y as f32 / (height - 1) as f32;
            let vertical_glow = 1.0 - ((v - 0.46).abs() * 2.0).clamp(0.0, 1.0);

            for x in 0..width {
                let u = x as f32 / (width - 1) as f32;
                let center_distance = ((u - 0.5).abs() * 2.0).clamp(0.0, 1.0);

                let vignette = 1.0 - (center_distance.powf(1.7) * 0.72 + (v - 0.5).abs() * 0.92);
                let base = vignette.clamp(0.0, 1.0);
                let panel_glow =
                    (vertical_glow * 0.55 + (1.0 - center_distance) * 0.25).clamp(0.0, 1.0);

                let mut r = 2.0 + 10.0 * base + 14.0 * panel_glow;
                let mut g = 4.0 + 15.0 * base + 18.0 * panel_glow;
                let mut b = 8.0 + 26.0 * base + 32.0 * panel_glow;

                let grid_x = repeated_line(u, 14.0, 0.0035);
                let grid_y = repeated_line(v, 10.0, 0.0035);
                let grid = grid_x.max(grid_y) * 0.16;

                r += 12.0 * grid;
                g += 18.0 * grid;
                b += 28.0 * grid;

                let index = ((y * width + x) * 4) as usize;
                rgba[index] = r.clamp(0.0, 255.0) as u8;
                rgba[index + 1] = g.clamp(0.0, 255.0) as u8;
                rgba[index + 2] = b.clamp(0.0, 255.0) as u8;
                rgba[index + 3] = 255;
            }
        }

        {
            let mut canvas = Canvas::new(&mut rgba, width, height);
            canvas.draw_workstation_scene(config);
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

    fn draw_workstation_scene(&mut self, config: &WorkstationVisualConfig) {
        let cyan = [55, 216, 255, 255];
        let cyan_soft = [108, 232, 255, 180];
        let violet = [171, 108, 255, 255];
        let white = [232, 242, 255, 255];
        let panel_fill = [9, 16, 30, 220];
        let panel_alt = [11, 22, 40, 230];
        let panel_border = [91, 176, 255, 170];
        let panel_border_soft = [153, 110, 255, 120];

        self.text_centered(&config.title, self.width as f32 * 0.5, 16.0, 8, white);
        self.text_centered(&config.subtitle, self.width as f32 * 0.5, 78.0, 4, cyan);

        self.line(
            (self.width as f32 * 0.18).round() as i32,
            (self.height as f32 * 0.15).round() as i32,
            (self.width as f32 * 0.82).round() as i32,
            (self.height as f32 * 0.15).round() as i32,
            [46, 75, 106, 180],
        );

        let arc_left = self.width as f32 * 0.06;
        let arc_right = self.width as f32 * 0.94;
        let arc_y = self.height as f32 * 0.30;
        self.dashed_arc(arc_left, arc_right, arc_y, self.height as f32 * 0.085, cyan);

        self.text_centered(
            &format!("~{:.0} DEG HORIZONTAL", config.horizontal_fov_degrees),
            self.width as f32 * 0.5,
            self.height as f32 * 0.20,
            6,
            cyan,
        );

        self.text_centered(
            &format!("~{:.0} DEG VERTICAL", config.vertical_fov_degrees),
            self.width as f32 * 0.09,
            self.height as f32 * 0.47,
            4,
            cyan,
        );

        self.text_centered(
            &format!("RAIO VIRTUAL: ~{:.1} M", config.observer_distance_m),
            self.width as f32 * 0.79,
            self.height as f32 * 0.80,
            4,
            cyan,
        );

        let left_support = (
            (self.width as f32 * 0.08).round() as i32,
            (self.height as f32 * 0.28).round() as i32,
            (self.width as f32 * 0.27).round() as i32,
            (self.height as f32 * 0.68).round() as i32,
        );

        let main_panel = (
            (self.width as f32 * 0.31).round() as i32,
            (self.height as f32 * 0.24).round() as i32,
            (self.width as f32 * 0.69).round() as i32,
            (self.height as f32 * 0.68).round() as i32,
        );

        let right_support = (
            (self.width as f32 * 0.73).round() as i32,
            (self.height as f32 * 0.28).round() as i32,
            (self.width as f32 * 0.92).round() as i32,
            (self.height as f32 * 0.68).round() as i32,
        );

        self.draw_glass_card(
            left_support.0,
            left_support.1,
            left_support.2,
            left_support.3,
            panel_fill,
            panel_border_soft,
        );
        self.draw_glass_card(
            main_panel.0,
            main_panel.1,
            main_panel.2,
            main_panel.3,
            panel_alt,
            panel_border,
        );
        self.draw_glass_card(
            right_support.0,
            right_support.1,
            right_support.2,
            right_support.3,
            panel_fill,
            panel_border_soft,
        );

        self.text(
            "SUPPORT",
            left_support.0 + 18,
            left_support.1 + 18,
            4,
            violet,
        );
        self.text("WORKSPACE", main_panel.0 + 18, main_panel.1 + 18, 4, white);
        self.text(
            "SUPPORT",
            right_support.0 + 18,
            right_support.1 + 18,
            4,
            violet,
        );

        let docs_card = (
            left_support.0 + 18,
            left_support.1 + 70,
            left_support.2 - 18,
            left_support.1 + 150,
        );
        let chat_card = (
            left_support.0 + 18,
            left_support.1 + 170,
            left_support.2 - 18,
            left_support.1 + 250,
        );
        self.draw_feature_card(docs_card, "DOCS", "REFERENCE", cyan, panel_fill);
        self.draw_feature_card(chat_card, "CHAT", "MESSAGES", violet, panel_fill);

        let editor_card = (
            main_panel.0 + 18,
            main_panel.1 + 56,
            main_panel.0 + 132,
            main_panel.3 - 18,
        );
        let desktop_card = (
            main_panel.0 + 144,
            main_panel.1 + 56,
            main_panel.2 - 144,
            main_panel.3 - 18,
        );
        let terminal_card = (
            main_panel.2 - 130,
            main_panel.1 + 56,
            main_panel.2 - 18,
            main_panel.3 - 18,
        );
        self.draw_editor_card(editor_card, cyan);
        self.draw_dashboard_card(desktop_card, white, cyan_soft);
        self.draw_terminal_card(terminal_card, violet);

        let logs_card = (
            right_support.0 + 18,
            right_support.1 + 70,
            right_support.2 - 18,
            right_support.1 + 150,
        );
        let monitor_card = (
            right_support.0 + 18,
            right_support.1 + 170,
            right_support.2 - 18,
            right_support.1 + 250,
        );
        self.draw_feature_card(logs_card, "LOGS", "SYSTEM EVENTS", violet, panel_fill);
        self.draw_monitor_card(monitor_card, cyan);

        let legend_x = (self.width as f32 * 0.03).round() as i32;
        let legend_y = (self.height as f32 * 0.78).round() as i32;
        self.draw_glass_card(
            legend_x,
            legend_y,
            legend_x + 420,
            legend_y + 170,
            [7, 12, 24, 210],
            [82, 144, 226, 130],
        );
        self.text("LEGEND", legend_x + 18, legend_y + 18, 4, white);
        self.text("CENTRAL", legend_x + 18, legend_y + 56, 3, cyan);
        self.text("FOCUS AREA", legend_x + 18, legend_y + 80, 3, white);
        self.text("SUPPORT", legend_x + 18, legend_y + 110, 3, violet);
        self.text("CONTEXT AND TOOLS", legend_x + 18, legend_y + 134, 3, white);

        let note_x = (self.width as f32 * 0.73).round() as i32;
        let note_y = (self.height as f32 * 0.79).round() as i32;
        self.draw_glass_card(
            note_x,
            note_y,
            note_x + 340,
            note_y + 150,
            [7, 12, 24, 200],
            [87, 206, 255, 110],
        );
        self.text("TARGET", note_x + 18, note_y + 18, 4, cyan);
        self.text(
            "DESKTOP FIRST, REAL WORK",
            note_x + 18,
            note_y + 52,
            3,
            white,
        );
        self.text(
            "LAYOUT CUSTOMIZABLE VIA ENV",
            note_x + 18,
            note_y + 76,
            3,
            white,
        );
        self.text(
            "XR DOME BECOMES THE WORKSTATION",
            note_x + 18,
            note_y + 104,
            3,
            violet,
        );
    }

    fn draw_glass_card(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        fill: [u8; 4],
        border: [u8; 4],
    ) {
        self.fill_rect(x0, y0, x1, y1, fill);
        self.outline_rect(x0, y0, x1, y1, border);
        self.outline_rect(x0 + 1, y0 + 1, x1 - 1, y1 - 1, [255, 255, 255, 28]);
    }

    fn draw_feature_card(
        &mut self,
        bounds: (i32, i32, i32, i32),
        title: &str,
        subtitle: &str,
        accent: [u8; 4],
        fill: [u8; 4],
    ) {
        let (x0, y0, x1, y1) = bounds;
        self.draw_glass_card(x0, y0, x1, y1, [fill[0], fill[1], fill[2], 200], accent);
        self.text(title, x0 + 14, y0 + 14, 4, accent);
        self.text(subtitle, x0 + 14, y0 + 38, 2, [235, 242, 255, 220]);
        self.fill_rect(x0 + 14, y1 - 18, x0 + 68, y1 - 10, accent);
    }

    fn draw_editor_card(&mut self, bounds: (i32, i32, i32, i32), accent: [u8; 4]) {
        let (x0, y0, x1, y1) = bounds;
        self.draw_glass_card(x0, y0, x1, y1, [6, 12, 24, 225], accent);
        self.text("EDITOR", x0 + 12, y0 + 12, 3, accent);

        let line_start = y0 + 38;
        let colors = [
            [109, 215, 255, 255],
            [181, 110, 255, 255],
            [122, 255, 179, 255],
            [255, 137, 112, 255],
        ];

        for i in 0..12 {
            let y = line_start + i * 11;
            let width = 24 + (i % 4) * 12;
            self.fill_rect(
                x0 + 10,
                y,
                x0 + width,
                y + 3,
                colors[(i as usize) % colors.len()],
            );
            self.fill_rect(x0 + width + 8, y, x1 - 12, y + 2, [46, 66, 86, 120]);
        }
    }

    fn draw_dashboard_card(
        &mut self,
        bounds: (i32, i32, i32, i32),
        title_color: [u8; 4],
        accent: [u8; 4],
    ) {
        let (x0, y0, x1, y1) = bounds;
        self.draw_glass_card(x0, y0, x1, y1, [8, 16, 30, 225], accent);
        self.text("DESKTOP", x0 + 12, y0 + 12, 3, title_color);

        let sidebar_x = x0 + 12;
        for i in 0..5 {
            let y = y0 + 40 + i * 24;
            let active = i == 1;
            self.fill_rect(
                sidebar_x,
                y,
                sidebar_x + 18,
                y + 18,
                if active { accent } else { [47, 67, 95, 200] },
            );
        }

        self.fill_rect(x0 + 48, y0 + 42, x1 - 12, y0 + 96, [13, 25, 47, 220]);
        self.fill_rect(x0 + 48, y0 + 108, x1 - 12, y0 + 158, [13, 25, 47, 220]);

        let chart_base_y = y0 + 74;
        let chart_points = [
            (x0 + 54, chart_base_y + 18),
            (x0 + 74, chart_base_y + 12),
            (x0 + 96, chart_base_y + 22),
            (x0 + 118, chart_base_y + 8),
            (x0 + 140, chart_base_y + 16),
            (x0 + 164, chart_base_y + 6),
        ];
        for pair in chart_points.windows(2) {
            let a = pair[0];
            let b = pair[1];
            self.line(a.0, a.1, b.0, b.1, accent);
        }

        self.fill_rect(x0 + 54, y0 + 128, x0 + 84, y0 + 152, accent);
        self.fill_rect(x0 + 92, y0 + 132, x0 + 122, y0 + 152, [95, 120, 160, 200]);
        self.fill_rect(x0 + 130, y0 + 124, x0 + 160, y0 + 152, [65, 190, 255, 180]);
        self.fill_rect(x0 + 168, y0 + 136, x0 + 198, y0 + 152, [180, 100, 255, 180]);

        self.text("FILES", x0 + 50, y1 - 26, 2, title_color);
    }

    fn draw_terminal_card(&mut self, bounds: (i32, i32, i32, i32), accent: [u8; 4]) {
        let (x0, y0, x1, y1) = bounds;
        self.draw_glass_card(x0, y0, x1, y1, [7, 12, 21, 225], accent);
        self.text("TERMINAL", x0 + 12, y0 + 12, 3, accent);

        let mut y = y0 + 40;
        let lines = [
            ("BUILD", accent),
            ("SYNC", [116, 255, 198, 255]),
            ("RENDER", [255, 193, 90, 255]),
            ("TRACK", [109, 215, 255, 255]),
            ("READY", [181, 110, 255, 255]),
        ];

        for (label, color) in lines {
            self.text(label, x0 + 12, y, 2, color);
            self.fill_rect(x0 + 48, y + 4, x1 - 12, y + 8, [42, 58, 80, 190]);
            y += 18;
        }
    }

    fn draw_monitor_card(&mut self, bounds: (i32, i32, i32, i32), accent: [u8; 4]) {
        let (x0, y0, x1, y1) = bounds;
        self.draw_glass_card(x0, y0, x1, y1, [7, 14, 25, 220], accent);
        self.text("MONITOR", x0 + 12, y0 + 12, 3, accent);
        self.text("72%", x1 - 56, y0 + 12, 3, [255, 255, 255, 230]);

        let center_x = (x0 + x1) / 2;
        let center_y = y0 + 54;
        self.circle(center_x, center_y, 22, [42, 70, 102, 180]);
        self.circle(center_x, center_y, 20, accent);
        self.text("OK", center_x - 12, center_y - 8, 3, [255, 255, 255, 255]);

        self.fill_rect(x0 + 16, y1 - 28, x0 + 90, y1 - 18, accent);
        self.fill_rect(x0 + 102, y1 - 40, x0 + 152, y1 - 18, [180, 100, 255, 180]);
        self.fill_rect(x0 + 164, y1 - 24, x1 - 16, y1 - 18, [116, 255, 198, 180]);
    }

    fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        let start_x = x0.min(x1).max(0) as u32;
        let end_x = x0.max(x1).min(self.width as i32) as u32;
        let start_y = y0.min(y1).max(0) as u32;
        let end_y = y0.max(y1).min(self.height as i32) as u32;

        if start_x >= end_x || start_y >= end_y {
            return;
        }

        for y in start_y..end_y {
            for x in start_x..end_x {
                self.set_pixel(x as i32, y as i32, color);
            }
        }
    }

    fn outline_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        self.line(x0, y0, x1, y0, color);
        self.line(x1, y0, x1, y1, color);
        self.line(x1, y1, x0, y1, color);
        self.line(x0, y1, x0, y0, color);
    }

    fn circle(&mut self, center_x: i32, center_y: i32, radius: i32, color: [u8; 4]) {
        let radius_sq = radius * radius;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius_sq {
                    self.set_pixel(center_x + dx, center_y + dy, color);
                }
            }
        }
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
