#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 frag_pos;

layout(set = 0, binding = 0) uniform Data {
    mat4 view;
    mat4 proj;
} uniforms;

layout(push_constant) uniform VertPushConstants {
    mat4 model;
} push_constants;

void main() {
    mat4 modelview = uniforms.view * push_constants.model;

    gl_Position = uniforms.proj * modelview * vec4(position, 1.0);

    v_normal = transpose(inverse(mat3(push_constants.model))) * normal;
    frag_pos = vec3(push_constants.model * vec4(position, 1.0));
}
