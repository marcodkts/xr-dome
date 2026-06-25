struct Camera {
    view_projection: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position =
        camera.view_projection * vec4<f32>(input.position, 1.0);

    output.uv = input.uv;

    return output;
}

fn line_at(value: f32, anchor: f32, width: f32) -> f32 {
    let d = abs(value - anchor);
    return 1.0 - smoothstep(width, width * 1.8, d);
}

fn wrapped_line_at(value: f32, anchor: f32, width: f32) -> f32 {
    let d1 = abs(value - anchor);
    let d2 = abs(value - anchor + 1.0);
    let d3 = abs(value - anchor - 1.0);
    let d = min(d1, min(d2, d3));

    return 1.0 - smoothstep(width, width * 1.8, d);
}

fn vertical_band(u: f32, anchor: f32, width: f32) -> f32 {
    return wrapped_line_at(u, anchor, width);
}

fn marker_block(uv: vec2<f32>, anchor_u: f32, center_v: f32) -> f32 {
    let x = vertical_band(uv.x, anchor_u, 0.018);
    let y = line_at(uv.y, center_v, 0.11);

    return x * y;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;

    /*
        Mapeamento do domo 360:
        u = 0.50 -> frente
        u = 0.75 -> direita
        u = 0.00 / 1.00 -> trás
        u = 0.25 -> esquerda
    */

    let base_top = vec3<f32>(0.015, 0.020, 0.040);
    let base_bottom = vec3<f32>(0.030, 0.045, 0.080);

    var color = mix(base_top, base_bottom, uv.y);

    // Grade fina
    let grid_size = vec2<f32>(48.0, 16.0);
    let grid_position = fract(uv * grid_size);

    let grid_x =
        1.0 - smoothstep(0.485, 0.500, abs(grid_position.x - 0.5));

    let grid_y =
        1.0 - smoothstep(0.485, 0.500, abs(grid_position.y - 0.5));

    let grid = max(grid_x, grid_y);
    color = mix(color, vec3<f32>(0.10, 0.22, 0.38), grid * 0.35);

    // Linha do horizonte
    let horizon = line_at(uv.y, 0.5, 0.004);
    color = mix(color, vec3<f32>(0.20, 0.75, 1.00), horizon * 0.85);

    // Linhas verticais fortes a cada 30 graus
    let major = 1.0 - smoothstep(
        0.006,
        0.012,
        abs(fract(uv.x * 12.0) - 0.5),
    );

    color = mix(color, vec3<f32>(0.18, 0.45, 0.80), major * 0.55);

    // Marcadores cardeais
    let front = marker_block(uv, 0.50, 0.50);
    let right = marker_block(uv, 0.75, 0.50);
    let back = marker_block(uv, 0.00, 0.50);
    let left = marker_block(uv, 0.25, 0.50);

    color = mix(color, vec3<f32>(1.00, 0.25, 0.20), front); // frente
    color = mix(color, vec3<f32>(0.25, 1.00, 0.35), right); // direita
    color = mix(color, vec3<f32>(1.00, 0.85, 0.20), back);  // trás
    color = mix(color, vec3<f32>(0.70, 0.35, 1.00), left);  // esquerda

    // Marcas extras acima e abaixo para perceber pitch
    let upper_front = marker_block(uv, 0.50, 0.25);
    let lower_front = marker_block(uv, 0.50, 0.75);

    color = mix(color, vec3<f32>(1.00, 0.55, 0.35), upper_front * 0.8);
    color = mix(color, vec3<f32>(1.00, 0.55, 0.35), lower_front * 0.8);

    return vec4<f32>(color, 1.0);
}