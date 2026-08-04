mod application;
mod font;
mod layout;
mod renderer;
use crate::layout::{ButtonColors, LayoutDirection, SliderColors, TextInputColors, UiContext};
use application::Application;
fn main() {
    // Initialize the UI Context with padding and gap settings
    let ui_instance = UiContext::new(10.0, 5.0);
    let mut volume: f32 = 0.5;
    let mut toggle: bool = false;
    let mut name = String::new();
    let mut app = Application::new(
        "Engine Test".into(),
        800,
        600,
        |renderer, font, fps, mut ui| {
            ui.begin([0.0, 0.0], LayoutDirection::Vertical);
            ui.fmt_label(
                "fps_label",
                font,
                [1.0, 1.0, 0.0, 1.0],
                format_args!("FPS: {}", fps),
            );
            ui.push_layout(LayoutDirection::Horizontal);
            if ui.button(
                "toggle_button",
                [100.0, 30.0],
                "Click me",
                font,
                ButtonColors::default(),
            ) {
                println!("cliked!");
                toggle = !toggle;
            }
            ui.text_input(
                "name_input",
                [200.0, 24.0],
                &mut name,
                font,
                TextInputColors::default(),
            );
            ui.fmt_label(
                "hello_label",
                font,
                [1.0, 1.0, 1.0, 1.0],
                format_args!("Hello, {}", name),
            );
            if toggle {
                ui.push_layout(LayoutDirection::Vertical);
                ui.label("row1_label", "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.push_layout(LayoutDirection::Horizontal);
                if ui.button(
                    "btn_1",
                    [100.0, 30.0],
                    "Click me",
                    font,
                    ButtonColors::default(),
                ) {}

                if ui.button(
                    "btn2",
                    [100.0, 30.0],
                    "Click me",
                    font,
                    ButtonColors::default(),
                ) {}

                ui.pop_layout();
                ui.label("row2_label", "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.label("row3_label", "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.push_layout(LayoutDirection::Horizontal);
                if ui.button(
                    "btn3",
                    [100.0, 30.0],
                    "Click me",
                    font,
                    ButtonColors::default(),
                ) {}
                if ui.button(
                    "btn4",
                    [100.0, 30.0],
                    "Click me",
                    font,
                    ButtonColors::default(),
                ) {}
                ui.pop_layout();
                ui.label("row4_label", "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.label("row5_label", "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.label("row6_label", "Bustton Clicked!", font, [1.0, 1.0, 1.0, 1.0]);
                ui.pop_layout();
            }
            ui.slider(
                "volume_slider",
                [200.0, 16.0],
                &mut volume,
                0.0,
                1.0,
                SliderColors::default(),
            );
            ui.fmt_label(
                "volume_label",
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
