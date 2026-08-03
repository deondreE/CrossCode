package engine

MAX_SCROLL_GROUPS :: 32

ScrollGroupState :: struct {
	id: u64,
	offset_y: f32,
	content_h_prev: f32,
}

@(private)
_g_scroll_states: [MAX_SCROLL_GROUPS]ScrollGroupState
@(private)
_g_scroll_count: int

@(private)
_get_scroll_state :: proc(id: u64) -> ^ScrollGroupState {
	for i in 0..< _g_scroll_count {
		if _g_scroll_states[i].id == id do return &_g_scroll_states[i]
	}
	if _g_scroll_count >= MAX_SCROLL_GROUPS {
		panic("UI: too many small groups")
	}
	_g_scroll_states[_g_scroll_count] = {id = id}
	_g_scroll_count += 1
	return &_g_scroll_states[_g_scroll_count - 1]
}
