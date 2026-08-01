#+build windows
package engine

import "core:fmt"
import "core:os"
import gl "vendor:OpenGL"
import stbtt "vendor:stb/truetype"

FIRST_CHAR :: 32
NUM_CHARS  :: 95 // 32..126

Font :: struct {
    atlas_tex:    u32,
    atlas_w:      i32,
    atlas_h:      i32,
    chars:        [NUM_CHARS]stbtt.bakedchar,
    pixel_height: f32,
    ttf_data:     []byte, // kept alive if you want later re-bake
}

load_font :: proc(path: string, pixel_height: f32) -> (Font, bool) {
    data, ok := os.read_entire_file_from_path(path, context.allocator)
    if ok != .SUCCESS {
        fmt.eprintln("[font] failed to read", path)
        return {}, false
    }

    font: Font
    font.pixel_height = pixel_height
    font.ttf_data = data

    ATLAS :: 512
    font.atlas_w = ATLAS
    font.atlas_h = ATLAS

    bitmap := make([]byte, ATLAS * ATLAS)
    defer delete(bitmap)

    res := stbtt.BakeFontBitmap(
        raw_data(data), 0, pixel_height,
        raw_data(bitmap), ATLAS, ATLAS,
        FIRST_CHAR, NUM_CHARS, &font.chars[0],
    )
    if res == 0 {
        fmt.eprintln("[font] atlas too small for", path)
        delete(data)
        return {}, false
    }

    gl.GenTextures(1, &font.atlas_tex)
    gl.BindTexture(gl.TEXTURE_2D, font.atlas_tex)
    gl.PixelStorei(gl.UNPACK_ALIGNMENT, 1)
    gl.TexImage2D(gl.TEXTURE_2D, 0, gl.R8, ATLAS, ATLAS, 0, gl.RED, gl.UNSIGNED_BYTE, raw_data(bitmap))
    gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR)
    gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR)
    gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE)
    gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE)

    return font, true
}

destroy_font :: proc(font: ^Font) {
    if font.atlas_tex != 0 do gl.DeleteTextures(1, &font.atlas_tex)
    delete(font.ttf_data)
    font^ = {}
}

// pos = top-left-ish start; GetBakedQuad advances the pen.
draw_text :: proc(font: ^Font, text: string, pos: [2]f32, color: [4]f32) {
    x := pos.x
    y := pos.y + font.pixel_height // baseline offset so top-left aligns

    for c in text {
        if c < FIRST_CHAR || c >= FIRST_CHAR + NUM_CHARS do continue
        q: stbtt.aligned_quad
        stbtt.GetBakedQuad(
            &font.chars[0], font.atlas_w, font.atlas_h,
            i32(c) - FIRST_CHAR,
            &x, &y, &q,
            true, // fill_rule: use integer pixel positions
        )

        draw_glyph(
            {q.x0, q.y0}, {q.x1 - q.x0, q.y1 - q.y0},
            {q.s0, q.t0}, {q.s1, q.t1},
            font.atlas_tex, color,
        )
    }
}

measure_text :: proc(font: ^Font, text: string) -> [2]f32 {
    x, y: f32 = 0, 0
    for c in text {
        if c < FIRST_CHAR || c >= FIRST_CHAR + NUM_CHARS do continue
        q: stbtt.aligned_quad
        stbtt.GetBakedQuad(
            &font.chars[0], font.atlas_w, font.atlas_h,
            i32(c) - FIRST_CHAR, &x, &y, &q, true,
        )
    }
    return {x, font.pixel_height}
}
