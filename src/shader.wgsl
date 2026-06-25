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

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let grid_size = vec2<f32>(24.0, 12.0);
    let grid_position = fract(input.uv * grid_size);

    let distance_x = abs(grid_position.x - 0.5);
    let distance_y = abs(grid_position.y - 0.5);

    let vertical_line =
        1.0 - smoothstep(0.46, 0.49, distance_x);

    let horizontal_line =
        1.0 - smoothstep(0.46, 0.49, distance_y);

    let grid = max(vertical_line, horizontal_line);

    let base_color = vec3<f32>(0.025, 0.04, 0.09);
    let line_color = vec3<f32>(0.18, 0.55, 1.0);

    let color = mix(base_color, line_color, grid * 0.65);

    return vec4<f32>(color, 1.0);
}