#+build windows
package engine

import "core:fmt"
import gl "vendor:OpenGL"
import glfw "vendor:glfw"

MAX_QUADS :: 10000
VERTS_PER_Q :: 4
INDS_PER_Q :: 6
MAX_VERTS :: MAX_QUADS * VERTS_PER_Q
MAX_INDS :: MAX_QUADS * INDS_PER_Q
MAX_TEXTURES :: 16

Vertex :: struct {
    pos: [2]f32,
    uv: [2]f32,
    color: [4]f32,
    tex_slot: f32,
    mode: f32,
}

Renderer :: struct {
    vao, vbo, ibo: u32,
    shader: u32,

    vertices: [MAX_VERTS]Vertex,
    vertex_count: int,
    index_count: int,
    textures: [MAX_TEXTURES]u32,
    texture_count: int,

    white_tex: u32, // 1x1 white for solid-color quads
    proj: matrix[4, 4]f32,
    screen_h: i32
}

@(private) g_renderer: Renderer

render_init :: proc(screen_w, screen_h: i32) -> bool {
    gl.load_up_to(4,6, glfw.gl_set_proc_address)

    gl.Enable(gl.BLEND)
    gl.BlendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA)

    _setup_buffers()
    if !_setup_shader() do return false
    _setup_white_texture()

    render_resize(screen_w, screen_h)
    fmt.println("[renderer] OpenGL 2D renderer ready")
   return true
}

render_resize :: proc(w, h: i32){
    gl.Viewport(0, 0, w, h)
    g_renderer.proj = _ortho(0, f32(w), f32(h), 0, -1, 1)
    g_renderer.screen_h = h
}

render_begin_frame :: proc() {
    gl.ClearColor(0.1, 0.1, 0.12, 1.0)
    gl.Clear(gl.COLOR_BUFFER_BIT)
    _begin_batch()
}

render_end_frame :: proc() {
    _flush()
}

render_shutdown :: proc() {
    gl.DeleteVertexArrays(1, &g_renderer.vao)
    gl.DeleteBuffers(1, &g_renderer.vbo)
    gl.DeleteBuffers(1, &g_renderer.ibo)
    gl.DeleteProgram(g_renderer.shader)
    gl.DeleteTextures(1, &g_renderer.white_tex)
}

draw_rect :: proc(pos, size: [2]f32, color: [4]f32) {
    _push_quad(pos, size, {0, 0}, {1, 1}, color, g_renderer.white_tex, 0)
}

draw_sub_image :: proc(
    pos, size: [2]f32,
    uv_min, uv_max: [2]f32,
    tex: u32,
    tint := [4]f32{1, 1, 1, 1},
) {
    _push_quad(pos, size, uv_min, uv_max, tint, tex, 0)
}

draw_glyph :: proc(
    pos, size, uv_min, uv_max: [2]f32,
    tex: u32,
    color: [4]f32,
) {
    _push_quad(pos, size, uv_min, uv_max, color, tex, 1)
}

@(private)
_begin_batch :: proc() {
    g_renderer.vertex_count = 0
    g_renderer.index_count =  0
    g_renderer.texture_count = 0
}

@(private)
_get_texture_slot :: proc(tex: u32) -> f32 {
    for i in 0 ..< g_renderer.texture_count {
        if g_renderer.textures[i] == tex {
            return f32(i)
        }
    }

    slot := g_renderer.texture_count
    g_renderer.textures[slot] = tex
    g_renderer.texture_count += 1
    return f32(slot)
}

@(private)
_push_quad :: proc(
    pos, size, uv_min, uv_max: [2]f32,
    color: [4]f32,
    tex: u32,
    mode: f32,
) {
    if g_renderer.index_count >= MAX_INDS ||
       g_renderer.texture_count >= MAX_TEXTURES {
        _flush()
        _begin_batch()
    }

    slot := _get_texture_slot(tex)
    x, y := pos.x, pos.y
    w, h := size.x, size.y

    i := g_renderer.vertex_count
    v := &g_renderer.vertices
    v[i + 0] = {{x,     y},     {uv_min.x, uv_min.y}, color, slot, mode}
    v[i + 1] = {{x + w, y},     {uv_max.x, uv_min.y}, color, slot, mode}
    v[i + 2] = {{x + w, y + h}, {uv_max.x, uv_max.y}, color, slot, mode}
    v[i + 3] = {{x,     y + h}, {uv_min.x, uv_max.y}, color, slot, mode}

    g_renderer.vertex_count += 4
    g_renderer.index_count  += 6
}

@(private)
_setup_buffers :: proc() {
gl.GenVertexArrays(1, &g_renderer.vao)
    gl.BindVertexArray(g_renderer.vao)

    gl.GenBuffers(1, &g_renderer.vbo)
    gl.BindBuffer(gl.ARRAY_BUFFER, g_renderer.vbo)
    gl.BufferData(gl.ARRAY_BUFFER, MAX_VERTS * size_of(Vertex), nil, gl.DYNAMIC_DRAW)

    // static index buffer: 6 indices per quad
    indices: [MAX_INDS]u32
    offset: u32 = 0
    for i := 0; i < MAX_INDS; i += 6 {
        indices[i + 0] = offset + 0
        indices[i + 1] = offset + 1
        indices[i + 2] = offset + 2
        indices[i + 3] = offset + 2
        indices[i + 4] = offset + 3
        indices[i + 5] = offset + 0
        offset += 4
    }
    gl.GenBuffers(1, &g_renderer.ibo)
    gl.BindBuffer(gl.ELEMENT_ARRAY_BUFFER, g_renderer.ibo)
    gl.BufferData(gl.ELEMENT_ARRAY_BUFFER, size_of(indices), &indices[0], gl.STATIC_DRAW)

    stride := i32(size_of(Vertex))
    // a_pos
    gl.EnableVertexAttribArray(0)
    gl.VertexAttribPointer(0, 2, gl.FLOAT, false, stride, offset_of(Vertex, pos))
    // a_uv
    gl.EnableVertexAttribArray(1)
    gl.VertexAttribPointer(1, 2, gl.FLOAT, false, stride, offset_of(Vertex, uv))
    // a_color
    gl.EnableVertexAttribArray(2)
    gl.VertexAttribPointer(2, 4, gl.FLOAT, false, stride, offset_of(Vertex, color))
    // a_tex_slot
    gl.EnableVertexAttribArray(3)
    gl.VertexAttribPointer(3, 1, gl.FLOAT, false, stride, offset_of(Vertex, tex_slot))
    // a_mode
    gl.EnableVertexAttribArray(4)
    gl.VertexAttribPointer(4, 1, gl.FLOAT, false, stride, offset_of(Vertex, mode))
}

@(private)
_setup_shader :: proc() -> bool {
    prog, ok := gl.load_shaders_source(VERTEX_SRC, FRAGMENT_SRC)
    if !ok {
        return false
    }
    g_renderer.shader = prog

    // bind sampler array uniforms 0..15
    gl.UseProgram(prog)
    samplers: [MAX_TEXTURES]i32
    for i in 0 ..< MAX_TEXTURES do samplers[i] = i32(i)
    loc := gl.GetUniformLocation(prog, "u_textures")
    gl.Uniform1iv(loc, MAX_TEXTURES, &samplers[0])
    return true
}

@(private)
_setup_white_texture :: proc() {
    white: u32 = 0xffffffff
    gl.GenTextures(1, &g_renderer.white_tex)
    gl.BindTexture(gl.TEXTURE_2D, g_renderer.white_tex)
    gl.TexImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, &white)
    gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST)
    gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST)
}

@(private)
_flush :: proc() {
    if g_renderer.index_count == 0 do return

    gl.UseProgram(g_renderer.shader)
    loc := gl.GetUniformLocation(g_renderer.shader, "u_proj")
    gl.UniformMatrix4fv(loc, 1, false, &g_renderer.proj[0, 0])

    for i in 0 ..< g_renderer.texture_count {
        gl.ActiveTexture(gl.TEXTURE0 + u32(i))
        gl.BindTexture(gl.TEXTURE_2D, g_renderer.textures[i])
    }

    gl.BindVertexArray(g_renderer.vao)
    gl.BindBuffer(gl.ARRAY_BUFFER, g_renderer.vbo)
    gl.BufferSubData(
        gl.ARRAY_BUFFER, 0,
        g_renderer.vertex_count * size_of(Vertex),
        &g_renderer.vertices[0],
    )
    gl.DrawElements(gl.TRIANGLES, i32(g_renderer.index_count), gl.UNSIGNED_INT, nil)
}

@(private)
_render_flush_now :: proc() {
	_flush()
	_begin_batch()
}


@(private)
_ortho :: proc(l, r, b, t, n, f: f32) -> matrix[4, 4]f32 {
    return matrix[4, 4]f32{
        2 / (r - l), 0,            0,             -(r + l) / (r - l),
        0,            2 / (t - b), 0,             -(t + b) / (t - b),
        0,            0,           -2 / (f - n),  -(f + n) / (f - n),
        0,            0,           0,              1,
    }
}
