#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv_coord;

layout(set = 0, binding = 0) uniform Data {
    mat4 proj;
    mat4 view;
} uniforms;

layout(push_constant) uniform VertPushConstants {
    mat4 model;
} push_constants;

layout(location = 0) out vec3 frag_pos;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec2 tex_coord;
layout(location = 3) out mat4 view;

void main() {
    mat4 modelview = uniforms.view * push_constants.model;

    gl_Position = uniforms.proj * modelview * vec4(position, 1.0);
    gl_Position.x = -gl_Position.x;

    frag_pos = vec3(push_constants.model * vec4(position, 1.0));
    v_normal = transpose(inverse(mat3(push_constants.model))) * normal;
    tex_coord = uv_coord;
    view = uniforms.view;
}
