#version 450

#define MAX_LIGHT_COUNT 255

layout(location = 0) in vec3 frag_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 tex_coord;
layout(location = 3) in mat4 view;

layout(set = 0, binding = 1) uniform sampler2D base_color_tex;
layout(set = 0, binding = 2) uniform sampler2D normal_tex;
layout(set = 0, binding = 3) uniform sampler2D metal_rough_tex;
layout(set = 0, binding = 4) uniform sampler2D ao_tex;

struct PointLight {
    vec4 position;
    vec4 color;
    uvec4 power;
};

layout(set = 0, binding = 5) uniform Data {
    vec4 view_pos;
    PointLight lights[MAX_LIGHT_COUNT];
} uniforms;

layout(push_constant) uniform FragPushConstants {
    layout(offset = 64) vec4 base_color;
    float metalness;
    float roughness;
    float reflectance;
    uint point_light_count;
} push_constants;

layout(location = 0) out vec4 f_color;

const float PI = 3.1415926538;

const int POINT_LIGHT = 0;
const int SPOT_LIGHT  = 1;

const float ILLUMINANCE_FACTOR[2] = float[2](
    4 * PI,
    PI
);

const float roughness = 0.4;
const float metalness = 1.0;

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
float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;
    float denominator = NdotV * (1.0 - k) + k;

    return NdotV / denominator;
}

// Smith-correlated visibility function. 
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

// Calculates the Disney diffuse, with modifications for energy conservation.
float Fd_Burley(float NdotV, float NdotL, float LdotH, float alpha) {
    float energy_bias = mix(0.0, 0.5, alpha);
    float energy_factor = mix(1.0, 1.0 / 1.51, alpha);

    float F90 = energy_bias + 2.0 * LdotH * LdotH * alpha;

    float light_scatter = Fd_Schlick(NdotL, 1.0, F90);
    float view_scatter = Fd_Schlick(NdotV, 1.0, F90);

    return light_scatter * view_scatter * energy_factor;
}

vec3 CalculateNormal() {
    vec3 tangentNormal = texture(normal_tex, tex_coord.xy).xyz * 2.0 - 1.0;

	vec3 q1 = dFdx(frag_pos);
	vec3 q2 = dFdy(frag_pos);
	vec2 st1 = dFdx(tex_coord);
	vec2 st2 = dFdy(tex_coord);

	vec3 N = normalize(v_normal);
	vec3 T = normalize(q1 * st2.t - q2 * st1.t);
	vec3 B = -normalize(cross(N, T));
	mat3 TBN = mat3(T, B, N);

	return normalize(TBN * tangentNormal);
}

void main() {
    vec3 result = vec3(0.0, 0.0, 0.0);

    vec3 base_color = texture(base_color_tex, tex_coord.xy).rgb;
    float metalness = texture(metal_rough_tex, tex_coord.xy).b;
    float roughness = texture(metal_rough_tex, tex_coord.xy).g;
    float ao = texture(ao_tex, tex_coord.xy).r;

    float reflectance_clamped = clamp(push_constants.reflectance, 0.0, 1.0);
    float reflectance = 0.16 * reflectance_clamped * reflectance_clamped;

    // V: view vector
    // N: normal
    vec3 V = normalize(vec3(uniforms.view_pos) - frag_pos);
    vec3 N = CalculateNormal();

    float alpha = roughness * roughness;

    vec3 F0 = vec3(reflectance);

    vec3 specular_color = mix(F0, base_color, metalness);
    F0 = vec3(max(max(specular_color.r, specular_color.g), specular_color.b));

    vec3 Lo = vec3(0.0);

    for (int i = 0; i < 4; i++) {
        PointLight light = uniforms.lights[i];

        vec3 light_pos = vec3(light.position * view);

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
        float G = G_Smith(NdotV, NdotL, roughness);

        // Normal Distribution Function (NDF): GGX
        float D = D_GGX(NdotH, alpha);

        // Calulcate the specular contribution with the BRDF
        vec3 numerator = F * G * D;
        float denominator = 4 * NdotL + NdotL;
        vec3 specular = clamp(numerator / denominator, 0.0, 1.0);
        specular *= specular_color;

        // Calculate the Disney diffuse contribution.
        float F90 = 0.5 * 2.0 * roughness * LdotH * LdotH;
        float diffuse_factor = Fd_Burley(NdotV, NdotL, LdotH, roughness); 
        diffuse_factor *= (1.0 - metalness);
        vec3 diffuse = diffuse_factor * base_color;

        // Calulcate the radiance of this light source.
        float distance = length(light_pos - frag_pos);
        float attenuation = 1.0 / max(distance * distance, 0.01 * 0.01);
        vec3 light_color = vec3(light.color) * light.power.x / ILLUMINANCE_FACTOR[POINT_LIGHT];
        vec3 radiance = light_color * attenuation * NdotL;

        Lo += (diffuse + specular) * radiance;
    }

    vec3 color = Lo * ao;

    f_color = vec4(color, 1.0);
}
