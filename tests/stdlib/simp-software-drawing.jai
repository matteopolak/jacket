// Simp with its CPU backend, checked pixel by pixel: triangles, fans, rounded rectangles, textured
// quads and gradients; windows supplied by the host through the size, target and present hooks;
// image files (PNG/BMP/TGA/JPG) and textures; and fonts (measuring, cursor positions, drawing text,
// highlights and code). Fonts come from the operating system (no font ships with the repository);
// without one the font part is left out. Images and fonts use stb_image and FreeType, which are not
// built for Windows, so Windows runs the drawing and window parts only. The wasm targets have no
// native libraries at all (see the skip list).
#import "Basic";
#import "Math";

S :: #import "Simp"(render_api = .SOFTWARE);
#import "File";
#import "Window_Type";

SCRATCH :: ".scratch/simp-software-drawing";

// RGBA of pixel (x, y), counting rows from the top of the stored image.
pixel :: (bitmap: *S.Bitmap, x: int, y: int) -> [4] u8 {
    p := bitmap.data.data + y * bitmap.stride + x * 4;
    return .[p[0], p[1], p[2], p[3]];
}
rgb_is :: (bitmap: *S.Bitmap, x: int, y: int, r: u8, g: u8, b: u8) -> bool {
    p := pixel(bitmap, x, y);
    return p[0] == r && p[1] == g && p[2] == b;
}
lit_count :: (bitmap: *S.Bitmap) -> int {
    count := 0;
    for y: 0..bitmap.height - 1 for x : 0..bitmap.width - 1 if pixel(bitmap, x, y)[0] > 0 count += 1;
    return count;
}

// A host window: the program owns the pixels and presents them itself.
FAKE_WINDOW :: cast(Window_Type) 77;
host_pixels: S.Bitmap;
host_width: s32 = 8;
host_height: s32 = 6;
presented: int;
host_size :: (window: Window_Type) -> s32, s32 {
    return host_width, host_height;
}
host_target :: (window: Window_Type) -> *S.Bitmap {
    if window != FAKE_WINDOW return null;
    return *host_pixels;
}
host_present :: (window: Window_Type, vsync: bool) -> bool {
    presented += 1;
    return window == FAKE_WINDOW;
}

main :: () {
    target := S.texture_create_render_target(16, 16, .RGBA8);
    assert(target != null);
    image := S.texture_bitmap(target);
    S.set_render_target(target, .LEFT_HANDED);
    S.CheckInit();

    // Triangles: one per overload, then a fan covering a square.
    S.set_shader_for_color();
    S.clear_render_target(0, 0, 0, 1);
    S.immediate_begin();
    S.immediate_triangle(.{ 0, 0, 0 }, .{ 8, 0, 0 }, .{ 0, 8, 0 }, .{ 1, 0, 0, 1 });
    S.immediate_triangle(.{ 16, 16, 0 }, .{ 8, 16, 0 }, .{ 16, 8, 0 }, .{ 0, 1, 0, 1 }, .{ 0, 1, 0, 1 }, .{ 0, 1, 0, 1 });
    S.immediate_flush();
    assert(rgb_is(image, 1, 1, 255, 0, 0), "red triangle at its corner: %", pixel(image, 1, 1));
    assert(rgb_is(image, 14, 14, 0, 255, 0), "green triangle at its corner: %", pixel(image, 14, 14));
    assert(rgb_is(image, 14, 1, 0, 0, 0) && rgb_is(image, 8, 8, 0, 0, 0), "triangles drew outside themselves");
    S.clear_render_target(0, 0, 0, 1);
    square := Vector2.[.{ 4, 4 }, .{ 12, 4 }, .{ 12, 12 }, .{ 4, 12 }, .{ 4, 4 }];
    S.immediate_triangle_fan(.{ 8, 8 }, square, .{ 0, 0, 1, 1 }, .{ 0, 0, 1, 1 });
    S.immediate_triangle_fan(.{ 8, 8 }, Vector2.[.{ 4, 4 }], .{ 1, 1, 1, 1 }, .{ 1, 1, 1, 1 }); // Too few points: nothing.
    S.immediate_flush();
    assert(rgb_is(image, 5, 5, 0, 0, 255) && rgb_is(image, 11, 10, 0, 0, 255), "fan interior: %", pixel(image, 5, 5));
    assert(rgb_is(image, 2, 2, 0, 0, 0) && rgb_is(image, 13, 8, 0, 0, 0), "fan spilled outside its square");

    // A rounded rectangle leaves its corners empty and fills its middle.
    S.clear_render_target(0, 0, 0, 1);
    S.set_shader_for_rects();
    S.immediate_rounded_rect(.{ 0, 0 }, .{ 16, 16 }, .{ 1, 1, 0, 1 }, .{ 8, 8 }, .{ 8, 8 }, 6);
    S.immediate_flush();
    assert(rgb_is(image, 8, 8, 255, 255, 0) && rgb_is(image, 8, 0, 255, 255, 0), "rounded rect body: %", pixel(image, 8, 8));
    assert(pixel(image, 0, 0)[0] < 40, "rounded rect corner is filled: %", pixel(image, 0, 0));

    // A gradient ramps red down the rectangle (mode 16: red by v).
    S.clear_render_target(0, 0, 0, 1);
    S.set_shader_for_gradient(.{ 0, 0, 16, 16 }, 16, 0);
    S.immediate_quad(0, 0, 16, 16, .{ 1, 1, 1, 1 });
    S.immediate_flush();
    top := pixel(image, 4, 0)[0];
    bottom := pixel(image, 4, 15)[0];
    assert(top < 20 && bottom > 235 && pixel(image, 4, 8)[0] > top, "gradient red % .. %", top, bottom);

    // Textured quads sample the image, each corner of a 2x2 texture landing on its quadrant.
    texels: S.Bitmap;
    S.bitmap_alloc(*texels, 2, 2, .RGBA8);
    colors := u8.[255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255];
    memcpy(texels.data.data, colors.data, 16);
    sprite: S.Texture;
    assert(S.texture_load_from_bitmap(*sprite, *texels), "texture_load_from_bitmap failed");
    assert(sprite.width == 2 && sprite.height == 2 && sprite.format == .RGBA8);
    S.clear_render_target(0, 0, 0, 1);
    S.set_shader_for_images(*sprite);
    S.immediate_quad(0, 0, 16, 16, .{ 1, 1, 1, 1 });
    S.immediate_flush();
    assert(rgb_is(image, 2, 2, 255, 0, 0) && rgb_is(image, 13, 2, 0, 255, 0), "top texels: % %", pixel(image, 2, 2), pixel(image, 13, 2));
    assert(rgb_is(image, 2, 13, 0, 0, 255) && rgb_is(image, 13, 13, 255, 255, 255), "bottom texels: % %", pixel(image, 2, 13), pixel(image, 13, 13));
    bad := texels;
    bad.stride = 4; // Rows shorter than the pixels need.
    refused: S.Texture;
    assert(!S.texture_load_from_bitmap(*refused, *bad) && !S.texture_load_from_bitmap(*refused, null));
    // With no shader there is nothing to bind a texture to: it is refused, not a crash.
    S.immediate_reset();
    S.immediate_set_texture(*sprite);

    // Host-supplied windows: the size, the pixels and the presentation come from the program.
    S.bitmap_alloc(*host_pixels, host_width, host_height, .RGBA8);
    S.window_dimension_query = host_size;
    S.software_window_target = host_target;
    S.window_present_sink = host_present;
    assert(S.prepare_window(FAKE_WINDOW, 4) == 0, "the CPU backend granted multisampling");
    S.set_render_target(FAKE_WINDOW, .LEFT_HANDED);
    w, h := S.get_render_dimensions(FAKE_WINDOW);
    assert(w == 8 && h == 6, "host window measures % x %", w, h);
    S.set_shader_for_color();
    S.clear_render_target(0, 0, 0, 1);
    S.immediate_quad(0, 0, 8, 6, .{ 0, 1, 1, 1 });
    S.swap_buffers(FAKE_WINDOW);
    assert(presented == 1 && rgb_is(*host_pixels, 7, 5, 0, 255, 255), "presented % times, corner %", presented, pixel(*host_pixels, 7, 5));
    // The window shrinks: after update_window the projection follows, so a full-window quad
    // covers only the new area.
    host_width, host_height = 4, 3;
    S.update_window(FAKE_WINDOW);
    S.clear_render_target(0, 0, 0, 1);
    S.immediate_quad(0, 0, 4, 3, .{ 1, 0, 1, 1 });
    S.swap_buffers(FAKE_WINDOW);
    assert(rgb_is(*host_pixels, 3, 2, 255, 0, 255) && rgb_is(*host_pixels, 7, 5, 0, 0, 0), "after update_window: % %", pixel(*host_pixels, 3, 2), pixel(*host_pixels, 7, 5));
    assert(presented == 2);
    // Without a size query, set_window_dimensions is what the window measures.
    S.window_dimension_query = null;
    S.set_window_dimensions(FAKE_WINDOW, 5, 2);
    w, h = S.get_render_dimensions(FAKE_WINDOW);
    assert(w == 5 && h == 2, "set_window_dimensions gave % x %", w, h);
    S.set_window_dimensions(cast(Window_Type) 78, -3, 9);
    w, h = S.get_render_dimensions(cast(Window_Type) 78);
    assert(w == 0 && h == 9, "negative sizes clamp: % x %", w, h);

    S.set_render_target(target, .LEFT_HANDED);
    #if OS != .WINDOWS {
        images(target);
        fonts(target);
    }
    S.deinit(*sprite);
    S.deinit(*texels);
    S.deinit(target);
    free(target);
    print("ok\n");
}

#if OS != .WINDOWS {
    images :: (target: *S.Texture) {
        make_directory_if_it_does_not_exist(".scratch");
        make_directory_if_it_does_not_exist(SCRATCH);
        source: S.Bitmap;
        S.bitmap_alloc(*source, 3, 2, .RGBA8);
        for 0..source.data.count - 1 source.data[it] = cast(u8) (it * 10);
        png := tprint("%/pixels.png", SCRATCH);
        assert(S.bitmap_save(*source, png, true), "bitmap_save png");
        for extension: string.["bmp", "tga", "jpg"] {
            assert(S.bitmap_save(*source, tprint("%/pixels.%", SCRATCH, extension)), "bitmap_save %", extension);
        }
        assert(!S.bitmap_save(*source, tprint("%/pixels.gif", SCRATCH)), "saved an unknown format");
        bgra: S.Bitmap;
        S.bitmap_alloc(*bgra, 1, 1, .BGRA8);
        assert(!S.bitmap_save(*bgra, png), "saved a BGRA8 bitmap");

        loaded: S.Bitmap;
        assert(S.bitmap_load(*loaded, png), "bitmap_load %", png);
        assert(loaded.width == 3 && loaded.height == 2 && loaded.format == .RGBA8, "loaded % x % %", loaded.width, loaded.height, loaded.format);
        for 0..source.data.count - 1 assert(loaded.data[it] == source.data[it], "png byte % is %, saved %", it, loaded.data[it], source.data[it]);
        bmp: S.Bitmap;
        assert(S.bitmap_load(*bmp, tprint("%/pixels.bmp", SCRATCH)) && bmp.width == 3 && bmp.height == 2);
        assert(!S.bitmap_load(*bmp, tprint("%/missing.png", SCRATCH)), "loaded a missing file");

        bytes, read := read_entire_file(png);
        assert(read);
        memory := cast([] u8) bytes;
        from_memory: S.Bitmap;
        assert(S.bitmap_load_from_memory(*from_memory, memory) && from_memory.data[5] == source.data[5]);
        assert(!S.bitmap_load_from_memory(*from_memory, .[]) && !S.bitmap_load_from_memory(*from_memory, cast([] u8) "not an image"));

        file_texture: S.Texture;
        assert(S.texture_load_from_file(*file_texture, png, srgb = true), "texture_load_from_file");
        assert(file_texture.width == 3 && file_texture.height == 2 && (file_texture.flags & .sRGB) != 0);
        memory_texture: S.Texture;
        assert(S.texture_load_from_memory(*memory_texture, memory, build_mipmaps = true), "texture_load_from_memory");
        assert(memory_texture.width == 3 && memory_texture.mipmap_count == 2, "mipmaps %", memory_texture.mipmap_count);
        assert(!S.texture_load_from_file(*file_texture, tprint("%/missing.png", SCRATCH)));
        assert(!S.texture_load_from_memory(*file_texture, cast([] u8) "nope"));

        for extension: string.["png", "bmp", "tga", "jpg"] file_delete(tprint("%/pixels.%", SCRATCH, extension));
        S.deinit(*file_texture);
        S.deinit(*memory_texture);
        S.deinit(*source);
        S.deinit(*bgra);
        S.deinit(*loaded);
        S.deinit(*bmp);
        S.deinit(*from_memory);
        free(bytes);
    }

    FONT_CANDIDATES :: string.[
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    ];

    fonts :: (target: *S.Texture) {
        found := "";
        for FONT_CANDIDATES if file_exists(it) {
            found = it;
            break;
        }
        #if OS == .MACOS assert(found.count > 0, "macOS ships Arial");
        if !found {
            print("no system font found: font drawing not checked\n");
            return;
        }
        cut := found.count - 1;
        while found[cut] != #char "/" cut -= 1;
        directory := string.{ cut, found.data };
        name := string.{ found.count - cut - 1, found.data + cut + 1 };

        S.init_fonts(512, 512);
        font := S.get_font_at_size(directory, name, 16);
        assert(font != null, "get_font_at_size(%, %)", directory, name);
        assert(S.get_font_at_size(directory, name, 16) == font, "fonts are not cached by name and size");
        assert(S.get_font_at_size(directory, "no-such-font.ttf", 16) == null);

        // Widths grow with the text, and the prepared width is the last measurement.
        a := S.get_text_width(font, "i");
        b := S.get_text_width(font, "ii");
        c := S.get_text_width(font, "iiW");
        assert(a > 0 && b > a && c > b, "widths % % %", a, b, c);
        assert(S.get_prepared_text_width(font) == c);
        assert(S.convert_to_temporary_glyphs(font, "ii") == b && font.temporary_glyphs.count == 2);
        assert(S.get_text_width(font, "") == 0);
        // Cursor positions are byte offsets; a width inside a glyph's first half stays before it.
        assert(S.get_cursor_pos_for_width(font, "abc", 0) == 0);
        assert(S.get_cursor_pos_for_width(font, "abc", 10000) == 3);
        ab := S.get_text_width(font, "ab");
        assert(S.get_cursor_pos_for_width(font, "abc", ab + 1) == 2, "cursor after ab: %", S.get_cursor_pos_for_width(font, "abc", ab + 1));
        assert(S.get_cursor_pos_for_width(font, "éa", S.get_text_width(font, "é") + 1) == 2, "a two-byte character is one cursor step");

        assert(S.is_latin(#char "A") && S.is_latin(0xe9) && !S.is_latin(0x65e5));
        assert(S.set_unknown_character(font, #char "?"), "the font has no '?'");
        assert(!S.set_unknown_character(font, 0x10fff0), "a private-use codepoint resolved");
        assert(!S.set_unknown_character(null, #char "?"));
        S.prepare_text(font, "a\xf4\x8f\xbf\xb0");
        assert(font.glyph_conversion_failed, "an unknown character converted");

        // Drawing: white text on black lights pixels inside the text's box only.
        image := S.texture_bitmap(target);
        S.clear_render_target(0, 0, 0, 1);
        width := S.draw_text(font, 1, 13, "Hi", .{ 1, 1, 1, 1 });
        assert(width == S.get_text_width(font, "Hi"));
        lit := lit_count(image);
        assert(lit > 10, "draw_text lit % pixels", lit);
        for y: 0..15 for x : 0..15 {
            if pixel(image, x, y)[0] == 0 continue;
            assert(x >= 1 && x <= 1 + width + 1, "text pixel at column % (width %)", x, width);
        }
        // Quads: one per inked glyph, in the order of the text.
        S.prepare_text(font, "H i");
        S.generate_quads_for_prepared_text(font, 0, 13);
        assert(font.current_quads.count == 2, "% quads for two inked glyphs", font.current_quads.count);
        assert(font.current_quads[0].p0.x < font.current_quads[1].p0.x);
        S.clear_render_target(0, 0, 0, 1);
        S.draw_generated_quads(font, Vector4.{ 1, 0, 0, 1 });
        assert(lit_count(image) > 0);
        S.clear_render_target(0, 0, 0, 1);
        S.draw_prepared_text(font, 0, 13, .{ 1, 1, 1, 1 });
        assert(lit_count(image) > 0);
        S.clear_render_target(0, 0, 0, 1);
        S.immediate_letter_quad(font.current_quads[0], .{ 1, 1, 1, 1 });
        S.immediate_flush();
        assert(lit_count(image) > 0, "immediate_letter_quad drew nothing");

        // Color-map drawing uses the installed table.
        table := Vector4.[.{ 1, 0, 0, 1 }, .{ 0, 0, 1, 1 }, .{ 0, 1, 0, 1 }];
        S.set_color_map(table);
        S.clear_render_target(0, 0, 0, 1);
        S.draw_text(font, 1, 13, "H", cast(u8) 2);
        green_only := true;
        for y: 0..15 for x : 0..15 {
            p := pixel(image, x, y);
            if p[0] > 0 || p[2] > 0 green_only = false;
        }
        assert(green_only && lit_count(image) == 0, "color index 2 is not green");
        S.prepare_text(font, "H");
        S.draw_prepared_text(font, 1, 13, cast(u8) 0);
        assert(lit_count(image) > 0, "color index 0 is not red");
        S.generate_quads_for_prepared_text(font, 1, 13);
        S.draw_generated_quads(font, cast(u8) 1);
        S.draw_generated_quads(font, cast(u8) 200); // Outside the table: ignored.

        // Highlights pick the highlight color per glyph.
        S.clear_render_target(0, 0, 0, 1);
        marks := bool.[false, true];
        highlighted := S.draw_text_with_highlights(font, 0, 13, "HH", marks, 0, 1);
        assert(highlighted == S.get_text_width(font, "HH"));
        red_left, blue_right := false, false;
        half := highlighted / 2;
        for y: 0..15 for x : 0..15 {
            p := pixel(image, x, y);
            if x < half && p[0] > 0 red_left = true;
            if x >= half && p[2] > 0 blue_right = true;
        }
        assert(red_left && blue_right, "highlights: red left %, blue right %", red_left, blue_right);
        assert(S.draw_text_with_highlights(font, 0, 13, "H", marks, 7, 0) == 0, "a color outside the map drew");

        // Code: tabs advance to tab stops, tokens pick colors, selections get a backdrop.
        S.set_show_tab_characters_in_selection(false);
        tokens := u8.[0, 0, 2];
        lines := S.Code_Line.[.{ line = "a\tb", tokens = tokens }];
        selection := S.Offset_Range.[.{ 0, 0 }];
        S.clear_render_target(0, 0, 0, 1);
        S.set_render_target(target, .RIGHT_HANDED);
        S.draw_code(font, 0, 4, 2, 16, lines, selection, tab_size = 4);
        image = S.texture_bitmap(target);
        backdrop := false;
        green := false;
        for y: 0..15 for x : 0..15 {
            p := pixel(image, x, y);
            if x < 2 && p[2] == 255 backdrop = true;
            if x >= 8 && p[1] > 0 green = true;
        }
        assert(backdrop, "the selected character has no backdrop");
        assert(green, "the token colored 2 after the tab stop is not green");
        S.set_show_tab_characters_in_selection(true);
        S.set_render_target(target, .LEFT_HANDED);

        // Rendering options change the glyphs that get rasterized, not the layout.
        S.set_hinting(false);
        unhinted := S.get_text_width(font, "Hi");
        S.set_anti_aliasing(.lcd);
        S.clear_render_target(0, 0, 0, 1);
        S.draw_text(font, 1, 13, "Hi");
        assert(lit_count(image) > 0, "LCD text drew nothing");
        S.set_anti_aliasing(.normal);
        S.set_hinting(true);
        assert(abs(unhinted - width) <= 2, "hinting changed the width from % to %", width, unhinted);

        // Fonts from memory, released one by one or all at once.
        bytes, read := read_entire_file(found);
        assert(read);
        memory := cast([] u8) bytes;
        small := S.get_font_at_size(memory, 10);
        assert(small != null && small != font && S.get_text_width(small, "Hi") < width);
        S.release_font(small);
        S.deinit(small);
        assert(S.get_text_width(small, "Hi") == 0, "a deinitialized font still measures");
        loaded := S.make_loaded_font("copy", memory);
        assert(loaded != null && loaded.name == "copy");
        S.destroy(loaded);
        free(loaded);
        S.deinit_fonts();
        // Starting over works after a full release.
        again := S.get_font_at_size(directory, name, 12);
        assert(again != null && S.get_text_width(again, "Hi") > 0);
        S.deinit_fonts();
        free(bytes);
        S.set_color_map(.[]);
    }
}
