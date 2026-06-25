struct Camera {
    view_projection: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var surface_texture: texture_2d<f32>;

@group(1)
@binding(1)
var surface_sampler: sampler;

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
    return textureSample(
        surface_texture,
        surface_sampler,
        input.uv,
    );
}