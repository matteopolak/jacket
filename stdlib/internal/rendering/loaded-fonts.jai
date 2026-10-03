init_fonts :: (_page_size_x: s32 = -1, _page_size_y: s32 = -1) {
    if _page_size_x >= 0 || _page_size_y >= 0 {
        assert(_page_size_x >= 64 && _page_size_y >= 64);
        font_page_width = _page_size_x;
        font_page_height = _page_size_y;
    }

    if font_library return;

    status := FT_Init_FreeType(*font_library);

    assert(status == 0 && font_library != null, "FreeType initialization failed");
}

ensure_fonts_are_initted :: () {
    if !font_library init_fonts();
}

find_loaded_font :: (filename: string) -> *Loaded_Font {
    for storage: loaded_fonts if storage.key == filename return storage.font;
    return null;
}

make_loaded_font :: (name: string, data: [] u8) -> *Loaded_Font {
    ensure_fonts_are_initted();
    if data.count <= 0 || !data.data return null;

    #if size_of(FT_Long) == 4 {
        if data.count > 0x7fffffff return null;
    }

    face: FT_Face;
    status := FT_New_Memory_Face(font_library, data.data, cast(FT_Long) data.count, 0, *face);

    if status != 0 return null;

    name_copy := Basic.copy_string(name);
    name_allocator := context.allocator;
    persistent := context;
    persistent.allocator = context.default_allocator;
    result: *Loaded_Font;

    push_context persistent {
        result = Basic.New(Loaded_Font);
        result.name = name_copy;
        result.face = face;
        result.data = data;
        Basic.array_add(*loaded_fonts, Font_Storage.{ result, name_allocator, Basic.copy_string(name) });
    }

    return result;
}

make_loaded_font :: (filename: string) -> (font: *Loaded_Font) {
    existing := find_loaded_font(filename);
    if existing return existing;

    persistent := context;
    persistent.allocator = context.default_allocator;
    push_context, defer_pop persistent;
    bytes, ok := File.read_entire_file(filename, log_errors = false);
    if !ok return null;

    result := make_loaded_font(filename, .{ bytes.count, bytes.data });
    if !result Basic.free(bytes);

    return result;
}

destroy :: (font: *Loaded_Font) {
    if !font || !font.name.data return;

    allocator := context.allocator;

    for storage: loaded_fonts if storage.font == font {
        allocator = storage.allocator;
        break;
    }

    allocator.proc(.FREE, 0, 0, font.name.data, allocator.data);
    font.name = "";
}

#scope_module
font_library: FT_Library;
font_page_width: s32 = 2048;
font_page_height: s32 = 1024;

Font_Storage :: struct {
    font: *Loaded_Font;
    allocator: Allocator;
    key: string;
}

loaded_fonts: [..] Font_Storage;
