package engine
import "core:math/linalg"
import "core:mem"

MAX_LAYOUT_STACK :: 16

UI_ARENA_SIZE :: 1024 * 1024

UIState :: struct {
	arena: mem.Arena,
	allocator: mem.Allocator,
	layout_stack: [MAX_LAYOUT_STACK]Layout,
	stack_index: int
}

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
	outer_size: linalg.Vector2f32,
	padding: f32,
	origin: linalg.Vector2f32,
	cursor: linalg.Vector2f32,
	active: bool,
}

LayoutStack :: struct {
	layouts: [MAX_LAYOUT_STACK]Layout,
	index: int
}

@(private)
_g_ui: UIState
@(private)
_g_current_layout: ^Layout

start_layout :: proc(type: LayoutType, max_w, max_h: f32, start_pos: linalg.Vector2f32 = {10.0, 10.0}, spacing: f32 = 10.0, padding: f32 = 0) -> ^Layout {
	if _g_ui.stack_index >= MAX_LAYOUT_STACK {
		panic("UI: Layout stack overflow!")
	}
	l := &_g_ui.layout_stack[_g_ui.stack_index]
	_g_ui.stack_index += 1

	l.widgets = make([dynamic]^Widget, 0, 8, _g_ui.allocator)

	l.options = LayoutOptions{
		type,
		spacing,
	}
	l.padding = padding // content size, as passed in
	l.bounds = linalg.Vector2f32{max_w, max_h}
	l.outer_size = l.bounds + {padding * 2, padding * 2}
	l.active = true

	l.cursor = start_pos
	l.origin = l.cursor
	_g_current_layout = l
	return _g_current_layout
}

append_widget :: proc(l: ^Layout, widget: ^Widget) {
	if l != nil {
		append(&l.widgets, widget)
	}
}

end_layout :: proc() {
    if _g_ui.stack_index == 0 do return

    current := &_g_ui.layout_stack[_g_ui.stack_index - 1]
    _update_layout(current) // Calculate final positions

    _g_ui.stack_index -= 1
    if _g_ui.stack_index > 0 {
        _g_current_layout = &_g_ui.layout_stack[_g_ui.stack_index - 1]
        _advance_layout(current.outer_size)
    } else {
        _g_current_layout = nil
    }
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
	l.cursor = l.origin
	for widget in l.widgets {
        widget.position = l.cursor

        if l.options.type == .Horizontal {
            l.cursor.x += widget.size.x + l.options.spacing
        } else if l.options.type == .Vertical {
            l.cursor.y += widget.size.y + l.options.spacing
        }
    }
}

@(private)
_ui_init :: proc() {
	data := make([]byte, UI_ARENA_SIZE)
	mem.arena_init(&_g_ui.arena, data)
	_g_ui.allocator = mem.arena_allocator(&_g_ui.arena)
}

@(private)
_ui_begin_frame :: proc() {
	free_all(_g_ui.allocator)
	_g_ui.stack_index = 0
	_g_current_layout = nil
	_g_hot_id = 0
}
