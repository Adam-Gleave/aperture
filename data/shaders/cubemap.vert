#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
    mat4 proj;
    mat4 view;
} uniforms;

layout(location = 0) out vec3 local_pos;

void main() {
    local_pos = position;
    gl_Position = uniforms.proj * uniforms.view * vec4(position, 1.0);
}
