use std::time::Instant;

use crate::font::Font;
use crate::renderer::Renderer;
use glfw::{Action, Context, Key};

pub struct Application {
    pub name: String,
    pub window_w: i32,
    pub window_h: i32,
    pub update: fn(&mut Renderer, &Font, i32),
    pub init: fn(),
}

impl Application {
    pub fn new(
        name: String,
        window_w: i32,
        window_h: i32,
        update: fn(&mut Renderer, &Font, i32),
        init: fn(),
    ) -> Self {
        Application {
            name,
            window_w,
            window_h,
            update,
            init,
        }
    }

    pub fn run(&mut self) {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        let mut last_time = Instant::now();
        let mut frame_count = 0;
        let mut fps_timer = 0.0;
        let mut current_fps = 0;

        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut window, events) = glfw
            .create_window(800, 600, "Test Window", glfw::WindowMode::Windowed)
            .expect("Failed to create window");

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|s| window.get_proc_address(s).unwrap() as *const _);
        let mut renderer = Renderer::new(800, 600);

        let font_path = "assets/JetBrainsMono-Medium.ttf";
        let font = Font::load(font_path, 16.0).expect("Failed to load font");

        (self.init)();

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

            (self.update)(&mut renderer, &font, current_fps);

            renderer.end_frame();
            glfw.set_swap_interval(glfw::SwapInterval::None);
            window.swap_buffers();
        }
    }
}
