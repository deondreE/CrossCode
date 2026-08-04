use crate::font::{Font, draw_text, measure_text};
use crate::renderer::Renderer;

use std::fmt::{Arguments, Write};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

pub enum WidgetKind {
    Rect {
        color: [f32; 4],
    },
    Text {
        range: std::ops::Range<usize>,
        color: [f32; 4],
    },
    ClipPush,
    ClipPop,
}

pub struct Widget {
    pub id: u64,
    pub size: [f32; 2],
    pub pos: [f32; 2],
    pub kind: WidgetKind,
}

struct LayoutState {
    origin: [f32; 2],
    cursor: [f32; 2],
    direction: LayoutDirection,
    max_content_size: [f32; 2],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MouseState {
    pub pos: [f32; 2],
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
}

#[derive(Debug, Clone, Default)]
pub struct KeyboardState {
    pub chars: Vec<char>,
    pub backspace: bool,
    pub enter: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TextInputColors {
    pub idle: [f32; 4],
    pub focused: [f32; 4],
    pub text: [f32; 4],
    pub cursor: [f32; 4],
}

impl Default for TextInputColors {
    fn default() -> Self {
        Self {
            idle: [0.18, 0.18, 0.2, 1.0],
            focused: [0.22, 0.22, 0.26, 1.0],
            text: [1.0, 1.0, 1.0, 1.0],
            cursor: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonColors {
    pub idle: [f32; 4],
    pub hover: [f32; 4],
    pub active: [f32; 4],
    pub text: [f32; 4],
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self {
            idle: [0.25, 0.25, 0.28, 1.0],
            hover: [0.35, 0.35, 0.4, 1.0],
            active: [0.15, 0.55, 0.9, 1.0],
            text: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SliderColors {
    track: [f32; 4],
    fill: [f32; 4],
    handle: [f32; 4],
    handle_hover: [f32; 4],
    handle_active: [f32; 4],
}

impl Default for SliderColors {
    fn default() -> Self {
        Self {
            track: [0.2, 0.2, 0.22, 1.0],
            fill: [0.15, 0.55, 0.9, 1.0],
            handle: [0.8, 0.8, 0.85, 1.0],
            handle_hover: [0.9, 0.9, 0.95, 1.0],
            handle_active: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

fn point_in_rect(p: [f32; 2], pos: [f32; 2], size: [f32; 2]) -> bool {
    p[0] >= pos[0] && p[0] < pos[0] + size[0] && p[1] >= pos[1] && p[1] < pos[1] + size[1]
}

/// Eqiuvilent to the html `body` tag.
pub struct UiContext {
    pub widgets: Vec<Widget>,
    ///  render on top. Intended for popups, dropdown lists, and tooltips
    pub overlay_widgets: Vec<Widget>,
    layout_stack: Vec<LayoutState>,
    id_stack: Vec<u64>,
    pub padding: f32,
    pub gap: f32,
    pub text_buffer: String,

    mouse: MouseState,
    keyboard: KeyboardState,
    hot_id: Option<u64>,
    active_id: Option<u64>,
    focus_id: Option<u64>,
    time: f32,
}

impl UiContext {
    pub fn new(padding: f32, gap: f32) -> Self {
        Self {
            widgets: Vec::with_capacity(1000),
            overlay_widgets: Vec::with_capacity(64),
            layout_stack: Vec::with_capacity(256),
            id_stack: Vec::with_capacity(32),
            padding,
            gap,
            text_buffer: String::with_capacity(4096),
            mouse: MouseState::default(),
            keyboard: KeyboardState::default(),
            hot_id: None,
            active_id: None,
            focus_id: None,
            time: 0.0,
        }
    }

    /// Feed this frame's mouse input in before calling `begin()`
    pub fn set_mouse(&mut self, mouse: MouseState) {
        self.mouse = mouse;
    }

    pub fn set_keyboard(&mut self, keyboard: KeyboardState) {
        self.keyboard = keyboard;
    }

    pub fn advance_time(&mut self, dt: f32) {
        self.time += dt;
    }

    /// Pushes a seed onto the ID stack, scoping every widget ID created
    /// until the matching `pop_id()`. Use this around loops or reusable
    pub fn push_id<H: Hash>(&mut self, seed: H) {
        let id = self.resolve_id(seed);
        self.id_stack.push(id);
    }

    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }

    /// Combines `seed` with whatever is currently on top of the ID stack to
    /// produce a widget ID. Widget calls that take `id: impl Hash` funnel
    /// through here, so passing a plain string label "just works" as long
    /// as it's unique within its current `push_id` scope.
    fn resolve_id<H: Hash>(&self, seed: H) -> u64 {
        let mut hasher = DefaultHasher::new();
        if let Some(parent) = self.id_stack.last() {
            parent.hash(&mut hasher);
        }
        seed.hash(&mut hasher);
        hasher.finish()
    }

    /// Entry point for the frame. Sets the root layout area.
    pub fn begin(&mut self, screen_pos: [f32; 2], dir: LayoutDirection) {
        self.widgets.clear();
        self.overlay_widgets.clear();
        self.text_buffer.clear();
        self.layout_stack.clear();
        self.id_stack.clear();
        self.hot_id = None;
        if self.mouse.pressed {
            self.focus_id = None;
        }
        self.layout_stack.push(LayoutState {
            origin: screen_pos,
            cursor: [self.padding, self.padding],
            direction: dir,
            max_content_size: [0.0, 0.0],
        });
    }

    /// Pushes a new nested layout relative to the current cursor
    pub fn push_layout(&mut self, dir: LayoutDirection) {
        let parent = self.layout_stack.last().expect("No active layout");
        let new_origin = [
            parent.origin[0] + parent.cursor[0],
            parent.origin[1] + parent.cursor[1],
        ];

        self.layout_stack.push(LayoutState {
            origin: new_origin,
            cursor: [0.0, 0.0],
            direction: dir,
            max_content_size: [0.0, 0.0],
        });
    }

    /// Finalizes a nested layout and advances the parent's cursor by the child's size
    pub fn pop_layout(&mut self) {
        if self.layout_stack.len() <= 1 {
            return;
        }

        let child = self.layout_stack.pop().unwrap();
        // The size of the child container is its cursor position or max cont
        let child_size = [
            child.max_content_size[0].max(child.cursor[0]),
            child.max_content_size[1].max(child.cursor[1]),
        ];
        self.advance_parent(child_size);
    }

    pub fn rect(&mut self, id: impl Hash, size: [f32; 2], color: [f32; 4]) {
        let id = self.resolve_id(id);
        let pos = self.current_pos();
        self.widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Rect { color },
        });
        self.advance_parent(size);
    }

    pub fn overlay_rect(&mut self, id: impl Hash, size: [f32; 2], color: [f32; 4]) {
        let id = self.resolve_id(id);
        let pos = self.current_pos();
        self.overlay_widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Rect { color },
        });
    }

    pub fn label(&mut self, id: impl Hash, text: &str, font: &Font, color: [f32; 4]) {
        let id = self.resolve_id(id);
        let size = measure_text(font, text);
        let pos = self.current_pos();

        let start = self.text_buffer.len();
        self.text_buffer.push_str(text);
        let end = self.text_buffer.len();

        self.widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Text {
                range: start..end,
                color,
            },
        });
        self.advance_parent(size);
    }

    pub fn overlay_label(&mut self, id: impl Hash, text: &str, font: &Font, color: [f32; 4]) {
        let id = self.resolve_id(id);
        let size = measure_text(font, text);
        let pos = self.current_pos();

        let start = self.text_buffer.len();
        self.text_buffer.push_str(text);
        let end = self.text_buffer.len();

        self.overlay_widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Text {
                range: start..end,
                color,
            },
        });
    }

    pub fn fmt_label(&mut self, id: impl Hash, font: &Font, color: [f32; 4], args: Arguments) {
        let id = self.resolve_id(id);
        let start = self.text_buffer.len();
        let _ = self.text_buffer.write_fmt(args);
        let end = self.text_buffer.len();

        let text_slice = &self.text_buffer[start..end];
        let size = measure_text(font, text_slice);
        let pos = self.current_pos();

        self.widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Text {
                range: start..end,
                color,
            },
        });
        self.advance_parent(size);
    }

    /// A clickable button. Returns true on the frame the click completes
    /// (mouse released while still hovering, having been pressed on this widget).
    /// The `id` is taken the pushed onto the stack for active id pressed / released.
    pub fn button(
        &mut self,
        id: impl Hash,
        size: [f32; 2],
        label: &str,
        font: &Font,
        colors: ButtonColors,
    ) -> bool {
        self.push_id(&id);
        let id = self.resolve_id(id);
        let pos = self.current_pos();
        let hovered = point_in_rect(self.mouse.pos, pos, size);

        if hovered {
            self.hot_id = Some(id);
            if self.mouse.pressed {
                self.active_id = Some(id);
            }
        }

        let is_active = self.active_id == Some(id);
        let clicked = is_active && hovered && self.mouse.released;

        if self.mouse.released && self.active_id == Some(id) {
            self.active_id = None;
        }

        // Turnary????
        let color = if is_active && hovered {
            colors.active
        } else if hovered {
            colors.hover
        } else {
            colors.idle
        };

        self.widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Rect { color },
        });

        let text_size = measure_text(font, label);
        let text_pos = [
            pos[0] + (size[0] - text_size[0]) * 0.5,
            pos[1] + (size[1] - text_size[1]) * 0.5,
        ];

        let start = self.text_buffer.len();
        self.text_buffer.push_str(label);
        let end = self.text_buffer.len();

        self.widgets.push(Widget {
            id,
            size: text_size,
            pos: text_pos,
            kind: WidgetKind::Text {
                range: start..end,
                color,
            },
        });

        self.advance_parent(size);
        self.pop_id();
        clicked
    }

    /// A horizontal slider. Mutates `value` in place while dragged and
    /// returns true on any frame the value changed.
    pub fn slider(
        &mut self,
        id: impl Hash,
        size: [f32; 2],
        value: &mut f32,
        min: f32,
        max: f32,
        colors: SliderColors,
    ) -> bool {
        let id = self.resolve_id(id);
        let pos = self.current_pos();
        let hovered = point_in_rect(self.mouse.pos, pos, size);

        if hovered && self.mouse.pressed {
            self.active_id = Some(id)
        }
        let is_active = self.active_id == Some(id);

        let mut changed = false;
        if is_active {
            if self.mouse.down {
                let rel_x = (self.mouse.pos[0] - pos[0]).clamp(0.0, size[0]);
                let t = if size[0] > 0.0 { rel_x / size[0] } else { 0.0 };
                let new_value = min + t * (max - min);
                if new_value != *value {
                    *value = new_value;
                    changed = true;
                }
            }
            if self.mouse.released {
                self.active_id = None;
            }
        }

        let t = if max > min {
            ((*value - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // track
        self.widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Rect {
                color: colors.track,
            },
        });

        // fill
        self.widgets.push(Widget {
            id,
            size: [size[0] * t, size[1]],
            pos,
            kind: WidgetKind::Rect { color: colors.fill },
        });

        let handle_w = 8.0_f32.min(size[0]);
        let handle_pos = [pos[0] + t * (size[0] - handle_w), pos[1]];
        let handle_color = if is_active {
            colors.handle_active
        } else if hovered {
            colors.handle_hover
        } else {
            colors.handle
        };

        self.widgets.push(Widget {
            id,
            size: [handle_w, size[1]],
            pos: handle_pos,
            kind: WidgetKind::Rect {
                color: handle_color,
            },
        });

        self.advance_parent(size);
        changed
    }

    /// A single-line text input. Appends typed characters to `buffer` while
    /// focused, backspace removes the last character. Returns true if the
    /// buffer changed this frame. Click to focus, click elsewhere to unfocus.
    pub fn text_input(
        &mut self,
        id: impl Hash,
        size: [f32; 2],
        buffer: &mut String,
        font: &Font,
        colors: TextInputColors,
    ) -> bool {
        // @TODO: Keyboard movemen -> keys
        let id = self.resolve_id(id);
        let pos = self.current_pos();
        let hovered = point_in_rect(self.mouse.pos, pos, size);

        if self.mouse.pressed && hovered {
            self.focus_id = Some(id);
        }
        let is_focused = self.focus_id == Some(id);

        let mut changed = false;
        if is_focused {
            for &c in &self.keyboard.chars {
                // filter control characters (backspace/enter etc. attrive here too)
                // on some platforms/backends, so keep only printable input
                if !c.is_control() {
                    buffer.push(c);
                    changed = true;
                }
            }
            if self.keyboard.backspace {
                if buffer.pop().is_some() {
                    changed = true;
                }
            }
        }

        let box_color = if is_focused {
            colors.focused
        } else {
            colors.idle
        };
        self.widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Rect { color: box_color },
        });

        let inner_pad = 6.0;
        let inner_width = (size[0] - inner_pad * 2.0).max(0.0);
        let text_size = measure_text(font, buffer);

        let scroll = (text_size[0] - inner_width).max(0.0);

        let text_pos = [
            pos[0] + inner_pad - scroll,
            pos[1] + (size[1] - font.pixel_height) * 0.5,
        ];

        self.widgets.push(Widget {
            id,
            pos: [pos[0] + 1.0, pos[1] + 1.0],
            size: [(size[0] - 2.0).max(0.0), (size[1] - 2.0).max(0.0)],
            kind: WidgetKind::ClipPush,
        });

        let start = self.text_buffer.len();
        self.text_buffer.push_str(buffer);
        let end = self.text_buffer.len();

        self.widgets.push(Widget {
            id,
            size: text_size,
            pos: text_pos,
            kind: WidgetKind::Text {
                range: start..end,
                color: colors.text,
            },
        });

        // blinking cursor, only while focused
        if is_focused && (self.time % 1.0) < 0.5 {
            let cursor_x = text_pos[0] + text_size[0] + 1.0;
            self.widgets.push(Widget {
                id,
                size: [2.0, font.pixel_height],
                pos: [cursor_x, text_pos[1]],
                kind: WidgetKind::Rect {
                    color: colors.cursor,
                },
            });
        }

        self.widgets.push(Widget {
            id,
            pos: [0.0, 0.0],
            size: [0.0, 0.0],
            kind: WidgetKind::ClipPop,
        });

        self.advance_parent(size);
        changed
    }

    pub fn overlay_hit(&self, pos: [f32; 2]) -> bool {
        self.overlay_widgets
            .iter()
            .any(|w| point_in_rect(pos, w.pos, w.size))
    }

    fn current_pos(&self) -> [f32; 2] {
        let state = self.layout_stack.last().unwrap();
        [
            state.origin[0] + state.cursor[0],
            state.origin[1] + state.cursor[1],
        ]
    }

    fn advance_parent(&mut self, size: [f32; 2]) {
        let state = self.layout_stack.last_mut().unwrap();
        match state.direction {
            LayoutDirection::Horizontal => {
                state.cursor[0] += size[0] + self.gap;
                state.max_content_size[1] = state.max_content_size[1].max(size[1]);
            }
            LayoutDirection::Vertical => {
                state.cursor[1] += size[1] + self.gap;
                state.max_content_size[0] = state.max_content_size[0].max(size[0]);
            }
        }
    }

    pub fn draw(&self, renderer: &mut Renderer, font: &Font) {
        for w in self.widgets.iter().chain(self.overlay_widgets.iter()) {
            match &w.kind {
                WidgetKind::Rect { color } => renderer.draw_rect(w.pos, w.size, *color),
                WidgetKind::Text { range, color } => {
                    let label = &self.text_buffer[range.clone()];
                    draw_text(renderer, font, label, w.pos, *color)
                }
                WidgetKind::ClipPush => renderer.push_scissor(w.pos, w.size),
                WidgetKind::ClipPop => renderer.pop_scissor(),
            }
        }
    }
}
