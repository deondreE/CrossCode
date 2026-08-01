#+build windows
package engine

@(private)
VERTEX_SRC :: `#version 330 core
layout(location=0) in vec2 a_pos;
layout(location=1) in vec2 a_uv;
layout(location=2) in vec4 a_color;
layout(location=3) in float a_tex;
layout(location=4) in float a_mode;

uniform mat4 u_proj;

out vec2 v_uv;
out vec4 v_color;
flat out int v_tex;
flat out int v_mode;

void main() {
    gl_Position = u_proj * vec4(a_pos, 0.0, 1.0);
    v_uv = a_uv;
    v_color = a_color;
    v_tex = int(a_tex);
    v_mode = int(a_mode);
}
`

@(private)
FRAGMENT_SRC :: `#version 330 core
in vec2 v_uv;
in vec4 v_color;
flat in int v_tex;
flat in int v_mode;

uniform sampler2D u_textures[16];

out vec4 frag;

void main() {
    vec4 sampled = texture(u_textures[v_tex], v_uv);
    if (v_mode == 1) {
        // glyph: red channel is coverage -> use as alpha
        frag = vec4(v_color.rgb, v_color.a * sampled.r);
    } else {
        frag = sampled * v_color;
    }
}
`
