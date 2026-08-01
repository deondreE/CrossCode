#+build windows
package engine

import "core:fmt"
import glfw "vendor:glfw"

Window :: struct {
    handle: glfw.WindowHandle,
    width:  i32,
    height: i32,
    title:  string,
}

window_create :: proc(title: string, width, height: i32) -> (Window, bool) {
    if !glfw.Init() {
        fmt.eprintln("[window] glfw init failed")
        return {}, false
    }

    glfw.WindowHint(glfw.CONTEXT_VERSION_MAJOR, 3)
    glfw.WindowHint(glfw.CONTEXT_VERSION_MINOR, 3)
    glfw.WindowHint(glfw.OPENGL_PROFILE, glfw.OPENGL_CORE_PROFILE)

    ctitle := fmt.ctprint(title)
    handle := glfw.CreateWindow(width, height, ctitle, nil, nil)
    if handle == nil {
        fmt.eprintln("[window] create failed")
        glfw.Terminate()
        return {}, false
    }

    glfw.MakeContextCurrent(handle)
    glfw.SwapInterval(1) // vsync

    fb_w, fb_h := glfw.GetFramebufferSize(handle)
    w := Window{handle = handle, width = fb_w, height = fb_h, title = title}
    return w, true
}

window_should_close :: proc(w: ^Window) -> bool {
    return bool(glfw.WindowShouldClose(w.handle))
}

window_poll :: proc(w: ^Window) {
    glfw.PollEvents()
}

window_swap :: proc(w: ^Window) {
    glfw.SwapBuffers(w.handle)
}

window_destroy :: proc(w: ^Window) {
    glfw.DestroyWindow(w.handle)
    glfw.Terminate()
}
