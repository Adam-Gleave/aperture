# version 450

layout(location = 0) in vec3 local_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D hdri;

const vec2 inv_tan = vec2(0.1591, 0.3183);

vec2 SampleSphericalMap(vec3 v) {
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= inv_tan;
    uv.y *= -1.0;
    uv += 0.5;

    return uv;
}

void main() {
    vec2 uv = SampleSphericalMap(normalize(local_pos));
    vec3 color = texture(hdri, uv).rgb;

    f_color = vec4(color, 1.0);
}
