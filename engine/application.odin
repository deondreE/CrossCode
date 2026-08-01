package engine

import "core:fmt"
import "base:runtime"
import "vendor:glfw"

ApplicationSpec :: struct {
    name: string,
    version: string,
    width: i32,
    height: i32,
    entry: proc(app: ^Application),
    update: proc(app: ^Application)
}

Application :: struct {
    spec: ApplicationSpec,
    window: Window,
    running: bool
}

create :: proc (spec: ApplicationSpec) -> (Application, bool) {
    app: Application
    app.spec = spec

    lw, lh := spec.width, spec.height
    if lw == 0 do lw = 1280
    if lh == 0 do lh = 720

    win, ok := window_create(spec.name, lw, lh)
    if !ok do return {}, false
    app.window = win

    if !render_init(win.width, win.height) do return {}, false

    glfw.SetFramebufferSizeCallback(win.handle, proc "c"(handle: glfw.WindowHandle, width: i32, height: i32) {
  		context = runtime.default_context() // needed since this is a C callback
        render_resize(width, height)
    })

    return app, true
}

run :: proc(app: ^Application) {
    fmt.printfln("Booting %s v%s", app.spec.name, app.spec.version)

    app.running = true

    if app.spec.entry != nil {
        app.spec.entry(app)
    }

    for app.running && !window_should_close(&app.window) {
        window_poll(&app.window)

        render_begin_frame()
        if app.spec.update != nil {
            app.spec.update(app)
        }
        render_end_frame()

        window_swap(&app.window)
    }

    shutdown(app)
}

shutdown :: proc(app: ^Application) {
    app.running = false
    render_shutdown()
    window_destroy(&app.window)
}
