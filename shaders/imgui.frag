#version 450 core

layout(binding = 1) uniform sampler2D tex;

layout(location = 0) in vec2 vUV;
layout(location = 1) in vec4 vColor;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vColor * texture(tex, vUV);
}
