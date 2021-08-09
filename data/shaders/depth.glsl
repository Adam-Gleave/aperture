#version 450

// TODO: Remove these uniforms
layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 frag_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform Data {
    vec3 view_pos;
} uniforms;

const float NEAR = 0.1;
const float FAR = 10.0;

float LineariseDepth(float depth) {
    float z = depth * 2.0 - 1.0;
    
    return (2.0 * NEAR * FAR) / (FAR + NEAR - z * (FAR - NEAR));
}

void main() {
    float depth = LineariseDepth(gl_FragCoord.z) / FAR;

    f_color = vec4(vec3(depth), 1.0);
}