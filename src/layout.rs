use crate::font::{Font, draw_text, measure_text};
use crate::renderer::Renderer;

use std::fmt::{Arguments, Write};

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

/// Eqiuvilent to the html `body` tag.
pub struct UiContext {
    pub widgets: Vec<Widget>,
    layout_stack: Vec<LayoutState>,
    pub padding: f32,
    pub gap: f32,
    pub text_buffer: String,
}

impl UiContext {
    pub fn new(padding: f32, gap: f32) -> Self {
        Self {
            widgets: Vec::with_capacity(1000),
            layout_stack: Vec::with_capacity(16),
            padding,
            gap,
            text_buffer: String::with_capacity(4096),
        }
    }

    /// Entry point for the frame. Sets the root layout area.
    pub fn begin(&mut self, screen_pos: [f32; 2], dir: LayoutDirection) {
        self.widgets.clear();
        self.text_buffer.clear();
        self.layout_stack.clear();
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

    pub fn rect(&mut self, id: u64, size: [f32; 2], color: [f32; 4]) {
        let pos = self.current_pos();
        self.widgets.push(Widget {
            id,
            size,
            pos,
            kind: WidgetKind::Rect { color },
        });
        self.advance_parent(size);
    }

    pub fn label(&mut self, id: u64, text: &str, font: &Font, color: [f32; 4]) {
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

    pub fn fmt_label(&mut self, id: u64, font: &Font, color: [f32; 4], args: Arguments) {
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
        for w in &self.widgets {
            match &w.kind {
                WidgetKind::Rect { color } => renderer.draw_rect(w.pos, w.size, *color),
                WidgetKind::Text { range, color } => {
                    let label = &self.text_buffer[range.clone()];
                    draw_text(renderer, font, label, w.pos, *color)
                }
            }
        }
    }
}
