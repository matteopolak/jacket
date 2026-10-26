// Private embedding protocol. This canvas has no native Window_Type handle.
// The page owns the canvas/context; the WebGPU host creates actual surfaces.

demo_canvas_available :: () -> bool {
    return demo_host_canvas_available() != 0;
}

demo_canvas_dimensions :: () -> s32, s32 {
    width, height: u32;
    demo_host_canvas_size(*width, *height);
    return cast(s32) width, cast(s32) height;
}

demo_canvas_input :: () -> resize_count: int, quit: bool, pause: bool, palette_clicks: int {
    resize_count = 0;
    quit = false;
    pause = false;
    palette_clicks = 0;
    event: Demo_Canvas_Event;
    while demo_host_canvas_poll(*event) != 0 {
        // Private host.resize emits type5; it is not native Input.QUIT.
        if event.type == 5 {
            resize_count += 1;
            continue;
        }

        if event.type != 1 || event.pressed == 0 continue;
        if event.key == 27 quit = true;
        if event.key == 32 pause = !pause;
        if event.key == 168 palette_clicks += 1;
    }

    return resize_count, quit, pause, palette_clicks;
}

demo_canvas_seconds :: () -> float64 {
    return cast(float64) demo_host_clock_microseconds() / 1_000_000.0;
}

#scope_file
Demo_Canvas_Event :: struct {
    type: u32;
    key: u32;
    pressed: u32;
    modifiers: u32;
    x: s32;
    y: s32;
    utf32: u32;
    repeat: u32;
}

#assert size_of(Demo_Canvas_Event) == 32;

demo_host_canvas_available :: () -> u32 #foreign "jai_canvas_available";
demo_host_canvas_size :: (width: *u32, height: *u32) #foreign "jai_canvas_size";
demo_host_canvas_poll :: (event: *Demo_Canvas_Event) -> u32 #foreign "jai_canvas_poll_event";

// Monotonic page clock; the native helper remains Basic.seconds_since_init.
demo_host_clock_microseconds :: () -> u64 #foreign "jai_canvas_clock_microseconds";
