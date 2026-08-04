use gl::types::*;
use memoffset::offset_of;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr;

pub const MAX_QUADS: usize = 10_000;
pub const VERTS_PER_Q: usize = 4;
pub const INDS_PER_Q: usize = 6;
pub const MAX_VERTS: usize = MAX_QUADS * VERTS_PER_Q;
pub const MAX_INDS: usize = MAX_QUADS * INDS_PER_Q;
pub const MAX_TEXTURES: usize = 16;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub tex_slot: f32,
    pub mode: f32,
}

const VERTEX_SRC: &str = r"#version 460 core
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
";

const FRAGMENT_SRC: &str = r"#version 460 core
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
";

pub struct Renderer {
    vao: GLuint,
    vbo: GLuint,
    ibo: GLuint,
    shader: GLuint,

    vertices: Vec<Vertex>,
    vertex_count: usize,
    index_count: usize,
    textures: [GLuint; MAX_TEXTURES],
    texture_count: usize,

    white_tex: GLuint, // 1x1 white for solid-color quads
    proj: [f32; 16],
    pub screen_h: i32,

    scissor_stack: Vec<(i32, i32, i32, i32)>,
}

impl Renderer {
    pub fn new(screen_w: i32, screen_h: i32) -> Self {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        let (vao, vbo, ibo) = Self::setup_buffers();
        let shader = Self::setup_shader();
        let white_tex = Self::setup_white_texture();

        let mut r = Renderer {
            vao: vao,
            vbo: vbo,
            ibo: ibo,
            shader: shader,
            vertices: vec![Vertex::default(); MAX_VERTS],
            vertex_count: 0,
            index_count: 0,
            textures: [0; MAX_TEXTURES],
            texture_count: 0,
            white_tex,
            proj: [0.0; 16],
            screen_h,
            scissor_stack: Vec::with_capacity(32),
        };
        r.resize(screen_w, screen_h);
        r
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        unsafe {
            gl::Viewport(0, 0, w, h);
        }
        self.proj = ortho(0.0, w as f32, h as f32, 0.0, -1.0, 1.0);
        self.screen_h = h;
    }

    pub fn begin_frame(&mut self) {
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.12, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        self.scissor_stack.clear();
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
        self.begin_batch();
    }

    pub fn end_frame(&mut self) {
        self.flush();
    }

    pub fn flush_now(&mut self) {
        self.flush();
        self.begin_batch();
    }

    pub fn draw_rect(&mut self, pos: [f32; 2], size: [f32; 2], color: [f32; 4]) {
        let tex = self.white_tex;
        self.push_quad(pos, size, [0.0, 0.0], [1.0, 1.0], color, tex, 0.0);
    }

    pub fn draw_sub_image(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv_min: [f32; 2],
        uv_max: [f32; 2],
        tex: GLuint,
        tint: [f32; 4],
    ) {
        self.push_quad(pos, size, uv_min, uv_max, tint, tex, 0.0);
    }

    pub fn draw_glyph(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv_min: [f32; 2],
        uv_max: [f32; 2],
        tex: GLuint,
        color: [f32; 4],
    ) {
        self.push_quad(pos, size, uv_min, uv_max, color, tex, 1.0);
    }

    fn begin_batch(&mut self) {
        self.vertex_count = 0;
        self.index_count = 0;
        self.texture_count = 0;
    }

    fn get_texture_slot(&mut self, tex: GLuint) -> f32 {
        for i in 0..self.texture_count {
            if self.textures[i] == tex {
                return i as f32;
            }
        }
        let slot = self.texture_count;
        self.textures[slot] = tex;
        self.texture_count += 1;
        slot as f32
    }

    fn push_quad(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv_min: [f32; 2],
        uv_max: [f32; 2],
        color: [f32; 4],
        tex: GLuint,
        mode: f32,
    ) {
        if self.index_count >= MAX_INDS || self.texture_count >= MAX_TEXTURES {
            self.flush();
            self.begin_batch();
        }

        let slot = self.get_texture_slot(tex);
        let (x, y) = (pos[0], pos[1]);
        let (w, h) = (size[0], size[1]);

        let i = self.vertex_count;
        self.vertices[i + 0] = Vertex {
            pos: [x, y],
            uv: [uv_min[0], uv_min[1]],
            color,
            tex_slot: slot,
            mode,
        };
        self.vertices[i + 1] = Vertex {
            pos: [x + w, y],
            uv: [uv_max[0], uv_min[1]],
            color,
            tex_slot: slot,
            mode,
        };
        self.vertices[i + 2] = Vertex {
            pos: [x + w, y + h],
            uv: [uv_max[0], uv_max[1]],
            color,
            tex_slot: slot,
            mode,
        };

        self.vertices[i + 3] = Vertex {
            pos: [x, y + h],
            uv: [uv_min[0], uv_max[1]],
            color,
            tex_slot: slot,
            mode,
        };

        self.vertex_count += 4;
        self.index_count += 6;
    }

    /// Pushes a new clip rect, intersected with whatever's currently on top of
    /// the stack. `pos`/`size` are in the same top-left, y-down UI space as
    /// everything else. Flushes the current batch first since scissor state
    /// applies per-draw-call, not per-vertex.
    pub fn push_scissor(&mut self, pos: [f32; 2], size: [f32; 2]) {
        self.flush_now();

        let x0 = pos[0];
        let y0 = pos[1];
        let x1 = pos[0] + size[0];
        let y1 = pos[0] + size[1];

        // UI Space is y-down from the top; GL scissor space is y-up from
        // bottom, so flip using screen_h
        let gl_x = x0;
        let gl_y = self.screen_h as f32 - y1;
        let gl_w = (x1 - x0).max(0.0);
        let gl_h = (y1 - y0).max(0.0);

        let mut rect = (
            gl_x.round() as i32,
            gl_y.round() as i32,
            gl_w.round() as i32,
            gl_h.round() as i32,
        );

        if let Some(&(px, py, pw, ph)) = self.scissor_stack.last() {
            let nx = rect.0.max(px);
            let ny = rect.1.max(py);
            let nx2 = (rect.0 + rect.2).min(px + pw);
            let ny2 = (rect.1 + rect.3).min(py + ph);
            rect = (nx, ny, (nx2 - nx).max(0), (ny2 - ny).max(0));
        }

        self.scissor_stack.push(rect);
        self.apply_scissor();
    }

    pub fn pop_scissor(&mut self) {
        self.flush_now();
        self.scissor_stack.pop();
        self.apply_scissor();
    }

    pub fn apply_scissor(&self) {
        unsafe {
            if let Some(&(x, y, w, h)) = self.scissor_stack.last() {
                gl::Enable(gl::SCISSOR_TEST);
                gl::Scissor(x, y, w.max(0), h.max(0));
            } else {
                gl::Disable(gl::SCISSOR_TEST);
            }
        }
    }

    fn setup_buffers() -> (GLuint, GLuint, GLuint) {
        unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (MAX_VERTS * size_of::<Vertex>()) as isize,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // static index buffer, 6 indices per quad
            let mut indices = vec![0u32; MAX_INDS];
            let mut offset: u32 = 0;
            let mut i = 0;
            while i < MAX_INDS {
                indices[i + 0] = offset + 0;
                indices[i + 1] = offset + 1;
                indices[i + 2] = offset + 2;
                indices[i + 3] = offset + 2;
                indices[i + 4] = offset + 3;
                indices[i + 5] = offset + 0;
                offset += 4;
                i += 6;
            }

            let mut ibo = 0;
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * size_of::<u32>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = size_of::<Vertex>() as i32;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, pos) as *const _,
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, uv) as *const _,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, color) as *const _,
            );
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, tex_slot) as *const _,
            );
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(
                4,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset_of!(Vertex, mode) as *const _,
            );

            (vao, vbo, ibo)
        }
    }

    fn compile_shader(src: &str, kind: GLenum) -> GLuint {
        unsafe {
            let shader = gl::CreateShader(kind);
            let c_src = CString::new(src).unwrap();
            gl::ShaderSource(shader, 1, &c_src.as_ptr(), ptr::null());

            gl::CompileShader(shader);
            let mut ok = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut ok);
            if ok == 0 {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = vec![0u8; len as usize];
                gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);
                let message = String::from_utf8_lossy(&buf);
                eprintln!("SHADER COMPILE ERROR:\n{}", message);
                panic!("[renderer] shader compile error: {}", message);
            }

            shader
        }
    }

    fn setup_shader() -> GLuint {
        unsafe {
            let vs = Self::compile_shader(VERTEX_SRC, gl::VERTEX_SHADER);
            let fs = Self::compile_shader(FRAGMENT_SRC, gl::FRAGMENT_SHADER);

            let prog = gl::CreateProgram();
            gl::AttachShader(prog, vs);
            gl::AttachShader(prog, fs);
            gl::LinkProgram(prog);

            let mut ok = 0;
            gl::GetProgramiv(prog, gl::LINK_STATUS, &mut ok);
            if ok == 0 {
                let mut len = 0;
                gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = vec![0u8; len as usize];
                gl::GetProgramInfoLog(prog, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);
                panic!(
                    "[renderer] program link error: {}",
                    String::from_utf8_lossy(&buf)
                );
            }
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            gl::UseProgram(prog);
            let samplers: Vec<i32> = (0..MAX_TEXTURES as i32).collect();
            let name = CString::new("u_textures").unwrap();
            let loc = gl::GetUniformLocation(prog, name.as_ptr());
            gl::Uniform1iv(loc, MAX_TEXTURES as i32, samplers.as_ptr());

            prog
        }
    }

    fn setup_white_texture() -> GLuint {
        unsafe {
            let white: u32 = 0xffffffff;
            let mut tex = 0;
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                1,
                1,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                &white as *const _ as *const _,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            tex
        }
    }

    fn flush(&mut self) {
        if self.index_count == 0 {
            return;
        }
        unsafe {
            gl::UseProgram(self.shader);
            let name = CString::new("u_proj").unwrap();
            let loc = gl::GetUniformLocation(self.shader, name.as_ptr());
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, self.proj.as_ptr());

            for i in 0..self.texture_count {
                gl::ActiveTexture(gl::TEXTURE0 + i as u32);
                gl::BindTexture(gl::TEXTURE_2D, self.textures[i]);
            }

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.vertex_count * size_of::<Vertex>()) as isize,
                self.vertices.as_ptr() as *const _,
            );
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteProgram(self.shader);
            gl::DeleteTextures(1, &self.white_tex);
        }
    }
}

fn ortho(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> [f32; 16] {
    // column-major, matches u_proj expecting mat4 uniform layout
    [
        2.0 / (r - l),
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / (t - b),
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / (f - n),
        0.0,
        -(r + l) / (r - l),
        -(t + b) / (t - b),
        -(f + n) / (f - n),
        1.0,
    ]
}
