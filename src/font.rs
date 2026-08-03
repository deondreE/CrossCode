use crate::renderer::Renderer;
use fontdue::{Font as FontdueFont, FontSettings};
use gl::types::*;
use std::collections::HashMap;

const FIRST_CHAR: u32 = 32;
const NUM_CHARS: u32 = 95;
const ATLAS: usize = 512;

#[derive(Clone, Copy, Default)]
struct BakedChar {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    xoff: f32,
    yoff: f32,
    xadvance: f32,
}

pub struct Font {
    pub atlas_tex: GLuint,
    pub atlas_w: i32,
    pub atlas_h: i32,
    pub pixel_height: f32,
    chars: HashMap<u32, BakedChar>,
}

impl Font {
    pub fn load(path: &str, pixel_height: f32) -> Result<Font, String> {
        let data = std::fs::read(path).map_err(|e| format!("[font] failed to read {path}: {e}"))?;
        let face = FontdueFont::from_bytes(data, FontSettings::default())
            .map_err(|e| format!("[font] fontdue parse error: {e}"))?;

        let mut atlas = vec![0u8; ATLAS * ATLAS];
        let mut chars = HashMap::new();

        let mut cursor_x: usize = 0;
        let mut cursor_y: usize = 0;
        let mut row_h: usize = 0;

        for code in FIRST_CHAR..(FIRST_CHAR + NUM_CHARS) {
            let ch = char::from_u32(code).unwrap();
            let (metrics, bitmap) = face.rasterize(ch, pixel_height);

            if cursor_x + metrics.width > ATLAS {
                cursor_x = 0;
                cursor_y += row_h + 1;
                row_h = 0;
            }
            if cursor_y + metrics.height > ATLAS {
                return Err(format!("[font] atlas too small for {path}"));
            }

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let src = row * metrics.width + col;
                    let dst = (cursor_y + row) * ATLAS + (cursor_x + col);
                    atlas[dst] = bitmap[src];
                }
            }

            // Vertical placement relative to baseline (pen.y), screen Y grows down:
            // fontdue's ymin is the glyph bbox's distance above the baseline (can be
            // negative for descenders like 'g'); top of the bitmap sits at
            // -(ymin + height) relative to the baseline.
            let yoff = -(metrics.ymin as f32 + metrics.height as f32);

            chars.insert(
                code,
                BakedChar {
                    x0: cursor_x as f32,
                    y0: cursor_y as f32,
                    x1: (cursor_x + metrics.width) as f32,
                    y1: (cursor_y + metrics.height) as f32,
                    xoff: metrics.xmin as f32,
                    yoff,
                    xadvance: metrics.advance_width,
                },
            );

            cursor_x += metrics.width + 1;
            row_h = row_h.max(metrics.height);
        }

        let atlas_tex = unsafe {
            let mut tex = 0;
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R8 as i32,
                ATLAS as i32,
                ATLAS as i32,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                atlas.as_ptr() as *const _,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            tex
        };

        Ok(Font {
            atlas_tex,
            atlas_w: ATLAS as i32,
            atlas_h: ATLAS as i32,
            pixel_height,
            chars,
        })
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.atlas_tex) };
    }
}

/// pos = top-left-ish start; baseline is pos.y + pixel_height, matching the
/// original stbtt-based convention.
pub fn draw_text(renderer: &mut Renderer, font: &Font, text: &str, pos: [f32; 2], color: [f32; 4]) {
    let mut x = pos[0];
    let y_baseline = pos[1] + font.pixel_height;

    for c in text.chars() {
        let code = c as u32;
        if code < FIRST_CHAR || code >= FIRST_CHAR + NUM_CHARS {
            continue;
        }
        if let Some(bc) = font.chars.get(&code) {
            let gw = bc.x1 - bc.x0;
            let gh = bc.y1 - bc.y0;
            let gx = x + bc.xoff;
            let gy = y_baseline + bc.yoff;

            renderer.draw_glyph(
                [gx, gy],
                [gw, gh],
                [bc.x0 / font.atlas_w as f32, bc.y0 / font.atlas_h as f32],
                [bc.x1 / font.atlas_w as f32, bc.y1 / font.atlas_h as f32],
                font.atlas_tex,
                color,
            );
            x += bc.xadvance;
        }
    }
}

/// Mirrors the original measure_text: returns total advance width and a
/// fixed single-line height (font.pixel_height) — not a true multi-line bound.
pub fn measure_text(font: &Font, text: &str) -> [f32; 2] {
    let mut x = 0.0f32;
    for c in text.chars() {
        let code = c as u32;
        if code < FIRST_CHAR || code >= FIRST_CHAR + NUM_CHARS {
            continue;
        }
        if let Some(bc) = font.chars.get(&code) {
            x += bc.xadvance;
        }
    }
    [x, font.pixel_height]
}
