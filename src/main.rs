mod application;
mod font;
mod layout;
mod renderer;

use crate::layout::{ButtonColors, LayoutDirection, SliderColors, UiContext};
use application::Application;

fn main() {
    // Initialize the UI Context with padding and gap settings
    let ui_instance = UiContext::new(10.0, 5.0);

    let mut volume: f32 = 0.5;
    let mut toggle: bool = false;

    let mut app = Application::new(
        "Engine Test".into(),
        800,
        600,
        |renderer, font, fps, mut ui| {
            ui.begin([0.0, 0.0], LayoutDirection::Vertical);

            ui.fmt_label(1, font, [1.0, 1.0, 0.0, 1.0], format_args!("FPS: {}", fps));

            ui.push_layout(LayoutDirection::Horizontal);
            if ui.button(10, [100.0, 30.0], "Click me", font, ButtonColors::default()) {
                println!("cliked!");
                toggle = !toggle;
            }

            if toggle {
                ui.push_layout(LayoutDirection::Vertical);
                ui.label(13, "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);

                ui.push_layout(LayoutDirection::Horizontal);
                if ui.button(14, [100.0, 30.0], "Click me", font, ButtonColors::default()) {}
                if ui.button(20, [100.0, 30.0], "Click me", font, ButtonColors::default()) {}
                ui.pop_layout();

                ui.label(15, "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.label(16, "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);

                ui.push_layout(LayoutDirection::Horizontal);
                if ui.button(21, [100.0, 30.0], "Click me", font, ButtonColors::default()) {}
                if ui.button(22, [100.0, 30.0], "Click me", font, ButtonColors::default()) {}
                ui.pop_layout();

                ui.label(17, "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.label(18, "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.label(19, "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);

                ui.pop_layout();
            }

            ui.slider(
                11,
                [200.0, 16.0],
                &mut volume,
                0.0,
                1.0,
                SliderColors::default(),
            );
            ui.fmt_label(
                12,
                font,
                [1.0, 1.0, 1.0, 1.0],
                format_args!("Volume {:.2}", volume),
            );

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
