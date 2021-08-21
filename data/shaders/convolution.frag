# version 450

layout(location = 0) in vec3 world_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform samplerCube environment_map;

const float PI = 3.14159265359;

void main() {
    vec3 N = normalize(world_pos);

    vec3 irradiance = vec3(0.0);

    vec3 up = vec3(0.0, -1.0, 0.0);
    vec3 right = normalize(cross(up, N));
    up = normalize(cross(N, right));

    float sample_delta = 0.0125;
    float n_samples = 0.0;

    for (float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for (float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            vec3 tangent_sample = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
            vec3 sample_vec = tangent_sample.x * right + tangent_sample.y * up + tangent_sample.z * N;

            irradiance += texture(environment_map, sample_vec).rgb * cos(theta) * sin(theta);
            n_samples++;
        }
    }

    irradiance = PI * irradiance * (1.0 / n_samples);

    f_color = vec4(irradiance, 1.0);
}
