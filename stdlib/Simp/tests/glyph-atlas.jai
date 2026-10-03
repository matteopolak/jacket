#import "Basic";
Basic :: #import "Basic";
#import "freetype";
S :: #import "Simp";

Font_Page :: S.Font_Page;
Font_Line :: S.Font_Line;
Glyph_Data :: S.Glyph_Data;
Bitmap :: S.Bitmap;
bitmap_alloc :: S.bitmap_alloc;
deinit :: S.deinit;

#load "../../internal/rendering/glyph-atlas.jai";

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);
    assert(args.count == 2, "provide a caller-owned native font path");
    font := S.make_loaded_font(args[1]);

    assert(font != null);
    defer S.destroy(font);
    assert(FT_Set_Pixel_Sizes(font.face, 0, 24) == 0);
    tracking: Atlas_Tracking_Allocator;
    tracking.base = context.allocator;
    context.allocator = .{ atlas_tracking_allocator, *tracking };

    defer {
        context.allocator = tracking.base;
        assert(tracking.blocks == 0, "CPU atlas allocation leaked");
    }

    page := atlas_create_page(128, 128);

    defer atlas_destroy_page(page);
    native_index := FT_Get_Char_Index(font.face, #char "A");

    assert(native_index != 0);
    glyph, error, native_error := atlas_rasterize(page, font.face, native_index, #char "A");

    assert(error == .NONE && native_error == 0 && glyph != null);
    defer free(glyph);
    slot := font.face.glyph;

    assert(glyph.glyph_index_within_font == native_index && glyph.utf32 == #char "A");
    assert(glyph.width == slot.bitmap.width && glyph.height == slot.bitmap.rows);
    assert(glyph.advance == slot.advance.x / 64);
    assert(glyph.offset_x == slot.bitmap_left && glyph.offset_y == slot.bitmap_top);
    assert(glyph.page == page && page.dirty);
    assert(page.texture.gl_handle == 0, "CPU staging never fabricates a GPU resource");
    assert(glyph.x0 > 0 && glyph.y0 > 0);
    ink: u64;

    for row: 0..glyph.height - 1 {
        source := slot.bitmap.buffer + cast(int) row * slot.bitmap.pitch;
        dest := page.bitmap_data.data.data + (glyph.y0 + cast(int) row) * page.bitmap_data.stride + glyph.x0 * 4;

        assert(dest[-1] == 0 && dest[glyph.width * 4 + 3] == 0);

        for col: 0..glyph.width - 1 {
            assert(dest[col * 4] == 255 && dest[col * 4 + 1] == 255 && dest[col * 4 + 2] == 255);
            assert(dest[col * 4 + 3] == source[col]);
            ink += dest[col * 4 + 3];
        }
    }

    assert(ink > 0);
    first_end := glyph.x0 + cast(int) glyph.width;
    second, second_error, second_native_error := atlas_rasterize(page, font.face, native_index, #char "A");

    assert(second_error == .NONE && second_native_error == 0);
    assert(second.x0 > first_end && second.y0 == glyph.y0);
    free(second);
    tiny := atlas_create_page(4, 4);

    defer atlas_destroy_page(tiny);
    rejected, reason, rejected_native := atlas_rasterize(tiny, font.face, native_index, #char "A");

    assert(rejected == null && reason == .PAGE_FULL && rejected_native == 0);
    assert(tiny.lines.count == 0 && !tiny.dirty);

    for byte, index: tiny.bitmap_data.data {
        if index % 4 == 3 assert(byte == 0);
        else assert(byte == 255);
    }

    blank_index := FT_Get_Char_Index(font.face, #char " ");
    blank, blank_error, blank_native := atlas_rasterize(page, font.face, blank_index, #char " ");

    assert(blank_error == .NONE && blank_native == 0 && blank != null);
    assert(blank.width == 0 && blank.height == 0 && blank.advance > 0);
    free(blank);
    bad, bad_error, bad_native := atlas_rasterize(page, font.face, cast(u32) font.face.num_glyphs + 1, 0);

    assert(bad == null && bad_error == .NATIVE_FONT_ERROR && bad_native != 0);
    negative_data := u8.[3, 4, 0, 0, 1, 2];
    native_bitmap: FT_Bitmap;
    native_bitmap.width = 2;
    native_bitmap.rows = 2;
    native_bitmap.pitch = -4;
    native_bitmap.buffer = negative_data.data + 4;
    native_bitmap.pixel_mode = cast(u8) FT_Pixel_Mode.GRAY;
    destination: [8] u8;

    atlas_copy_coverage(*native_bitmap, destination.data, 4);
    assert(destination[0] == 1 && destination[1] == 2 && destination[4] == 3 && destination[5] == 4);
    assert(destination[2] == 0 && destination[3] == 0);
    mono := u8.[0x81, 0x42];

    native_bitmap.width = 8;
    native_bitmap.pitch = 1;
    native_bitmap.buffer = mono.data;
    native_bitmap.pixel_mode = cast(u8) FT_Pixel_Mode.MONO;
    expanded: [20] u8;

    atlas_copy_coverage(*native_bitmap, expanded.data, 10);

    for col: 0..7 {
        expected: u8 = 0;
        if col == 0 || col == 7 expected = 255;

        assert(expanded[col] == expected);
        expected = 0;
        if col == 1 || col == 6 expected = 255;

        assert(expanded[10 + col] == expected);
    }

    print("ok\n");
}

#scope_module
Atlas_Tracking_Allocator :: struct {
    base: Allocator;
    blocks: int;
}

atlas_tracking_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Atlas_Tracking_Allocator) allocator_data;
    result := state.base.proc(mode, size, old_size, old_memory, state.base.data);

    if (mode == .ALLOCATE || mode == .RESIZE) && old_memory == null && result != null state.blocks += 1;
    if mode == .FREE && old_memory != null state.blocks -= 1;

    return result;
}
