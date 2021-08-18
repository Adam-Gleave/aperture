# version 450

layout(location = 0) in vec3 local_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform samplerCube environment_map;

const vec2 inv_tan = vec2(0.1591, 0.3183);

void main() {
    vec3 env_color = texture(environment_map, local_pos).rgb;
    env_color = env_color / (env_color + vec3(1.0));

    f_color = vec4(pow(env_color, vec3(1.0 / 2.2)), 1.0);
}
