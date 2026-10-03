#scope_module

// CPU atlas ownership belongs to its font allocator. GPU upload/teardown is a
// separate renderer operation: a dirty page owns real pixels, never a GL token.
Atlas_Error :: enum u8 {
    NONE;
    INVALID_PAGE;
    NATIVE_FONT_ERROR;
    UNSUPPORTED_BITMAP;
    PAGE_FULL;
    METRIC_RANGE;
}

atlas_create_page :: (width: s32, height: s32) -> *Font_Page {
    assert(width >= 3 && height >= 3 && width <= 32767 && height <= 32767);
    page := Basic.New(Font_Page);
    page.bitmap_data = Basic.New(Bitmap);
    bitmap_alloc(page.bitmap_data, width, height, .RGBA8);

    for *byte, index: page.bitmap_data.data {
        byte.* = 255;
        if index % 4 == 3 byte.* = 0;
    }

    page.texture.width = width;
    page.texture.height = height;
    page.texture.format = .RGBA8;

    return page;
}

// Call after releasing any renderer-owned texture. Glyph records are owned by
// the font's lookup and must be freed before releasing their referenced page.
atlas_destroy_page :: (page: *Font_Page) {
    if !page return;
    for line: page.lines Basic.free(line);

    Basic.array_free(page.lines);
    deinit(page.bitmap_data);
    Basic.free(page.bitmap_data);
    Basic.free(page);
}

atlas_rasterize :: (page: *Font_Page, face: FT_Face, glyph_index: u32, utf32: u32) -> (glyph: *Glyph_Data, error: Atlas_Error, native_error: FT_Error) {
    if !page || !page.bitmap_data || !face return null, .INVALID_PAGE, 0;

    bitmap := page.bitmap_data;
    if bitmap.format != .RGBA8 || bitmap.width < 3 || bitmap.height < 3 || !bitmap.data.data return null, .INVALID_PAGE, 0;
    if bitmap.stride < bitmap.width * 4 || bitmap.data.count < cast(s64) bitmap.stride * bitmap.height return null, .INVALID_PAGE, 0;

    status := FT_Load_Glyph(face, glyph_index, FT_LOAD_RENDER);
    if status != 0 return null, .NATIVE_FONT_ERROR, status;

    slot := face.glyph;
    source := *slot.bitmap;

    if source.width != 0 && source.rows != 0 {
        if !source.buffer return null, .UNSUPPORTED_BITMAP, 0;
        if source.pixel_mode != cast(u8) FT_Pixel_Mode.GRAY && source.pixel_mode != cast(u8) FT_Pixel_Mode.MONO return null, .UNSUPPORTED_BITMAP, 0;
        if source.pixel_mode == cast(u8) FT_Pixel_Mode.GRAY && source.num_grays != 256 return null, .UNSUPPORTED_BITMAP, 0;
    }

    advance := slot.advance.x / 64;
    if slot.bitmap_left < -32768 || slot.bitmap_left > 32767 || slot.bitmap_top < -32768 || slot.bitmap_top > 32767 || advance < -32768 || advance > 32767 return null, .METRIC_RANGE, 0;
    if cast(u64) source.width + 2 > cast(u64) bitmap.width || cast(u64) source.rows + 2 > cast(u64) bitmap.height return null, .PAGE_FULL, 0;

    width := cast(int) source.width;
    height := cast(int) source.rows;
    row_bytes := width;

    if source.pixel_mode == cast(u8) FT_Pixel_Mode.MONO row_bytes = (width + 7) / 8;

    absolute_pitch := cast(s64) source.pitch;
    if absolute_pitch < 0 absolute_pitch = -absolute_pitch;
    if height > 0 && absolute_pitch < row_bytes return null, .UNSUPPORTED_BITMAP, 0;

    // Shelf packing keeps one transparent pixel around each coverage bitmap.
    line: *Font_Line;

    for candidate: page.lines {
        if candidate.height >= height + 2 && candidate.height * 7 <= (height + 2) * 10 && cast(int) candidate.bitmap_cursor_x + width + 2 <= bitmap.width {
            line = candidate;
            break;
        }
    }

    if !line {
        row_height := (height * 11 + 9) / 10 + 2;
        if cast(int) page.line_cursor_y + row_height > bitmap.height return null, .PAGE_FULL, 0;

        line = Basic.New(Font_Line);
        line.page = page;
        line.bitmap_cursor_y = page.line_cursor_y;
        line.height = row_height;
        page.line_cursor_y = cast(s16) (cast(int) page.line_cursor_y + line.height);
        Basic.array_add(*page.lines, line);
    }

    glyph := Basic.New(Glyph_Data);
    glyph.utf32 = utf32;
    glyph.glyph_index_within_font = glyph_index;
    glyph.page = page;
    glyph.x0 = line.bitmap_cursor_x + 1;
    glyph.y0 = line.bitmap_cursor_y + 1;
    glyph.width = source.width;
    glyph.height = source.rows;
    glyph.offset_x = cast(s16) slot.bitmap_left;
    glyph.offset_y = cast(s16) slot.bitmap_top;
    glyph.ascent = cast(s16) slot.bitmap_top;
    glyph.advance = cast(s16) advance;
    destination := bitmap.data.data + cast(int) glyph.y0 * bitmap.stride + glyph.x0 * 4;

    atlas_copy_coverage(source, destination, bitmap.stride, 4);
    line.bitmap_cursor_x = cast(s16) (cast(int) line.bitmap_cursor_x + width + 2);
    page.dirty = true;

    return glyph, .NONE, 0;
}

atlas_copy_coverage :: (source: *FT_Bitmap, destination: *u8, destination_stride: int, channels: int = 1) {
    width := cast(int) source.width;
    height := cast(int) source.rows;

    for row: 0..height - 1 {
        // FreeType defines signed pitch as the offset to the next downward row.
        from := source.buffer + row * source.pitch;
        to := destination + row * destination_stride;

        for col: 0..width - 1 {
            coverage: u8;
            if source.pixel_mode == cast(u8) FT_Pixel_Mode.GRAY coverage = from[col];

            if source.pixel_mode == cast(u8) FT_Pixel_Mode.MONO {
                bit := from[col / 8] & (cast(u8) 0x80 >> (col % 8));
                coverage = ifx bit != 0 then 255 else 0;
            }

            to[col * channels + channels - 1] = coverage;
        }
    }
}
