
#version 450

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec3 inColor;

layout(set = 0, binding = 0) uniform UBO {
    mat4 mvp;
    float time;
} ubo;

layout(location = 0) out vec4 fragColor;

void main() {
    float t = ubo.time;
    vec3 shifted = inColor * 0.5 + 0.5 * vec3(sin(t), cos(t), sin(t * 0.5));
    fragColor = vec4(shifted, 1.0);
    gl_Position = ubo.mvp * vec4(inPos, 1.0);
}
