# version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
    mat4 proj;
    mat4 views[6];
} uniforms;

layout(push_constant) uniform VertPushConstants {
    uint index;
} push_constants;

layout(location = 0) out vec3 world_pos;

void main() {
    world_pos = position;
    gl_Position = uniforms.proj * uniforms.views[push_constants.index] * vec4(world_pos, 1.0);
}
