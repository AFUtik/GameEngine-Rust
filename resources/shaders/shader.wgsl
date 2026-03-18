struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct Camera {
    view_proj: mat4x4<f32>,
};

struct Model {
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> model: Model;

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) color: vec4<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.pos = camera.view_proj * model.model * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.color = color;
    return out;
}

@group(2) @binding(0)
var my_texture: texture_2d<f32>;
@group(2) @binding(1)
var my_sampler: sampler;

@fragment
fn fs_main(@location(0) frag_uv: vec2<f32>, @location(1) color: vec4<f32>) -> @location(0) vec4<f32> {
    let tex_color = textureSample(my_texture, my_sampler, frag_uv);
    return tex_color * color;
}