#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 frag_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform Data {
    vec3 view_pos;
} uniforms;

const vec3 LIGHT_POS = vec3(0.4, -0.4, 1.0);

const vec3 AMBIENT_COLOR = vec3(0.3, 0.3, 0.3);
const vec3 LIGHT_COLOR = vec3(1.0, 1.0, 1.0);
const vec3 OBJECT_COLOR = vec3(0.8, 0.0, 0.0);

const float SHININESS = 64.0;
const float SPEC_STRENGTH = 0.8;

void main() {
    vec3 light_dir = normalize(LIGHT_POS - frag_pos);

    float diffuse_factor = max(dot(v_normal, light_dir), 0.0);
    vec3 diffuse = diffuse_factor * LIGHT_COLOR;

    vec3 view_dir = normalize(uniforms.view_pos - frag_pos);
    vec3 reflect_dir = reflect(-light_dir, v_normal);
    float specular_component = pow(max(dot(view_dir, reflect_dir), 0.0), SHININESS);
    vec3 specular = SPEC_STRENGTH * specular_component * LIGHT_COLOR;

    vec3 result = (AMBIENT_COLOR + diffuse + specular) * OBJECT_COLOR;

    f_color = vec4(result, 1.0);
}