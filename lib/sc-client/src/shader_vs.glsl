#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;

layout(location = 0) out vec3 o_normal;

layout(set = 0, binding = 0) uniform Data {
    mat4 worldview;
    mat4 proj;
} uniforms;

void main() {
    o_normal = transpose(inverse(mat3(uniforms.worldview))) * i_normal;
    gl_Position = uniforms.proj * uniforms.worldview * vec4(i_position, 1.0);
}
