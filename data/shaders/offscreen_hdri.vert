#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
	mat4 proj;
	mat4 views[6]; 
} uniforms;

layout(location = 0) out vec3 local_pos;
 
layout(push_constant) uniform VertPushConstants {
    uint index;
} push_constants;

void main() {
	local_pos = vec3(position);
	gl_Position = uniforms.proj * uniforms.views[push_constants.index] * vec4(position, 1.0);
}