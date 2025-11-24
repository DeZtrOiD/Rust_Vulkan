#version 450

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec2 inTexCoord;

layout(set = 0, binding = 0) uniform UBO {
    mat4 mvp;
    float time;
} ubo;

layout(set = 3, binding = 0) uniform MBO {
    mat4 mvp;
} model;

layout(location = 0) out vec3 fragPos;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragTexCoord;

void main() {
    fragPos = inPos;
    fragNormal = inNormal;
    fragTexCoord = inTexCoord;
    gl_Position = ubo.mvp * model.mvp * vec4(inPos, 1.0);
}