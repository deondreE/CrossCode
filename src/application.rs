use std::time::Instant;

use crate::font::Font;
use crate::layout::MouseState;
use crate::layout::UiContext;
use crate::renderer::Renderer;
use glfw::{Action, Context, Key};

pub struct Application<F>
where
    F: FnMut(&mut Renderer, &Font, i32, UiContext) -> UiContext,
{
    pub name: String,
    pub window_w: i32,
    pub window_h: i32,
    pub update: F,
    pub init: fn(),
    pub ui: UiContext,
}

impl<F> Application<F>
where
    F: FnMut(&mut Renderer, &Font, i32, UiContext) -> UiContext,
{
    pub fn new(
        name: String,
        window_w: i32,
        window_h: i32,
        update: F,
        init: fn(),
        ui: UiContext,
    ) -> Self {
        Application {
            name,
            window_w,
            window_h,
            update,
            init,
            ui,
        }
    }

    pub fn run(&mut self) {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        let mut last_time = Instant::now();
        let mut frame_count = 0;
        let mut fps_timer = 0.0;
        let mut current_fps = 0;

        let mut mouse_pos = [0.0f32, 0.0f32];
        let mut mouse_down = false;
        let mut mouse_pressed = false;
        let mut mouse_released = false;

        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut window, events) = glfw
            .create_window(800, 600, "Test Window", glfw::WindowMode::Windowed)
            .expect("Failed to create window");

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);

        glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        gl::load_with(|s| window.get_proc_address(s).unwrap() as *const _);

        let mut renderer = Renderer::new(800, 600);

        let font_path = "assets/JetBrainsMono-Medium.ttf";
        let font = Font::load(font_path, 16.0).expect("Failed to load font");

        (self.init)();

        let mut current_ui = std::mem::replace(&mut self.ui, UiContext::new(0.0, 0.0));

        while !window.should_close() {
            glfw.poll_events();

            // Handle window events
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::FramebufferSize(width, height) => {
                        renderer.resize(width, height);
                    }
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        window.set_should_close(true);
                    }
                    glfw::WindowEvent::CursorPos(x, y) => {
                        mouse_pos = [x as f32, y as f32];
                    }
                    glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, action, _) => {
                        match action {
                            Action::Press => {
                                mouse_pressed = true;
                                mouse_down = true;
                            }
                            Action::Release => {
                                mouse_released = true;
                                mouse_down = false;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            let now = Instant::now();
            let delta_time = now.duration_since(last_time).as_secs_f32();
            last_time = now;

            fps_timer += delta_time;
            frame_count += 1;
            if fps_timer >= 1.0 {
                current_fps = frame_count;
                frame_count = 0;
                fps_timer -= 1.0;
            }

            renderer.begin_frame();
            current_ui.set_mouse(MouseState {
                pos: mouse_pos,
                down: mouse_down,
                pressed: mouse_pressed,
                released: mouse_released,
            });

            current_ui = (self.update)(&mut renderer, &font, current_fps, current_ui);

            renderer.end_frame();
            window.swap_buffers();

            mouse_pressed = false;
            mouse_released = false;
        }
        self.ui = current_ui;
    }
}
