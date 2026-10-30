// test: skip_on(.WASM, reason = "needs Window_Creation, which has no wasm target") serial
// Rendering into a window after its context is torn down: the frame that was already running when a window
// closed still draws and presents, and that must be harmless.
#import "Basic";
#import "Window_Creation";
#import "Input";
#import "Simp";
Lifecycle :: #import "internal/window/lifecycle";

main :: () {
    win := create_window(200, 150, "closed window");

    set_render_target(win);
    update_window_events();
    clear_render_target(0.2, 0.3, 0.3, 1);
    swap_buffers(win);

    // The same notification the window system sends when the user closes the window.
    Lifecycle.notify_closed(win);

    for 1..3 {
        update_window_events();
        clear_render_target(0, 0, 0, 1);
        set_shader_for_color(true);
        immediate_quad(10, 10, 100, 100, .{ 1, 1, 0, 1 });
        swap_buffers(win);
    }

    print("ok\n");
}
