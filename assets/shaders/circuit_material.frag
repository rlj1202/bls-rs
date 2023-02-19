#version 450

layout(location = 0) in vec2 v_uv;

layout(location = 0) out vec4 o_target;

// layout(set = 1, binding = 0) uniform circuit_material {
    // vec4 color;
// };

layout(set = 1, binding = 1) uniform texture2D circuit_material_texture;
layout(set = 1, binding = 2) uniform sampler circuit_material_sampler;

layout(set = 1, binding = 3) uniform texture2D circuit_material_overlay_texture;
layout(set = 1, binding = 4) uniform sampler circuit_material_overlay_sampler;

void main() {
    vec4 diffuse_color = texture(sampler2D(circuit_material_texture, circuit_material_sampler), v_uv);
    vec4 overlay_color = texture(sampler2D(circuit_material_overlay_texture, circuit_material_overlay_sampler), v_uv);
    o_target = diffuse_color * overlay_color;
}