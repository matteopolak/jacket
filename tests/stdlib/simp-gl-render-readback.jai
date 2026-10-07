// Simp's OpenGL backend in a real window: triangles, fans, rounded rectangles, textured quads,
// gradients and text drawn into a render-target texture are read back and checked pixel by pixel;
// the window itself is prepared, cleared, presented and remeasured; and the renderer is shut down
// and brought back on the same context. Fonts come from the operating system (no font ships with
// the repository); without one the text part is left out. Needs OpenGL 3.3 (Windows CI runners
// lack it) and a display (Linux CI runs Xvfb with Mesa); wasm has neither (see the skip list).
#import "Basic";
#import "Math";

#import "Window_Creation";
#import "Input";
#import "File";
#import "Simp";
GL :: #import "GL";

SIZE :: 32;

// Reads the target back as RGBA8, top row first.
read_target :: (target: *Texture) -> Bitmap {
    reader := pixel_read_begin(target, .RGBA8);
    assert(reader != null, "pixel_read_begin failed");
    for 1..1000 {
        if pixel_read_is_done(reader) break;
        sleep_milliseconds(1);
    }
    return pixel_read_end(reader);
}

pixel :: (bitmap: *Bitmap, x: int, y: int) -> [4] u8 {
    p := bitmap.data.data + y * bitmap.stride + x * 4;
    return .[p[0], p[1], p[2], p[3]];
}

// Within a little of the expected color: drivers may round differently.
rgb_near :: (bitmap: *Bitmap, x: int, y: int, r: int, g: int, b: int) -> bool {
    p := pixel(bitmap, x, y);
    return abs(p[0] - r) <= 3 && abs(p[1] - g) <= 3 && abs(p[2] - b) <= 3;
}

lit_count :: (bitmap: *Bitmap, channel := 0) -> int {
    count := 0;
    for y: 0..bitmap.height - 1 for x : 0..bitmap.width - 1 if pixel(bitmap, x, y)[channel] > 32 count += 1;
    return count;
}

main :: () {
    window := create_window(160, 120, "jaic simp gl test");
    assert(cast(u64) window != 0, "create_window failed");
    for 1..10 {
        update_window_events();
        sleep_milliseconds(5);
    }
    granted := prepare_window(window, 0);
    assert(granted >= 0, "prepare_window granted % samples", granted);
    set_render_target(window);
    width, height := get_render_dimensions(window);
    assert(width > 0 && height > 0, "window renders at % x %", width, height);
    update_window(window);
    set_window_dimensions(window, width, height);
    clear_render_target(0.1, 0.2, 0.3, 1);
    set_shader_for_color();
    immediate_quad(0, 0, 10, 10, .{ 1, 1, 0, 1 });
    swap_buffers(window);
    #if OS == .MACOS {
        swap_view_buffers(window); // The view presents the same context's back buffer.
    }

    target := texture_create_render_target(SIZE, SIZE, .RGBA8);
    assert(target != null, "texture_create_render_target failed");
    shapes(target);
    images(target);
    text(target);

    // The renderer can be shut down and restarted on the current context.
    initialize_opengl_renderer(null); // Refused: no procedures.
    shutdown_opengl_renderer();
    shutdown_opengl_renderer(); // Twice is harmless.
    assert(initialize_opengl_renderer(*GL.gl), "initialize_opengl_renderer on the live context failed");
    set_render_target(target, .LEFT_HANDED);
    set_shader_for_color();
    clear_render_target(0, 0, 0, 1);
    immediate_quad(0, 0, SIZE, SIZE, .{ 0, 1, 0, 1 });
    image := read_target(target);
    assert(rgb_near(*image, 5, 5, 0, 255, 0), "drawing after a restart: %", pixel(*image, 5, 5));
    deinit(*image);

    set_render_target(window);
    clear_render_target(0, 0, 0, 1);
    swap_buffers(window, vsync = false);
    deinit(target);
    free(target);
    print("ok\n");
}

shapes :: (target: *Texture) {
    set_render_target(target, .LEFT_HANDED);

    set_shader_for_color();
    clear_render_target(0, 0, 0, 1);
    immediate_begin();
    immediate_triangle(.{ 0, 0, 0 }, .{ 16, 0, 0 }, .{ 0, 16, 0 }, .{ 1, 0, 0, 1 });
    immediate_triangle(.{ 32, 32, 0 }, .{ 16, 32, 0 }, .{ 32, 16, 0 }, .{ 0, 1, 0, 1 }, .{ 0, 1, 0, 1 }, .{ 0, 1, 0, 1 });
    image := read_target(target);
    assert(rgb_near(*image, 2, 2, 255, 0, 0), "red triangle (top-left): %", pixel(*image, 2, 2));
    assert(rgb_near(*image, 29, 29, 0, 255, 0), "green triangle (bottom-right): %", pixel(*image, 29, 29));
    assert(rgb_near(*image, 29, 2, 0, 0, 0) && rgb_near(*image, 16, 16, 0, 0, 0), "triangles drew outside themselves");
    deinit(*image);

    clear_render_target(0, 0, 0, 1);
    square := Vector2.[.{ 8, 8 }, .{ 24, 8 }, .{ 24, 24 }, .{ 8, 24 }, .{ 8, 8 }];
    immediate_triangle_fan(.{ 16, 16 }, square, .{ 0, 0, 1, 1 }, .{ 0, 0, 1, 1 });
    image = read_target(target);
    assert(rgb_near(*image, 10, 10, 0, 0, 255) && rgb_near(*image, 22, 20, 0, 0, 255), "fan interior: %", pixel(*image, 10, 10));
    assert(rgb_near(*image, 4, 4, 0, 0, 0) && rgb_near(*image, 27, 16, 0, 0, 0), "fan spilled outside its square");
    deinit(*image);

    // Scissor: only the inside of the box changes.
    set_scissor(0, 0, 8, 8);
    immediate_quad(0, 0, SIZE, SIZE, .{ 1, 1, 1, 1 });
    clear_scissor();
    image = read_target(target);
    assert(rgb_near(*image, 3, 3, 255, 255, 255) && rgb_near(*image, 12, 12, 0, 0, 255), "scissored quad: % %", pixel(*image, 3, 3), pixel(*image, 12, 12));
    deinit(*image);

    clear_render_target(0, 0, 0, 1);
    set_shader_for_rects();
    immediate_rounded_rect(.{ 0, 0 }, .{ SIZE, SIZE }, .{ 1, 1, 0, 1 }, .{ 16, 16 }, .{ 16, 16 }, 12);
    image = read_target(target);
    assert(rgb_near(*image, 16, 16, 255, 255, 0) && rgb_near(*image, 16, 1, 255, 255, 0), "rounded rect body: %", pixel(*image, 16, 16));
    assert(pixel(*image, 0, 0)[0] < 60, "rounded rect corner is filled: %", pixel(*image, 0, 0));
    deinit(*image);

    clear_render_target(0, 0, 0, 1);
    set_shader_for_gradient(.{ 0, 0, SIZE, SIZE }, 16, 0);
    immediate_quad(0, 0, SIZE, SIZE, .{ 1, 1, 1, 1 });
    image = read_target(target);
    first := pixel(*image, 8, 0)[0];
    last := pixel(*image, 8, SIZE - 1)[0];
    assert(abs(cast(int) last - cast(int) first) > 200, "gradient spans % .. %", first, last);
    middle := pixel(*image, 8, SIZE / 2)[0];
    assert(middle > min(first, last) && middle < max(first, last), "gradient middle % between % and %", middle, first, last);
    deinit(*image);

    // Nothing bound: the texture is refused, and the reset drops the shader.
    immediate_reset();
    immediate_set_texture(null);
}

images :: (target: *Texture) {
    texels: Bitmap;
    bitmap_alloc(*texels, 2, 2, .RGBA8);
    colors := u8.[255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255];
    memcpy(texels.data.data, colors.data, 16);
    sprite: Texture;
    sprite.nearest = true;
    assert(texture_load_from_bitmap(*sprite, *texels), "texture_load_from_bitmap failed");
    set_render_target(target, .LEFT_HANDED);
    clear_render_target(0, 0, 0, 1);
    set_shader_for_images(*sprite);
    immediate_quad(0, 0, SIZE, SIZE, .{ 1, 1, 1, 1 });
    image := read_target(target);
    assert(rgb_near(*image, 4, 4, 255, 0, 0) && rgb_near(*image, 27, 4, 0, 255, 0), "top texels: % %", pixel(*image, 4, 4), pixel(*image, 27, 4));
    assert(rgb_near(*image, 4, 27, 0, 0, 255) && rgb_near(*image, 27, 27, 255, 255, 255), "bottom texels: % %", pixel(*image, 4, 27), pixel(*image, 27, 27));
    deinit(*image);

    // The same image through a PNG file and memory (stb_image).
    make_directory_if_it_does_not_exist(".scratch");
    make_directory_if_it_does_not_exist(".scratch/simp-gl-render-readback");
    png := ".scratch/simp-gl-render-readback/texels.png";
    assert(bitmap_save(*texels, png, true), "bitmap_save failed");
    from_file: Texture;
    from_file.nearest = true;
    assert(texture_load_from_file(*from_file, png), "texture_load_from_file failed");
    bytes, read := read_entire_file(png);
    assert(read);
    from_memory: Texture;
    assert(texture_load_from_memory(*from_memory, cast([] u8) bytes), "texture_load_from_memory failed");
    assert(from_file.width == 2 && from_memory.height == 2);
    clear_render_target(0, 0, 0, 1);
    set_shader_for_images(*from_file);
    immediate_quad(0, 0, SIZE, SIZE, .{ 1, 1, 1, 1 });
    image = read_target(target);
    assert(rgb_near(*image, 27, 27, 255, 255, 255) && rgb_near(*image, 4, 27, 0, 0, 255), "texture from file: % %", pixel(*image, 27, 27), pixel(*image, 4, 27));
    deinit(*image);
    file_delete(png);
    free(bytes);
    deinit(*from_file);
    deinit(*from_memory);
    deinit(*sprite);
    deinit(*texels);
}

FONT_CANDIDATES :: string.[
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "C:/Windows/Fonts/arial.ttf",
];

text :: (target: *Texture) {
    found := "";
    for FONT_CANDIDATES if file_exists(it) {
        found = it;
        break;
    }
    #if OS == .MACOS assert(found.count > 0, "macOS ships Arial");
    if !found {
        print("no system font found: text drawing not checked\n");
        return;
    }
    cut := found.count - 1;
    while found[cut] != #char "/" cut -= 1;
    font := get_font_at_size(string.{ cut, found.data }, string.{ found.count - cut - 1, found.data + cut + 1 }, 20);
    assert(font != null);

    set_render_target(target, .LEFT_HANDED);
    clear_render_target(0, 0, 0, 1);
    width := draw_text(font, 2, 22, "Hi", .{ 1, 1, 1, 1 });
    assert(width > 0);
    image := read_target(target);
    lit := lit_count(*image);
    assert(lit > 10, "text lit % pixels", lit);
    for y: 0..SIZE - 1 for x : 0..SIZE - 1 {
        if pixel(*image, x, y)[0] <= 32 continue;
        assert(x >= 1 && x <= 2 + width + 1, "text pixel at column % (width %)", x, width);
        assert(y >= 22 - 21 && y <= 22 + 6, "text pixel at row % (baseline 22)", y);
    }
    deinit(*image);

    // Highlights and color-map text through the GPU.
    table := Vector4.[.{ 1, 0, 0, 1 }, .{ 0, 0, 1, 1 }];
    set_color_map(table);
    clear_render_target(0, 0, 0, 1);
    marks := bool.[false, true];
    total := draw_text_with_highlights(font, 0, 22, "HH", marks, 0, 1);
    image = read_target(target);
    red_left, blue_right := false, false;
    for y: 0..SIZE - 1 for x : 0..SIZE - 1 {
        p := pixel(*image, x, y);
        if x < total / 2 && p[0] > 64 red_left = true;
        if x >= total / 2 && p[2] > 64 blue_right = true;
    }
    assert(red_left && blue_right, "highlighted text: red left %, blue right %", red_left, blue_right);
    deinit(*image);
    clear_render_target(0, 0, 0, 1);
    draw_text(font, 2, 22, "H", cast(u8) 1);
    image = read_target(target);
    assert(lit_count(*image, 2) > 5 && lit_count(*image, 0) == 0, "color index 1 is not blue");
    deinit(*image);
    set_color_map(.[]);
    deinit_fonts();
}
