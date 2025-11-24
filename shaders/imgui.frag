#version 450 core

layout(binding = 0) uniform sampler2D tex;

layout(location = 0) in vec2 vUV;
layout(location = 1) in vec4 vColor;

layout(location = 0) out vec4 outColor;

void main() {
    // outColor = vec4(1, 0, 0, 1);

    outColor = vColor * texture(tex, vUV);
}
