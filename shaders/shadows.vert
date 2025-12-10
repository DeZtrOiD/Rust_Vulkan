// shadows.vert
#version 450
// #extension GL_ARB_shader_viewport_layer_array: enable
// #extension VK_NV_viewport_array2: enable

layout(location = 0) in vec3 inPosition;

layout(set = 0, binding = 0) uniform ShadowUniform {
    mat4 lightSpaceMatrix;
    uint indx;
    vec3 _pad;
} ubo;

layout(set = 1, binding = 0) uniform MBO {
    mat4 model;
    mat4 normal;
} model;

void main() {
    gl_Position = ubo.lightSpaceMatrix * model.model * vec4(inPosition, 1.0);
    // gl_Layer = int(ubo.indx);
}
