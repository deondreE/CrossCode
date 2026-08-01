package main

import "core:fmt"
import engine "engine"

my_font: engine.Font

start_app :: proc(app: ^engine.Application) {
	fmt.println("Start app...")
	f, ok := engine.load_font("assets/JetBrainsMono-Medium.ttf", 32);
	if ok do my_font = f
}

update :: proc(app: ^engine.Application) {
	engine.draw_rect({40, 40}, {300, 120}, {0.2, 0.22, 0.28, 1})
	engine.draw_text(&my_font, "Hello UI!", {60, 100},  {1, 1, 1, 1})
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
