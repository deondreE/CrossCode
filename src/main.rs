mod application;
mod font;
mod renderer;

use application::Application;

fn main() {
    let mut app = Application::new(
        "Engine Test".into(),
        800,
        600,
        |renderer, font, fps| {
            // Draw your game objects
            renderer.draw_rect([50.0, 50.0], [100.0, 100.0], [0.0, 0.8, 0.4, 1.0]);

            // Draw FPS counter in the corner
            let fps_text = format!("FPS: {}", fps);
            crate::font::draw_text(
                renderer,
                font,
                &fps_text,
                [10.0, 10.0],         // Top left
                [1.0, 1.0, 0.0, 1.0], // Yellow
            );
        },
        || {
            println!("Engine Initialized");
        },
    );

    app.run();
}
