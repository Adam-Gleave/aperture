#version 450

// TODO: Remove these uniforms
layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 frag_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform Data {
    vec3 view_pos;
} uniforms;

void main() {
    float depth = gl_FragCoord.z;
    depth = 0.1 * 1000.0 / (1000.0 + depth * (0.1 - 1000.0));
    depth = depth / 10.0;

    f_color = vec4(depth, depth, depth, 1.0);
}