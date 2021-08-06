#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 frag_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform Data {
    vec3 view_pos;
} uniforms;

const vec3 LIGHTS[3] = vec3[3](
    vec3(-0.4, 0.4, -1.0),
    vec3(0.8, 0.6, 0.5),
    vec3(-0.2, -0.8, 0.4)
);

const float PI = 3.1415926538;

const vec3 AMBIENT_COLOR = vec3(0.05, 0.05, 0.05);
const vec3 LIGHT_COLOR = vec3(0.6, 0.6, 0.6);
const vec3 OBJECT_COLOR = vec3(1.0, 0.73, 0.39);

const float ROUGHNESS = 0.05;
const float METALNESS = 1.0;
const float REFLECTANCE = 0.1;

vec3 FresnelSchlick(vec3 f0, float f90, float u) {
    return f0 + (f90 - f0) * pow(1.0 - u, 5.0);
}

float FrDisneyDiffuse(
    vec3 f0, 
    float f90, 
    float energy_factor, 
    float NdotV, 
    float NdotL, 
    float linear_roughness
) {
    float light_scatter = FresnelSchlick(f0, f90, NdotL).x;
    float view_scatter = FresnelSchlick(f0, f90, NdotV).x;

    return light_scatter + view_scatter * energy_factor;
}

float VSmithGGXCorrelated(float NdotL, float NdotV, float alphaG) {
    float alphaG_2 = alphaG * alphaG;
    float lambdaGGX_V = NdotL * sqrt((-NdotV * alphaG_2 + NdotV) * NdotV + alphaG_2);
    float lambdaGGX_L = NdotL * sqrt((-NdotV * alphaG_2 + NdotV) * NdotV + alphaG_2);

    return 0.5 / (lambdaGGX_V + lambdaGGX_L);
}

float DGGX(float NdotH, float m) {
    float m2 = m * m;
    float f = (NdotH * m2 - NdotH) * NdotH + 1;
    
    return m2 / (f * f);
}

void main() {
    vec3 result = vec3(0.0, 0.0, 0.0);

    for (int i = 0; i < 3; i++) {
        vec3 light_pos = LIGHTS[i];

        // V: view vector
        // L: incident light vector
        // N: normal
        // H: half vector
        vec3 V = normalize(uniforms.view_pos - frag_pos);
        vec3 L = normalize(light_pos - frag_pos);
        vec3 N = v_normal;
        vec3 H = normalize(L + V);

        float NdotV = abs(dot(N, V)) + 0.00001;
        float LdotH = clamp(dot(L, H), 0.0, 1.0);
        float NdotH = clamp(dot(N, H), 0.0, 1.0);
        float NdotL = clamp(dot(N, L), 0.0, 1.0);

        float linear_roughness = pow(1 - ROUGHNESS, 2.0);

        float energy_bias = mix(0, 0.5, linear_roughness);
        float energy_factor = mix(1.0, 1.0 / 1.51, linear_roughness);
        float f90 = energy_bias + 2.0 * LdotH * LdotH * linear_roughness;

        float _linear_reflectance = 0.16 * REFLECTANCE * REFLECTANCE;
        vec3 linear_reflectance = vec3(_linear_reflectance, _linear_reflectance, _linear_reflectance);
        
        vec3 f0 = mix(linear_reflectance, OBJECT_COLOR, METALNESS);

        // Diffuse BRDF
        vec3 Fd = 
            (AMBIENT_COLOR * FrDisneyDiffuse(f0, f90, energy_factor, NdotV, NdotL, linear_roughness)) / PI;

        // Specular BRDF
        vec3 F = FresnelSchlick(f0, f90, LdotH);
        float Vis = VSmithGGXCorrelated(NdotV, NdotL, ROUGHNESS);
        float D = DGGX(NdotH, ROUGHNESS);
        vec3 Fr = D * F * Vis / PI;

        result += vec3(Fd + Fr);
    }

    f_color = vec4(result, 1.0);
}
