// test: skip_on(.WASM, reason = "needs a native window, and Window_Type has no wasm target") serial args("${JAIC_TEST_FONT_PATH}")
// High-DPI drawing: the window reports its pixel scale, the viewport covers the whole framebuffer while
// the application keeps drawing in window units, nearest filtering keeps hard edges when a texture is
// magnified, and a font made for a scale above 1 is rasterized at pixel size and laid out in window units.
#import "Basic";
#import "Window_Creation";
#import "Input";
#import "File";
#import "Simp";
GL :: #import "GL";

main :: () {
    window := create_window(320, 240, "jaic pixel scale test");

    assert(cast(u64) window != 0, "create_window failed");

    for 1..10 {
        update_window_events();
        sleep_milliseconds(5);
    }

    prepare_window(window, 0);
    set_render_target(window, .LEFT_HANDED);

    scale := get_pixel_scale(window);
    width, height := get_render_dimensions(window);
    pixels_w, pixels_h := get_pixel_dimensions(window);

    assert(scale >= 1);
    assert(width == 320 && height == 240, "render size %x%", width, height);
    assert(pixels_w == cast(s32) (320 * scale + 0.5) && pixels_h == cast(s32) (240 * scale + 0.5), "pixel size %x% at scale %", pixels_w, pixels_h, scale);

    viewport: [4] s32;

    GL.gl.glGetIntegerv(GL.GL_VIEWPORT, viewport.data);
    assert(viewport[2] == pixels_w && viewport[3] == pixels_h, "the viewport must cover the framebuffer in pixels");

    // A 2x1 texture drawn eight times larger: nearest keeps two flat halves, linear blends across the middle.
    bitmap: Bitmap;

    bitmap_alloc(*bitmap, 2, 1, .RGBA8);

    bitmap.data.data[0] = 255;
    bitmap.data.data[1] = 0;
    bitmap.data.data[2] = 0;
    bitmap.data.data[3] = 255;
    bitmap.data.data[4] = 0;
    bitmap.data.data[5] = 0;
    bitmap.data.data[6] = 255;
    bitmap.data.data[7] = 255;

    source: Texture;

    assert(texture_load_from_bitmap(*source, *bitmap));

    target := texture_create_render_target(16, 4, .RGBA8);

    for filter: Texture_Filter.[.NEAREST, .LINEAR] {
        texture_set_filter(*source, filter);
        set_render_target(*target, .LEFT_HANDED);
        clear_render_target(0, 0, 0, 1);
        set_shader_for_images(*source);
        immediate_quad(.{ 0, 0 }, .{ 16, 0 }, .{ 16, 4 }, .{ 0, 4 }, .{ 1, 1, 1, 1 }, .{ 0, 0 }, .{ 1, 0 }, .{ 1, 1 }, .{ 0, 1 });
        immediate_flush();

        reader := pixel_read_begin(*target, .RGBA8, -1, -1, false, true);

        for 1..1000 {
            if pixel_read_is_done(reader) break;
            sleep_milliseconds(1);
        }

        result := pixel_read_end(reader);
        middle := result.data.data + 2 * result.stride + 7 * 4; // just left of the seam
        next := middle + 4;
        blended := (middle[2] > 20 && middle[0] > 20) || (next[2] > 20 && next[0] > 20);

        if filter == .NEAREST assert(!blended && middle[0] == 255 && next[2] == 255, "nearest filtering must keep hard edges");
        else assert(blended, "linear filtering blends across the seam");
    }

    // Fonts.
    args := get_command_line_arguments();

    defer free(args.data);
    assert(args.count == 2);
    bytes, ok := read_entire_file(args[1]);

    assert(ok);
    defer free(bytes);
    init_fonts(256, 256);

    plain := get_font_at_size(.{ bytes.count, bytes.data }, 20);
    sharp := get_font_at_size(.{ bytes.count, bytes.data }, 20, 2);

    assert(plain != null && sharp != null && plain != sharp);
    assert(plain.character_height == 20 && sharp.character_height == 40 && sharp.pixel_scale == 2);
    assert(get_font_at_size(.{ bytes.count, bytes.data }, 20, 2) == sharp);

    plain_width := prepare_text(plain, "Hello");
    sharp_width := prepare_text(sharp, "Hello");

    assert(plain_width > 0 && difference(plain_width, sharp_width) <= 3, "widths are in window units at any scale");
    assert(sharp.temporary_glyphs_width_in_pixels >= 2 * sharp_width - 2, "the run itself is laid out in pixels");

    generate_quads_for_prepared_text(sharp, 10, 30, 0);

    first := sharp.current_quads[0];

    assert(difference(cast(s64) ((first.p1.x - first.p0.x) * 2 + 0.5), cast(s64) first.glyph.width) <= 1, "glyph quads are drawn at half their pixel size");
}

difference :: (a: s64, b: s64) -> s64 {
    return ifx a > b then a - b else b - a;
}
