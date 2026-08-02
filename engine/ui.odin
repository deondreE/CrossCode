package engine
import "core:math/linalg"
import "vendor:glfw"

InputState :: struct {
	mouse_pos: linalg.Vector2f32,
	mouse_down: bool,
	mouse_clicked: bool
}

@(private)
_g_input: InputState

Color :: struct {
	r, g, b, a: f32
}

Widget :: struct {
	position: linalg.Vector2f32,
	size: linalg.Vector2f32,
	parent: ^Widget,
	children: [dynamic]^Widget,
}


@(private)
_update_input :: proc(window: ^Window) {
	x, y := glfw.GetCursorPos(window.handle)
	_g_input.mouse_pos = {f32(x), f32(y)}

	is_down := glfw.GetMouseButton(window.handle, glfw.MOUSE_BUTTON_LEFT) == glfw.PRESS;
	_g_input.mouse_clicked = is_down && !_g_input.mouse_down
	_g_input.mouse_down = is_down
}

// Returns true if the point is inside the rectangle defined by rect_pos and rect_size.
@(private)
point_is_rect :: proc(p: linalg.Vector2f32, rect_pos, rect_size: linalg.Vector2f32) -> bool {
	return p.x >= rect_pos.x &&
		p.x <= rect_pos.x + rect_size.x &&
		p.y >= rect_pos.y &&
		p.y <= rect_pos.y + rect_size.y
}

// @Todo: Wrapped text
label :: proc(font: ^Font, text: string, color: Color = {1, 1, 1, 1}) {
	if _g_current_layout == nil do return

	size := measure_text(font, text)
	pos := _g_current_layout.cursor

	draw_text(font, text, pos, {color.r, color.g, color.b, color.a})

	_advance_layout(size)
}

// Checkbox element
checkbox :: proc(font: ^Font, label_text: string, checked: ^bool) -> bool {
	// @Todo: Every value needs to be customizable.
	if _g_current_layout == nil do return false

	size := linalg.Vector2f32{25, 25}
	pos := _g_current_layout.cursor

	is_hovered := point_is_rect(_g_input.mouse_pos, pos, size)
	toggled := false

	if is_hovered && _g_input.mouse_clicked {
		checked^ = !checked^
		toggled = true
	}

	// Draw Box
	bg_col := [4]f32{0.2, 0.2, 0.25, 1.0}
	if is_hovered do bg_col = {0.3, 0.3, 0.4, 1.0}
	draw_rect(pos, size, bg_col)

	// Draw "X" or checkmark if true
	if checked^ {
		draw_rect(pos + 5, size - 10, { 0.1, 0.8, 0.2, 1.0 })
	}

	// Draw label text next to checkbox
	text_pos := pos + {size.x+ 10, 5}
	draw_text(font, label_text, text_pos, {1, 1, 1, 1})

	t_size := measure_text(font, label_text)
	total_w := size.x + 10 + t_size.x

	_advance_layout(size)

	return toggled
}

// Returns true if the button is clicked, false otherwise.
button :: proc(font: ^Font, label: string, size: linalg.Vector2f32 = {100, 40}, bg_color: Color = {0.2, 0.2, 0.25, 1.0}) -> bool {
	if _g_current_layout == nil do return false

	pos := _g_current_layout.cursor

	w := new(Widget)
	w.size = size
	w.position = pos
	append_widget(_g_current_layout, w)

	is_hovered := point_is_rect(_g_input.mouse_pos, w.position, w.size)
	is_clicked := is_hovered && _g_input.mouse_clicked

	l_color := bg_color
	if is_hovered {
		l_color = {0.3, 0.3, 0.4, 1.0}
		if _g_input.mouse_down do l_color = {0.1, 0.1, 0.15, 1.0}
	}
	draw_rect(w.position, size, {l_color.r, l_color.g, l_color.b, l_color.a})

	// Center text to button
	text_size := measure_text(font, label)
	text_pos := w.position + (w.size / 2.0) - (text_size / 2.0)
	text_pos.y -= font.pixel_height / 2
	draw_text(font, label, text_pos, {1, 1, 1, 1})

	_advance_layout(size)

	return is_clicked
}

slider :: proc(val: ^f32, min, max: f32, width: f32 = 200) -> bool {
	if _g_current_layout == nil do return false

	size: linalg.Vector2f32 = {width, 20}
	pos := _g_current_layout.cursor

	is_hovered := point_is_rect(_g_input.mouse_pos, pos, size)
	changed := false

	// Handle dragging
	if _g_input.mouse_down && is_hovered {
		relative_x := _g_input.mouse_pos.x - pos.x
		t := clamp(relative_x / width, 0.0, 1.0)
		val^ = min + (max - min) * t
		changed =  true
	}

	// draw background track
	draw_rect(pos, size, {0.15, 0.15, 0.15, 1.0})

	// Draw Handle
	handle_x := (val^ - min) / (max - min) * width
	handle_size: linalg.Vector2f32 = {10, 20}
	draw_rect(pos + {handle_x - 5, 0}, handle_size, {0.4, 0.6, 0.9, 1.0})

	_advance_layout(size)

	return changed
}

spacer :: proc(amount: f32) {
	if _g_current_layout == nil do return
	if _g_current_layout.options.type == .Vertical {
		_g_current_layout.cursor.y += amount
	} else {
		_g_current_layout.cursor.x += amount
	}
}
