// Simp: a window program using automatic GL context creation. Checked (not run) for every desktop OS.
#import "Basic";

// Simp needs native C libraries (stb_image, FreeType) and a window system, which the WASM targets
// lack: they are listed in tests/stdlib-runtime-skips.txt.
#import "Input";
#import "Window_Creation";
#import "Simp";

main :: () {
    win := create_window(400, 300, "simp");
    set_render_target(win);
    w, h := get_render_dimensions(win);
    for 1..3 {
        update_window_events();
        clear_render_target(0.2, 0.3, 0.3, 1);
        set_shader_for_color(true);
        immediate_quad(10, 10, 100, 100, .{ 1, 1, 0, 1 });
        swap_buffers(win);
    }
    print("% %\n", w, h);
}
