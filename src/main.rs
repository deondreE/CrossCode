mod application;
mod font;
mod layout;
mod renderer;
use crate::layout::{ButtonColors, LayoutDirection, TextInputColors, UiContext};
use application::Application;

fn main() {
    let ui_instance = UiContext::new(0.0, 0.0); // Editor usually needs zero root padding

    // Editor State
    let mut file_content =
        String::from("// Welcome to Test\nfn main() {\n    println!(\"Hello Wayland!\");\n}");
    let mut filename = String::from("main.rs");
    let mut sidebar_width: f32 = 180.0;

    let mut app = Application::new(
        "T3 Editor".into(),
        1024,
        768,
        |renderer, font, _fps, mut ui| {
            let [win_w, win_h] = [800.0, 600.0]; // Should ideally come from app state

            ui.begin([0.0, 0.0], LayoutDirection::Vertical);

            // 1. Top Toolbar / Header
            ui.div(LayoutDirection::Horizontal, |ui| {
                ui.rect("header_bg", [win_w, 35.0], [0.12, 0.12, 0.14, 1.0]);
                // We'd use overlay_label here to draw over the rect if manual positioning
                ui.label(
                    "file_title",
                    &format!(" Editing: {}", filename),
                    font,
                    [0.8, 0.8, 0.8, 1.0],
                );
            });

            // 2. Main Content Area (Sidebar + Editor)
            ui.div(LayoutDirection::Horizontal, |ui| {
                // Sidebar
                ui.div(LayoutDirection::Vertical, |ui| {
                    ui.rect(
                        "sidebar_bg",
                        [sidebar_width, win_h - 60.0],
                        [0.1, 0.1, 0.11, 1.0],
                    );
                    ui.label("proj_label", "  PROJECT", font, [0.4, 0.4, 0.4, 1.0]);
                    ui.button(
                        "file_1",
                        [sidebar_width, 25.0],
                        "  main.rs",
                        font,
                        ButtonColors::default(),
                    );
                    ui.button(
                        "file_2",
                        [sidebar_width, 25.0],
                        "  layout.rs",
                        font,
                        ButtonColors::default(),
                    );
                    ui.button(
                        "file_3",
                        [sidebar_width, 25.0],
                        "  Cargo.toml",
                        font,
                        ButtonColors::default(),
                    );
                });

                // Editor TextArea
                let editor_size = [win_w - sidebar_width, win_h - 60.0];
                ui.text_area(
                    "main_editor",
                    editor_size,
                    &mut file_content,
                    font,
                    TextInputColors {
                        idle: [0.08, 0.08, 0.09, 1.0],
                        focused: [0.08, 0.08, 0.09, 1.0],
                        text: [0.9, 0.9, 0.8, 1.0],
                        cursor: [0.4, 0.6, 1.0, 1.0],
                    },
                );
            });

            // 3. Status Bar
            ui.div(LayoutDirection::Horizontal, |ui| {
                ui.rect("status_bg", [win_w, 25.0], [0.15, 0.35, 0.6, 1.0]);
                ui.fmt_label(
                    "status_text",
                    font,
                    [1.0, 1.0, 1.0, 1.0],
                    format_args!(
                        " UTF-8  |  Rust  |  Lines: {}",
                        file_content.lines().count()
                    ),
                );
            });

            ui.draw(renderer, font);
            ui
        },
        || {
            println!("Editor Core Initialized");
        },
        ui_instance,
    );
    app.run();
}
