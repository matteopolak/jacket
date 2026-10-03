glyph_id_to_hash_key :: (x: u32) -> int {
    return cast(int) x;
}

isdigit :: (utf32: u32) -> bool {
    return utf32 >= #char "0" && utf32 <= #char "9";
}

is_latin :: (utf32: u32) -> bool {
    return utf32 <= 0x24f || (utf32 >= 0x2000 && utf32 <= 0x218f);
}

isLegalUTF8 :: (source: *u8, length: s64) -> bool {
    if !source || length < 1 || length > 4 return false;
    _, consumed, result := Unicode.character_utf8_to_utf32(source, length, true);
    return result == .CONVERSION_OK && consumed == length;
}

get_font_at_size :: (path: string, name: string, pixel_height: int) -> *Dynamic_Font {
    for entry: dynamic_fonts if entry.key == name && entry.font.character_height == pixel_height return entry.font;
    return load_font(path, name, pixel_height);
}

load_font :: (path: string, basename: string, pixel_height: int) -> font: *Dynamic_Font {
    persistent := font_context();

    push_context, defer_pop persistent;
    filename := Basic.sprint("%/%", path, basename);

    if path.count == 0 {
        Basic.free(filename);
        filename = Basic.copy_string(basename);
    }

    defer Basic.free(filename);
    loaded := make_loaded_font(filename);
    if !loaded return null;

    result := Basic.New(Dynamic_Font);

    if !initialize_dynamic_font(result, loaded.*, pixel_height) {
        Basic.free(result.name);
        Basic.free(result);
        return null;
    }

    Basic.array_add(*dynamic_fonts, Dynamic_Storage.{ result, loaded, Basic.copy_string(basename) });

    return result;
}

get_font_at_size :: (memory: [] u8, pixel_height: int) -> *Dynamic_Font {
    for entry: dynamic_fonts {
        if entry.loaded.data.data == memory.data && entry.loaded.data.count == memory.count && entry.font.character_height == pixel_height return entry.font;
    }

    persistent := font_context();

    push_context, defer_pop persistent;
    loaded: *Loaded_Font;

    for entry: loaded_fonts if entry.font.data.data == memory.data && entry.font.data.count == memory.count {
        loaded = entry.font;
        break;
    }

    if !loaded loaded = make_loaded_font("", memory);
    if !loaded return null;

    result := Basic.New(Dynamic_Font);

    if !initialize_dynamic_font(result, loaded.*, pixel_height) {
        Basic.free(result.name);
        Basic.free(result);
        return null;
    }

    Basic.array_add(*dynamic_fonts, Dynamic_Storage.{ result, loaded, "" });

    return result;
}

load_font_part_2 :: (result: *Dynamic_Font, loaded_font: Loaded_Font, pixel_height: int) {
    if !initialize_dynamic_font(result, loaded_font, pixel_height) return;
}

release_font :: (font: *Dynamic_Font) {
    // Reviewed API retains dynamic fonts/faces/atlas pools in module registries.
}

set_unknown_character :: (using font: *Dynamic_Font, utf32: u32) -> bool {
    index := FT_Get_Char_Index(private1, utf32);
    if index == 0 return false;

    glyph_index_for_unknown_character = index;
    utf32_for_unknown_character = utf32;

    return true;
}

get_baseline_height :: (font: *Dynamic_Font) -> s64 {
    return font.typical_ascender;
}

get_glyph_width_in_pixels :: (glyph: *Glyph_Data) -> s64 {
    return glyph.advance;
}

get_character_width_in_pixels :: (using font: *Dynamic_Font, utf8: *u8, effect_flags: Font_Effects) -> u32 {
    if !utf8 || utf8[0] == 0 return 0;

    count: s64 = 1;
    if utf8[0] >= 0xc2 && utf8[0] <= 0xdf count = 2;
    else if utf8[0] >= 0xe0 && utf8[0] <= 0xef count = 3;
    else if utf8[0] >= 0xf0 && utf8[0] <= 0xf4 count = 4;

    for index: 1..count - 1 if utf8[index] == 0 {
        count = index;
        break;
    }

    code, _, status := Unicode.character_utf8_to_utf32(utf8, count, true);
    if status != .CONVERSION_OK code = cast(u32) utf32_for_unknown_character;

    glyph := font_glyph(font, code, effect_flags);
    if !glyph return 0;

    return cast(u32) glyph.advance;
}

prepare_text :: (font: *Dynamic_Font, text: string, effect_flags: Font_Effects = 0) -> (width: s64) {
    convert_to_temporary_glyphs(font, text, effect_flags);
    return font.temporary_glyphs_width_in_pixels;
}

convert_to_temporary_glyphs :: (using font: *Dynamic_Font, s: string, effect_flags: Font_Effects) {
    persistent := font_context();

    push_context, defer_pop persistent;
    Basic.array_reset_keeping_memory(*temporary_glyphs);
    Basic.array_reset_keeping_memory(*temporary_glyphs_byte_offsets);
    temporary_glyphs_width_in_pixels = 0;
    glyph_conversion_failed = false;
    offset: int;
    while offset < s.count {
        code, count, status := Unicode.character_utf8_to_utf32(s.data + offset, s.count - offset, true);
        if status != .CONVERSION_OK {
            glyph_conversion_failed = true;
            code = cast(u32) utf32_for_unknown_character;
            count = 1;
        }

        glyph := font_glyph(font, code, effect_flags);
        if !glyph {
            Basic.array_reset_keeping_memory(*temporary_glyphs);
            Basic.array_reset_keeping_memory(*temporary_glyphs_byte_offsets);
            temporary_glyphs_width_in_pixels = 0;
            glyph_conversion_failed = true;

            return;
        }

        Basic.array_add(*temporary_glyphs, glyph);
        Basic.array_add(*temporary_glyphs_byte_offsets, offset);
        temporary_glyphs_width_in_pixels += glyph.advance;
        offset += count;
    }
}

accumulate_glyphs_from_font :: (source: *Dynamic_Font, dest: *Dynamic_Font) {
    persistent := font_context();
    push_context, defer_pop persistent;
    for glyph, key: source.glyph_lookup table_set(*dest.glyph_lookup, key, glyph);
}

generate_quads_for_prepared_text :: (font: *Dynamic_Font, x: s64, y: s64, effect_flags: Font_Effects) {
    persistent := font_context();

    push_context, defer_pop persistent;
    Basic.array_reset_keeping_memory(*font.current_quads);
    pen := x;

    for glyph: font.temporary_glyphs {
        x0 := cast(float) (pen + glyph.offset_x);
        y1 := cast(float) (y + glyph.offset_y);
        x1 := x0 + cast(float) glyph.width;
        y0 := y1 - cast(float) glyph.height;
        quad: Font_Quad;
        quad.glyph = glyph;
        quad.p0 = .{ x0, y0 };
        quad.p1 = .{ x1, y0 };
        quad.p2 = .{ x1, y1 };
        quad.p3 = .{ x0, y1 };
        quad.u0 = cast(float) glyph.x0 / cast(float) glyph.page.bitmap_data.width;
        quad.u1 = cast(float) (cast(int) glyph.x0 + glyph.width) / cast(float) glyph.page.bitmap_data.width;
        quad.v0 = cast(float) glyph.y0 / cast(float) glyph.page.bitmap_data.height;
        quad.v1 = cast(float) (cast(int) glyph.y0 + glyph.height) / cast(float) glyph.page.bitmap_data.height;
        Basic.array_add(*font.current_quads, quad);
        pen += glyph.advance;
    }
}

#scope_module
Dynamic_Storage :: struct {
    font: *Dynamic_Font;
    loaded: *Loaded_Font;
    key: string;
}

dynamic_fonts: [..] Dynamic_Storage;
font_pages: [..] *Font_Page;

font_context :: () -> #Context {
    persistent := context;
    persistent.allocator = context.default_allocator;
    assert(persistent.allocator.proc != null, "font pools require a long-lived default allocator");
    return persistent;
}

initialize_dynamic_font :: (result: *Dynamic_Font, loaded_font: Loaded_Font, pixel_height: int) -> bool {
    assert(result != null && loaded_font.face != null && pixel_height > 0 && pixel_height <= 32767);
    persistent := font_context();

    push_context, defer_pop persistent;
    status := FT_Set_Pixel_Sizes(loaded_font.face, 0, cast(u32) pixel_height);
    if status != 0 {
        report_font_error("FreeType font size selection failed", status);
        return false;
    }

    result.private1 = loaded_font.face;
    result.name = Basic.copy_string(loaded_font.name);
    result.character_height = pixel_height;
    face := result.private1;
    metrics := face.size.metrics;

    result.default_line_spacing = (metrics.height + 63) / 64;
    result.typical_ascender = metrics.ascender / 64;
    result.typical_descender = -metrics.descender / 64;
    result.max_ascender = (FT_MulFix(face.bbox.yMax, metrics.y_scale) + 63) / 64;
    result.max_descender = (-FT_MulFix(face.bbox.yMin, metrics.y_scale) + 63) / 64;
    result.em_width = metrics.x_ppem;
    glyph: u32;
    code := FT_Get_First_Char(face, *glyph);

    result.character_range_low = cast(int) code;

    while glyph != 0 {
        result.character_range_high = cast(int) code;
        code = FT_Get_Next_Char(face, code, *glyph);
    }

    if !set_unknown_character(result, 0xfffd) && !set_unknown_character(result, 0x2022) && !set_unknown_character(result, #char "?") {
        context.logger("font has no replacement glyph\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
    }

    return discover_font_features(result);
}

report_font_error :: (message: string, status: FT_Error) {
    context.logger(Basic.tprint("% (FreeType error %)\n", message, status), context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
}

font_glyph :: (font: *Dynamic_Font, utf32: u32, effects: Font_Effects) -> *Glyph_Data {
    index := FT_Get_Char_Index(font.private1, utf32);
    if font.latin_font && is_latin(utf32) && (font.use_other_font_for_latin_always || index == 0) return font_glyph(font.latin_font, utf32, effects);

    if index == 0 {
        font.glyph_conversion_failed = true;
        index = font.glyph_index_for_unknown_character;
    }

    if (effects & .LINING_FIGURES) != 0 && isdigit(utf32) index = apply_font_feature(font.feature_lining_numbers, index);
    if (effects & .SMALLCAPS) != 0 && !((effects & .LINING_FIGURES) != 0 && isdigit(utf32)) index = apply_font_feature(font.feature_smallcaps, index);

    key := glyph_id_to_hash_key(index);
    found, cached := table_find(*font.glyph_lookup, key);

    if found return cached;

    persistent := font_context();

    push_context, defer_pop persistent;
    status := FT_Set_Pixel_Sizes(font.private1, 0, cast(u32) font.character_height);
    if status != 0 {
        report_font_error("FreeType glyph pixel size failed", status);
        return null;
    }

    for page: font_pages {
        glyph, error, native_error := atlas_rasterize(page, font.private1, index, utf32);
        if error == .PAGE_FULL continue;

        if error != .NONE || native_error != 0 {
            report_font_error("glyph rasterization failed", native_error);
            return null;
        }

        table_set(*font.glyph_lookup, key, glyph);

        return glyph;
    }

    page := atlas_create_page(font_page_width, font_page_height);
    glyph, error, native_error := atlas_rasterize(page, font.private1, index, utf32);

    if error != .NONE || native_error != 0 {
        atlas_destroy_page(page);
        report_font_error("glyph does not fit configured atlas page or native rasterization failed", native_error);
        return null;
    }

    Basic.array_add(*font_pages, page);
    table_set(*font.glyph_lookup, key, glyph);

    return glyph;
}
