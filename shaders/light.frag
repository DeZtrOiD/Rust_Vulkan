#version 450

layout(location = 0) in vec3 fragPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragTexCoord;
layout(location = 3) in vec3 camPos;

// layout(set = 0, binding = 0) uniform UBO {
//     mat4 view_proj;
//     // mat4 world;
//     vec4 cam_pos;
//     float time;
// } ubo;

layout(set = 3, binding = 0) uniform MBO {
    mat4 model;
} model;

struct DirectionalLight {
    vec4 direction; // xyz = dir, w unused
    vec4 color;     // rgb + intensity in w
};

struct PointLight {
    vec4 position;
    vec4 color;  // rgb + intensity
    vec4 coefficient;
    vec4 _pad;
    // vec4 _pad1;
};

struct Spotlight {
    vec4 position;
    vec4 direction; // xyz + cutoff radians in w
    vec4 color;     // rgb + intensity
    vec4 cut_off;
    // vec3 _pad;
};

layout(std430, set = 0, binding = 1) buffer LightsSSBO {
    uint light_count_directional;
    uint light_count_point;
    uint light_count_spotlight;
    uint _pad_ssbo;
    DirectionalLight directional_lights[5];
    PointLight       point_lights[5];
    Spotlight        spotlights[5];
};

layout(set = 1, binding = 0) uniform sampler2D textureSampler;

layout(set = 2, binding = 0) uniform MaterialUBO {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
    vec4 extra; // extra.x = shininess
} material;

layout(location = 0) out vec4 outColor;

vec3 calc_Blinn_Phong(vec3 N, vec3 L, vec3 V, vec3 light_color, float intensity, float shininess, vec3 specular_color, vec3 albedo) {
    float diff = max(dot(N, L), 0.0);

    vec3 H = normalize(L + V);
    float spec = pow(max(dot(N, H), 0.0), shininess);

    return light_color * intensity * (diff * albedo + spec * specular_color);
}

void main() {
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(camPos - fragPos);

    vec4 texture_full = texture(textureSampler, fragTexCoord);

    // Ka
    vec3 ambient_m  = material.ambient.rgb;
    // Kd
    vec3 diffuse_m  = material.diffuse.rgb;
    // Ks
    vec3 specular_m = material.specular.rgb;
    // Ns
    float shininess =  material.extra.x;//32.0;

    // цвета либо в текстуре либо в diffuse_m (Kd в mtl) 
    // оно позволит фон делать прозрачный
    vec3 albedo = texture_full.rgb * texture_full.a + diffuse_m * (1.0 - texture_full.a);

    vec3 result = ambient_m * albedo;

    // -----------------------------------
    // Directional lights
    // -----------------------------------
    for (uint i = 0; i < light_count_directional; ++i) {
        vec3 L = normalize(-directional_lights[i].direction.xyz);
        vec3 col = directional_lights[i].color.rgb;
        float light_intensity = directional_lights[i].color.w;

        result += calc_Blinn_Phong(N, L, V, col, light_intensity, shininess, specular_m, albedo);
    }

    // -----------------------------------
    // Point lights
    // -----------------------------------
    for (uint i = 0; i < light_count_point; ++i) {
        vec3 lightPos = point_lights[i].position.xyz;
        float c_const  = point_lights[i].coefficient.x;
        float c_lin  = point_lights[i].coefficient.y;
        float c_quad  = point_lights[i].coefficient.z;

        vec3 L = (lightPos - fragPos);
        float dist = length(L);
        L = normalize(L);
        // 1 / (a + d*b + d^2*c)
        float attenuation = 1.0 / (c_const + c_lin * dist + c_quad * dist * dist); 

        float light_intensity = point_lights[i].color.w ;
        vec3 col = point_lights[i].color.rgb;
        
        result += calc_Blinn_Phong(N, L, V, col, attenuation * light_intensity, shininess, specular_m, albedo);
    }

    // -----------------------------------
    // Spotlights
    // -----------------------------------
    for (uint i = 0; i < light_count_spotlight; ++i) {
        vec3 lightPos = spotlights[i].position.xyz;
        vec3 L = normalize(lightPos - fragPos);
        vec3 dir = normalize(spotlights[i].direction.xyz);
        
        float cutoff = spotlights[i].direction.w;
        float outer_cos = spotlights[i].cut_off.x;
        // https://registry.khronos.org/OpenGL-Refpages/gl4/html/smoothstep.xhtml
        // Results are undefined if edge0 ≥ edge1.
        float inner_cos = max(spotlights[i].cut_off.y, outer_cos + 0.0001);
        
        float spot_angle = dot(L, -dir);
        float spot_intensity = smoothstep(outer_cos, inner_cos, spot_angle);

        float light_intensity = spotlights[i].color.w;

        vec3 col = spotlights[i].color.rgb;


        result += calc_Blinn_Phong(N, L, V, col, light_intensity * spot_intensity, shininess, specular_m, albedo);
    }

    outColor = vec4(result, 1.0);
}
