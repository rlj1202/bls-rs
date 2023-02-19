#version 450

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec2 vertex_uv;

layout(location = 0) out vec2 v_uv;

layout(set = 0, binding = 0) uniform camera_view_proj {
    mat4 view_proj;
    mat4 view;
    mat4 inverse_view;
    mat4 projection;
    vec3 world_position;
    float width;
    float height;
};

layout(set = 2, binding = 0) uniform mesh {
    mat4 model;
    mat4 inverse_transpose_model;
    uint flags;
};

void main() {
    v_uv = vertex_uv;
    gl_Position = view_proj * model * vec4(vertex_position, 1.0);
}