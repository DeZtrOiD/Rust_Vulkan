#version 450

layout(location = 0) in vec3 fragPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragTexCoord;

layout(set = 1, binding = 0) uniform sampler2D textureSampler;

layout(std140, set = 2, binding = 0) uniform MaterialUBO {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
    vec4 extra; // extra.x = shininess
} material;

layout(location = 0) out vec4 outColor;

void main() {
    // --- Материал ---
    vec3 ambient_m  = material.ambient.rgb;
    vec3 diffuse_m  = material.diffuse.rgb;
    vec3 specular_m = material.specular.rgb;
    float shininess = material.extra.x;

    // --- Текстура ---
    vec4 texColor = texture(textureSampler, fragTexCoord);
    vec3 baseColor = texColor.rgb;

    // --- Свет ---
    vec3 lightDir = normalize(vec3(1.0, 1.0, 1.0));
    vec3 N = normalize(fragNormal);
    
    // --- Диффузная часть ---
    float diffTerm = max(dot(N, lightDir), 0.0);
    vec3 diffuse = diffTerm * diffuse_m * baseColor;

    // --- блики (Blinn–Phong) ---
    vec3 viewDir = normalize(-fragPos);
    vec3 halfDir = normalize(lightDir + viewDir);
    float specTerm = pow(max(dot(N, halfDir), 0.0), shininess);
    vec3 specular = specTerm * specular_m;

    // --- эмбиент ---
    vec3 ambient = 0.2 * ambient_m * baseColor;

    outColor = vec4(ambient + diffuse + specular, 1.0);
}
