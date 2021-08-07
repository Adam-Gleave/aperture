#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 frag_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform Data {
    mat4 rotation;
    vec3 view_pos;
} uniforms;

const vec3 LIGHTS[4] = vec3[4](
    vec3(-0.4, 0.4, -1.0),
    vec3(0.8, 0.6, 0.5),
    vec3(-0.2, -0.8, 0.4),
    vec3(0.5, 2.6, 0.4)
);

const float PI = 3.1415926538;

const vec3 AMBIENT_COLOR = vec3(0.5, 0.5, 0.5);
const vec3 LIGHT_COLOR = vec3(1.0, 1.0, 1.0);
const vec3 OBJECT_COLOR = vec3(1.0, 0.73, 0.39);

const float ROUGHNESS = 0.05;
const float METALNESS = 0.5;
const float REFLECTANCE = 0.04;

float DistributionGGX(vec3 N, vec3 H, float roughness) {
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
	
    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
	
    return num / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return num / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
	
    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(max(1.0 - cosTheta, 0.0), 5.0);
}

void main() {
    vec3 result = vec3(0.0, 0.0, 0.0);

    // V: view vector
    // N: normal
    vec3 V = normalize(uniforms.view_pos - frag_pos);
    vec3 N = v_normal;

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, OBJECT_COLOR, METALNESS);

    vec3 Lo = vec3(0.0);

    for (int i = 0; i < 4; i++) {
        vec3 light_pos = vec3(uniforms.rotation * vec4(LIGHTS[i], 1.0));

        // L: incident light vector
        // H: half vector
        vec3 L = normalize(light_pos - frag_pos);
        vec3 H = normalize(L + V);

        // Cook-torrance BRDF
        float NDF = DistributionGGX(N, H, ROUGHNESS);
        float G = GeometrySmith(N, V, L, ROUGHNESS);
        vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - METALNESS;

        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
        vec3 specular = numerator / max(denominator, 0.001);

        // Add to outgoing radiance Lo
        float NdotL = max(dot(N, L), 0.0);
        Lo += (kD * OBJECT_COLOR / PI + specular) * LIGHT_COLOR * NdotL;
    }

    vec3 ambient = vec3(0.03) * OBJECT_COLOR;
    vec3 color = ambient + Lo;

    color = pow(color, vec3(1.0 / 2.2));

    f_color = vec4(color, 1.0);
}
