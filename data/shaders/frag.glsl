#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 frag_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform Data {
    mat4 rotation;
    vec3 view_pos;
} uniforms;

const vec3 LIGHTS[4] = vec3[4](
    vec3(-2.8, 0.2, -0.5),
    vec3(0.6, 1.2, 0.6),
    vec3(12.7, 2.1, 0.5),
    vec3(-18.7, -12.5, 8.6)
);

const float PI = 3.1415926538;

const vec3 LIGHT_COLOR = vec3(1.0, 1.0, 1.0);

const float roughness = 0.4;
const float metalness = 1.0;
const float REFLECTANCE = 0.04;

layout(push_constant) uniform FragPushConstants {
    layout(offset = 64) vec4 base_color;
    float metalness;
    float roughness;
} push_constants;

// Calculates the Fresnel-Schlick approximation.
//
// This describes the amount of light that reflects from the surface given 
// its index of refraction.
//
// Instead of using IoR, which is unintuitive, we use F0:
//  - F0: the reflectance at normal incidence (angle of 0 degrees).
//
// For dielectrics, F0 is monochromatic, and usually between 2% and ~20%.
// For metals, this is the "specular color", and is RGB.
//
vec3 F_FresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(max(1.0 - cosTheta, 0.0), 5.0);
}

// Calculate the Smith Schlick-GGX approximation.
//
// This is the geometric shadowing function, and describes the shadowing
// from material microfacets.
//
// Hence, roughness is accounted for in this calculation. The higher the
// roughness, the greater the geometric shadowing.
//
float GeometrySchlickGGX(float NdotV, float k) {
    float denominator = NdotV * (1.0 - k) + k;

    return NdotV / denominator;
}

float G_Smith(float NdotV, float NdotL, float alpha) {
    float GGX_L = GeometrySchlickGGX(NdotV, alpha);
    float GGX_V = GeometrySchlickGGX(NdotL, alpha);
    
    return 0.5 / GGX_L * GGX_V;
}

// Calculate the GGX (Trowbridge-Reitz) normal distribution.
//
// This Normal Distribution Function (NDF) describes the distribution
// of microfacets for the surface that are angled so as to reflect
// light in the direction of the view.
//
float D_GGX(float NdotH, float alpha) {
    float alpha_2 = alpha * alpha;

    float denominator = 
        PI * 
        ((NdotH * NdotH) * (alpha_2 - 1) + 1) *
        ((NdotH * NdotH) * (alpha_2 - 1) + 1);

    return alpha_2 / denominator;
}

// Calulcates the diffuse Fresnel-Schlick contribution.
float Fd_Schlick(float u, float F0, float F90) {
    return F0 + (F90 - F0) * pow(1.0 - u, 5.0);
}

float Fd_Burley(float NdotV, float NdotL, float LdotH, float alpha) {
    float F90 = 0.5 * 2.0 * alpha * LdotH * LdotH;
    float light_scatter = Fd_Schlick(NdotL, 1.0, F90);
    float view_scatter = Fd_Schlick(NdotV, 1.0, F90);

    return light_scatter * view_scatter * (1.0 / PI);
}

// Calculates the Lambertian diffuse factor.
float lambert(vec3 N, vec3 L) {
    float result = dot(N, L);

    return max(result, 0.0);
}

void main() {
    vec3 result = vec3(0.0, 0.0, 0.0);

    vec3 base_color = push_constants.base_color.rgb;
    float metalness = push_constants.metalness; 
    float roughness = clamp(push_constants.roughness, 0.05, 1.0);

    vec3 ambient_color = base_color * vec3(0.03);

    // V: view vector
    // N: normal
    vec3 V = normalize(uniforms.view_pos - frag_pos);
    vec3 N = v_normal;

    float alpha = roughness * roughness;

    vec3 F0 = vec3(0.04);

    vec3 specular_color = mix(F0, base_color, metalness);
    F0 = vec3(max(max(specular_color.r, specular_color.g), specular_color.b));

    vec3 Lo = vec3(0.0);

    for (int i = 0; i < 4; i++) {
        vec3 light_pos = vec3(uniforms.rotation * vec4(LIGHTS[i], 1.0));

        // L: incident light vector
        // H: half vector
        vec3 L = normalize(light_pos - frag_pos);
        vec3 H = normalize(V + L);

        float LdotH = clamp(dot(L, H), 0.0, 1.0);
        float NdotH = clamp(dot(N, H), 0.0, 1.0);
        float HdotV = clamp(dot(H, V), 0.0, 1.0);
        float NdotL = clamp(dot(N, L), 0.00001, 1.0);
        float NdotV = clamp(abs(dot(N, V)), 0.00001, 1.0);

        // Specular highlights: Fresnel-Schlick
        vec3 F = F_FresnelSchlick(HdotV, F0);

        // Geometric shadowing: Smith Schlick-GGX
        float G = G_Smith(NdotV, NdotL, alpha);

        // Normal Distribution Function (NDF): GGX
        float D = D_GGX(NdotH, alpha);

        // Calulcate the specular contribution with the BRDF
        vec3 numerator = F * G * D;
        float denominator = 4 * NdotL + NdotL;
        vec3 specular = clamp(numerator / denominator, 0.0, 1.0);
        specular *= specular_color;

        // Calculate the Disney diffuse contribution.
        float F90 = 0.5 * 2.0 * alpha * LdotH * LdotH;
        float diffuse_factor = Fd_Burley(NdotV, NdotL, LdotH, alpha); 
        diffuse_factor *= (1.0 - metalness);
        vec3 diffuse = diffuse_factor * base_color;

        Lo += (diffuse + specular) * LIGHT_COLOR;
    }

    vec3 color = ambient_color + Lo;

    // Gamma correction
    color = pow(color, vec3(1.0 / 2.2));

    f_color = vec4(color, 1.0);
}
