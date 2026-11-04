// Production image upload, composition and packed readback; exit 77 only for adapter absence.
#import "Basic";
#import "Math";
#import "Window_Type";
#import "Extensions/UI";
#import "Extensions/WebGPU";
Image :: #import "Extensions/Image";

WIDTH :: 19;
HEIGHT :: 13;
failures: int;

pixel :: (image: Rendered_Image, x: int, y: int, expected: [4] int, tolerance: int = 0) {
    offset := (y * WIDTH + x) * 4;

    for channel: 0..3 {
        actual := cast(int) image.pixels[offset + channel];
        if abs(actual - expected[channel]) > tolerance {
            print("FAIL pixel (%, %) channel %: expected %, actual %\n", x, y, channel, expected[channel], actual);
            failures += 1;
        }
    }
}

// The device helper copies this logger into the uncaptured-error callback's context.
// Fail immediately, including when a native callback arrives on another thread.
gpu_error :: (message: string, data: *void, info: Log_Info) {
    assert(data == null);
    print("%\n", message);
    if (info.common_flags & .ERROR) != 0 exit(1);
}

read :: (renderer: *Renderer, ui: *UI_State, clear: Color) -> Rendered_Image {
    wgpuDevicePushErrorScope(renderer.device, .Validation);
    result := render_to_image(renderer, ui, clear);
    type, message := webgpu_pop_error_scope(renderer.instance, renderer.device);

    if type != .NoError {
        print("FAIL GPU error during submission/readback: %: %\n", type, message);
        exit(1);
    }

    if result.width != WIDTH || result.height != HEIGHT || result.pixels.count != WIDTH * HEIGHT * 4 {
        print("FAIL readback dimensions/count: % x %, % bytes\n", result.width, result.height, result.pixels.count);
        exit(1);
    }

    return result;
}

main :: () {
    fallback := false;
    for get_command_line_arguments() if it == "fallback" fallback = true;

    // Preflight distinguishes adapter absence from renderer/device setup failure.
    instance := wgpuCreateInstance(null);
    if !instance exit(1);

    options: WGPURequestAdapterOptions;
    options.forceFallbackAdapter = xx fallback;
    adapter := webgpu_request_adapter(instance, *options);
    if !adapter {
        wgpuInstanceRelease(instance);
        exit(77);
    }

    wgpuAdapterRelease(adapter);
    wgpuInstanceRelease(instance);

    // Deterministic partial setup failure: no native window means no surface.
    absent_window: Window_Type;

    assert(create_window_renderer(absent_window, WIDTH, HEIGHT) == null);

    // Keep the intentional absent-window log outside the rendering error capture.
    push_context, defer_pop;
    context.logger = gpu_error;
    context.logger_data = null;

    renderer := create_offscreen_renderer(WIDTH, HEIGHT, *options);
    if !renderer {
        print("FAIL renderer creation on available adapter\n");
        exit(1);
    }

    defer destroy_renderer(renderer);
    info: WGPUAdapterInfo;

    wgpuAdapterGetInfo(renderer.adapter, *info);
    print("UI renderer adapter: % (%, %)\n", to_string(info.device), info.backendType, info.adapterType);
    wgpuAdapterInfoFreeMembers(info);
    source := Image.make_image(2, 2, .RGBA8);

    defer Image.deinit(*source);
    colors: [16] u8 = .[230, 20, 40, 255, 10, 210, 60, 255, 30, 50, 190, 255, 170, 90, 15, 255];
    for colors source.pixels[it_index] = it;

    ui := headless(.{ width = WIDTH, height = HEIGHT });

    defer free_ui(*ui);

    // Non-square destination with a nonzero origin; samples lie well within texels.
    draw_image(*ui, .{ 2, 3, 8, 6 }, source);
    image := read(renderer, *ui, rgb(1, 2, 3));

    pixel(image, 3, 4, .[230, 20, 40, 255]);
    // This interior point differs under linear filtering (u=.3125, v=.4167).
    pixel(image, 4, 5, .[230, 20, 40, 255]);
    pixel(image, 8, 4, .[10, 210, 60, 255]);
    pixel(image, 3, 7, .[30, 50, 190, 255]);
    pixel(image, 8, 7, .[170, 90, 15, 255]);
    pixel(image, 1, 3, .[1, 2, 3, 255]);
    pixel(image, 10, 8, .[1, 2, 3, 255]);
    array_free(image.pixels);

    // New pixels remain caller owned; the next frame must upload the mutation.
    reset_draw_list(*ui.draw);

    for 0..3 {
        source.pixels[it * 4] = 200;
        source.pixels[it * 4 + 1] = 40;
        source.pixels[it * 4 + 2] = 80;
        source.pixels[it * 4 + 3] = 128;
    }

    draw_image(*ui, .{ 0, 0, 8, 8 }, source);
    image = read(renderer, *ui, .{});
    // round(200*128/255), round(40*128/255), round(80*128/255).
    pixel(image, 3, 3, .[100, 20, 40, 128], 1);
    array_free(image.pixels);

    // Source-over order, interrupted by a solid batch; alpha is checked too.
    for reverse: 0..1 {
        reset_draw_list(*ui.draw);
        other := Image.make_image(1, 1, .RGBA8);

        defer Image.deinit(*other);
        other.pixels[0] = 20;
        other.pixels[1] = 180;
        other.pixels[2] = 240;
        other.pixels[3] = 128;
        if reverse == 0 draw_image(*ui, .{ 0, 0, 8, 8 }, source);
        else draw_image(*ui, .{ 0, 0, 8, 8 }, other);

        draw_rect(*ui, .{ 12, 0, 2, 2 }, rgb(9, 8, 7));
        if reverse == 0 draw_image(*ui, .{ 0, 0, 8, 8 }, other);
        else draw_image(*ui, .{ 0, 0, 8, 8 }, source);

        image = read(renderer, *ui, .{});
        if reverse == 0 pixel(image, 3, 3, .[60, 100, 140, 192], 1);
        else pixel(image, 3, 3, .[105, 65, 100, 192], 1);

        pixel(image, 12, 0, .[9, 8, 7, 255]);
        array_free(image.pixels);
    }

    // Transparent color contributes nothing over an opaque background.
    source.pixels[3] = 0;
    reset_draw_list(*ui.draw);
    draw_image(*ui, .{ 0, 0, 8, 8 }, source);
    image = read(renderer, *ui, rgb(30, 60, 90));
    pixel(image, 1, 1, .[30, 60, 90, 255]);
    pixel(image, 6, 1, .[115, 50, 85, 255], 1);
    array_free(image.pixels);

    // Scale and fractional clip: floor(1.25*2)=2, ceil(4.25*2)=9.
    scaled := headless(.{ width = WIDTH, height = HEIGHT, scale = 2 });

    defer free_ui(*scaled);
    for 0..3 source.pixels[it * 4 + 3] = 255;

    push_clip(*scaled, .{ 1.25, -1, 3, 5.25 });
    draw_image(*scaled, .{ 1, 1, 5, 5 }, source);
    pop_clip(*scaled);
    image = read(renderer, *scaled, rgb(1, 2, 3));
    pixel(image, 2, 2, .[200, 40, 80, 255]);
    pixel(image, 8, 8, .[200, 40, 80, 255]);
    pixel(image, 9, 4, .[1, 2, 3, 255]);
    pixel(image, 4, 9, .[1, 2, 3, 255]);
    pixel(image, 1, 4, .[1, 2, 3, 255]);
    array_free(image.pixels);
    if failures exit(1);

    print("ui images: ok\n");
}
