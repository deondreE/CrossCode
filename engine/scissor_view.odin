package engine
import gl "vendor:OpenGL"
import "core:math/linalg"

MAX_SCISSOR_STACK :: 16


@(private)
_g_scissor_stack: [MAX_SCISSOR_STACK]struct { pos, size: linalg.Vector2f32}
@(private)
_g_scissor_index: int

push_scissor :: proc(pos, size: linalg.Vector2f32) {
	if _g_scissor_index >= MAX_SCISSOR_STACK {
		panic("UI: scissor stack overflow")
	}

	// Intersect with the current top of stack so nested clips can only shrink, never grow
	final_pos := pos
	final_size := size
	if _g_scissor_index > 0 {
		parent := _g_scissor_stack[_g_scissor_index - 1]
		x0 := max(pos.x, parent.pos.x)
		y0 := max(pos.y, parent.pos.y)
		x1 := min(pos.x + size.x, parent.pos.x + parent.size.x)
		y1 := min(pos.y + size.y, parent.pos.y + parent.size.y)
		final_pos = {x0, y0}
		final_size = {max(0, x1 - x0), max(0, y1 - y0)}
	}

	_g_scissor_stack[_g_scissor_index] = {final_pos, final_size}
	_g_scissor_index += 1

	_render_flush_now()

	gl.Enable(gl.SCISSOR_TEST)
	_apply_scissor(final_pos, final_size)
}

pop_scissor :: proc() {
	if _g_scissor_index == 0 do return

	_render_flush_now()

	_g_scissor_index -= 1
	if _g_scissor_index == 0 {
		gl.Disable(gl.SCISSOR_TEST)
	} else {
		top := _g_scissor_stack[_g_scissor_index - 1]
		_apply_scissor(top.pos, top.size)
	}
}

@(private)
_apply_scissor :: proc(pos, size: linalg.Vector2f32) {
	gl.Scissor(i32(pos.x), g_renderer.screen_h - i32(pos.y + size.y), i32(size.x), i32(size.y))
}
