package main

import "core:fmt"
import engine "engine"

my_font: engine.Font

start_app :: proc(app: ^engine.Application) {
	fmt.println("Start app...")
	f, ok := engine.load_font("assets/JetBrainsMono-Medium.ttf", 16);
	if ok do my_font = f
}

test_val := false
slider_val: f32 = 0
update :: proc(app: ^engine.Application) {
	l := engine.start_layout(.Horizontal, 400, 400)
	if (engine.button(&my_font, "CLICK ME:")){
		fmt.println("Button Pressed!")
	}
	engine.spacer(250)

	if (engine.button(&my_font, "Quit")) {
		// handle quit
		fmt.println("Button Pressed")
	}

	engine.begin_group("Settings Form", {220, 220})
		engine.label(&my_font, "Form");

		if (engine.checkbox(&my_font, "Test", &test_val)) {
			fmt.println("Test")
		}
		engine.label(&my_font, fmt.tprintf("%.2f", slider_val))

		if (engine.slider(&slider_val, 0, 100)) {
			fmt.println(slider_val)
		}
	engine.end_group()

	engine.end_layout()

}

main :: proc() {
	app, ok := engine.create(engine.ApplicationSpec{
			name = "testing",
			version = "0.1.0",
			width = 1280,
			height = 720,
			entry = start_app,
			update = update
	})
	if !ok do return

	engine.run(&app);
}
