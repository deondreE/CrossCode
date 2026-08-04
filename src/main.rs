mod application;
mod font;
mod layout;
mod renderer;

use crate::layout::{LayoutDirection, UiContext};
use application::Application;

fn main() {
    // Initialize the UI Context with padding and gap settings
    let ui_instance = UiContext::new(10.0, 5.0);

    let mut app = Application::new(
        "Engine Test".into(),
        800,
        600,
        |renderer, font, fps, mut ui| {
            ui.begin([0.0, 0.0], LayoutDirection::Vertical);

            ui.fmt_label(1, font, [1.0, 1.0, 0.0, 1.0], format_args!("FPS: {}", fps));

            ui.push_layout(LayoutDirection::Horizontal);
            ui.rect(2, [20.0, 20.0], [0.0, 1.0, 0.0, 1.0]); // Green status
            ui.label(3, "System Nominal", font, [0.8, 0.8, 0.8, 1.0]);
            ui.pop_layout();

            ui.draw(renderer, font);

            ui
        },
        || {
            println!("Engine Initialized");
        },
        ui_instance,
    );

    app.run();
}
