// imgui.vert
#version 450

layout(binding = 0) uniform Ubo {
    vec2 scale;
    vec2 translate;
} ubo;

layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aUV;
layout(location = 2) in vec4 aColor;

layout(location = 0) out vec2 vUV;
layout(location = 1) out vec4 vColor;

void main() {
    vUV = aUV;
    vColor = aColor;
    gl_Position = vec4(aPos * ubo.scale + ubo.translate, 0.0, 1.0);
}
