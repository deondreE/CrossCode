package engine

@(private)
_g_hot_id: u64
@(private)
_g_active_id: u64

@(private)
_hash_string :: proc(s: string) -> u64 {
	h: u64 = 0xcbf29ce484222325
	for c in s {
		h ~= u64(c)
		h *= 0x100000001b3
	}
	return h
}

widget_id :: proc(label: string) -> u64 {
	return _hash_string(label)
}

set_hot :: proc(id: u64) { _g_hot_id = id }
set_active :: proc(id: u64) { _g_active_id = id }
clear_active :: proc() { _g_active_id = 0 }
is_active :: proc(id: u64) -> bool { return _g_active_id == id }
is_hot :: proc(id: u64) -> bool { return _g_hot_id == id }
