package engine
import "core:math/linalg"

// Layout represents the layout of a UI element.
LayoutType :: enum {
	Vertical,
	Horizontal,
	Grid
}

// LayoutOptions represents the options for a layout.
LayoutOptions :: struct {
	type: LayoutType,
	spacing: f32
}

// Layout represents the layout of a UI element.
Layout :: struct {
	widgets: [dynamic]^Widget,
	options: LayoutOptions,
	bounds: linalg.Vector2f32,
	cursor: linalg.Vector2f32,
	active: bool,
}

@(private)
_g_layout_storage: Layout
@(private)
_g_current_layout: ^Layout

start_layout :: proc(type: LayoutType, max_w, max_h: f32, start_pos: linalg.Vector2f32 = {10.0, 10.0}) -> ^Layout {
	if _g_layout_storage.widgets == nil {
		_g_layout_storage.widgets = make([dynamic]^Widget)
	}

	clear(&_g_layout_storage.widgets)

	_g_layout_storage.options = LayoutOptions{
		type = type,
		spacing = 10.0,
	}
	_g_layout_storage.bounds = linalg.Vector2f32{max_w, max_h}
	_g_layout_storage.cursor = start_pos
	_g_layout_storage.active = true

	_g_current_layout = &_g_layout_storage
	return _g_current_layout
}

append_widget :: proc(l: ^Layout, widget: ^Widget) {
	if l != nil {
		append(&l.widgets, widget)
	}
}

end_layout :: proc(l: ^Layout) {
	if l == nil do return

	_update_layout(l)
	for w in l.widgets {
		// draw_rect()
		free(w)
	}

	l.active = false
	_g_current_layout = nil
}

@(private)
_advance_layout :: proc(size: linalg.Vector2f32) {
	// Advance cursor to the next widget manually
	if _g_current_layout.options.type == .Vertical {
		_g_current_layout.cursor.y += size.y + _g_current_layout.options.spacing
	} else {
		_g_current_layout.cursor.x += size.x + _g_current_layout.options.spacing
	}
}

@(private)
_update_layout :: proc(l: ^Layout) {
	l.cursor = {10, 10}
	// Update the layout of the widgets in the current layout.
	for widget in l.widgets {
        widget.position = l.cursor

        if l.options.type == .Horizontal {
            l.cursor.x += widget.size.x + l.options.spacing
        } else if l.options.type == .Vertical {
            l.cursor.y += widget.size.y + l.options.spacing
        }
    }
}
