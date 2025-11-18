
#version 450

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec3 inColor;

layout(set = 0, binding = 0) uniform UBO {
    mat4 mvp;
    float time;
} ubo;

layout(location = 0) out vec4 fragColor;

void main() {
    fragColor = vec4(inColor, 1.0);
    gl_Position = ubo.mvp * vec4(inPos, 1.0);
}
