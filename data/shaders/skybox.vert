#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
    mat4 proj;
    mat4 view;
} uniforms;

layout(location = 0) out vec3 local_pos;

void main() {
    local_pos = position;

    mat4 view_rotation = mat4(mat3(uniforms.view));
    vec4 clipped_pos = uniforms.proj * view_rotation * vec4(local_pos, 1.0);

    gl_Position = clipped_pos.xyww;
}
