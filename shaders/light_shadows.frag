// light_shadows.frag

/// Это работает плохо

#version 450

layout(location = 0) in vec3 fragPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragTexCoord;
layout(location = 3) in vec3 camPos;
layout(location = 4) in vec4 fragPosLightSpace;


const uint MAX_LIGHTS = 5;

layout(set = 3, binding = 0) uniform MBO {
    mat4 model;
} model;

struct DirectionalLight {
    vec4 direction; // xyz = dir, w unused
    vec4 color;     // rgb + intensity in w
    mat4 light_mtx;
};

struct PointLight {
    vec4 position;
    vec4 color;  // rgb + intensity
    vec4 coefficient;
    vec4 _pad;
    // vec4 _pad1;
    mat4 light_mtx;
};

struct Spotlight {
    vec4 position;
    vec4 direction; // xyz + cutoff radians in w
    vec4 color;     // rgb + intensity
    vec4 cut_off;
    // vec3 _pad;
    mat4 light_mtx;
};

layout(std430, set = 0, binding = 1) buffer LightsSSBO {
    uint light_count_directional;
    uint light_count_point;
    uint light_count_spotlight;
    float time;
    // uint _pad_ssbo;
    DirectionalLight directional_lights[5];
    PointLight point_lights[5];
    Spotlight spotlights[5];
};

layout(set = 1, binding = 0) uniform sampler2D textureSampler;

layout(set = 2, binding = 0) uniform MaterialUBO {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
    vec4 extra; // extra.x = shininess
} material;

layout(set = 4, binding = 0) uniform sampler2DArrayShadow shadowMap;


layout(location = 0) out vec4 outColor;


float calculateShadow(int lightIndex, vec4 PosLightSpace, mat4 light_mtx, vec3 normal, vec3 lightDir, bool flag) {
    const vec2 gMapSize = vec2(512, 512);

    PosLightSpace = light_mtx * PosLightSpace;
    vec3 projCoords = PosLightSpace.xyz / PosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;
    
    if (projCoords.z > 1.0 || projCoords.x < 0.0 || projCoords.x > 1.0 || 
        projCoords.y < 0.0 || projCoords.y > 1.0) {
        return 1.0;
    }

    float minBias = 0.2;
    float maxBias = 0.27;

    if (flag) {
        maxBias = 0.0148;
    } else {
        maxBias = 0.27;
    }
    float bias = clamp(6 * minBias * max(dot(normal, lightDir), 1.0 - dot(normal, lightDir)), minBias, maxBias);
    // bias = 0.0148;
    float currentDepth = projCoords.z;

    float compareDepth = projCoords.z - bias;

    vec2 texelSize = 1.0 / gMapSize;

    float shadow = 0.0;

    // for (int x = -1; x <= 1; x++) {
    //     for (int y = -1; y <= 1; y++) {
    //         vec2 offset = vec2(x, y) * texelSize;

    //         vec4 coord = vec4(
    //             projCoords.xy + offset,
    //             lightIndex,
    //             compareDepth
    //         );

    //         shadow += texture(shadowMap, coord);
    //     }
    // }

    // shadow /= 9.0;

    // return shadow;


    vec4 coord = vec4(projCoords.x, projCoords.y, lightIndex, currentDepth - bias);
    shadow = texture(shadowMap, coord);
    return shadow;
    
}

vec3 calc_Blinn_Phong(vec3 N, vec3 L, vec3 V, vec3 light_color, float intensity, float shininess, vec3 specular_color, vec3 albedo) {
    float diff = max(dot(N, L), 0.0);

    vec3 H = normalize(L + V);
    float spec = pow(max(dot(N, H), 0.0), shininess);

    return light_color * intensity * (diff * albedo + spec * specular_color);
}

void main() {
    const float PI = 3.1415;
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(camPos - fragPos);

    vec2 uv = fragTexCoord;
    vec2 center = vec2(0.5, 0.5);
    vec2 dir = uv - center;
    float dist = length(dir);

    float angle = dist * 5.0 * sin(time);
    float s = sin(angle);
    float c = cos(angle);
    mat2 rot = mat2(c, -s, s, c);
    uv = center + rot * dir;

    // uv = fragTexCoord;

    vec4 texture_full = texture(textureSampler, uv);

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

        float shadow = calculateShadow(int(i), fragPosLightSpace, directional_lights[i].light_mtx,  N, L, false);

        result += calc_Blinn_Phong(N, L, V, col, light_intensity, shininess, specular_m, albedo) * (shadow);
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

        // float shadow = calculateShadow(int(i + MAX_LIGHTS), fragPosLightSpace, point_lights[i].light_mtx);
        
        result += calc_Blinn_Phong(N, L, V, col, attenuation * light_intensity, shininess, specular_m, albedo);
        //  * (1.0 - shadow);
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

        // float shadow = calculateShadow(int(i + MAX_LIGHTS * 2), fragPosLightSpace, spotlights[i].light_mtx);
        float shadow = calculateShadow(int(i + MAX_LIGHTS * 2), fragPosLightSpace, spotlights[i].light_mtx,  N, L, true);
        
        result += calc_Blinn_Phong(N, L, V, col, light_intensity * spot_intensity, shininess, specular_m, albedo) * (shadow);
    }
    // ========== ПРОСТАЯ ПРОВЕРКА ТЕНЕЙ ==========
    // ВРЕМЕННАЯ ПРОВЕРКА: замените сложные тени на простой тест
    // vec3 N = normalize(fragNormal);
    // vec3 V = normalize(camPos - fragPos);
    
    // // Простой цвет для проверки
    // vec3 result_ = vec3(0.5, 0.5, 0.5); // Серый базовый цвет
    
    // // Проверка shadow map
    // vec2 testUV = fragTexCoord ;
    // vec4 shadowTest0 = texture(shadowMap, vec3(testUV, 0.0));
    // if (shadowTest0.r < 0.99999) {
    //     result_ = vec3(shadowTest0.r, 0.0, 0.0);  // Красный
    // }

    // // Проверяем десятый слой (spotlight 0)
    // vec4 shadowTest10 = texture(shadowMap, vec3(testUV, 1.0));
    // if (shadowTest10.r < 0.99999) {
    //     result_ = result_ + vec3(0.0, shadowTest10.r, 0.0);  // Зеленый
    // }

    // vec4 shadowTest20 = texture(shadowMap, vec3(testUV, 2.0));
    // if (shadowTest20.r < 0.99999) {
    //     result_ = result_ + vec3(0.0, 0.0, shadowTest20.r);  // Зеленый
    // }

    // vec4 shadowTest30 = texture(shadowMap, vec3(testUV, 3.0));
    // if (shadowTest30.r < 0.99999) {
    //     result_ = result_ + vec3(shadowTest30.r, 0.0, 0.0);  // Зеленый
    // }

    // // Если оба пустые
    // if (shadowTest0.r > 0.99999 && shadowTest10.r > 0.99999 && shadowTest20.r > 0.99999) {
    //     result_ = vec3(0.0, 0.0, 1.0);  // Синий
    // }
       
    // outColor = vec4(result_, 1.0);

    outColor = vec4(result, 1.0);
}
