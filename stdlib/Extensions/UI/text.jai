// Fonts, glyph rasterizing (FreeType), the glyph atlas and text measuring.
//
// Sizes are logical points. Glyphs are rasterized at `size * scale` pixels so text is sharp on a
// high-density display, and measured back in points. A font without a face (`face == null`) has
// synthetic fixed metrics: layout and tests work without any font file or native library.

// Which font a piece of text uses.
Font_Slot :: enum u8 {
    SANS; // the interface font
    MONO; // the code font
}

// How text is drawn.
Text_Style :: struct {
    font: Font_Slot;
    size: float = 14; // points
    color: Color = .{ 1, 1, 1, 1 };
}

// A loaded font file. Created by `load_font` and owned by the `Text_System`.
Font :: struct {
    face: FT_Face;
    data: [..] u8; // the file, which FreeType reads from for the face's lifetime
    name: string;
}

// Where a glyph is in the atlas and how to place it.
Glyph :: struct {
    advance: float; // pixels
    left: float; // pixels from the pen to the bitmap's left edge
    top: float; // pixels from the baseline up to the bitmap's top edge
    width: float; // bitmap size in pixels
    height: float;
    u0: float;
    v0: float;
    u1: float;
    v1: float;
    visible: bool; // false: nothing to draw (a space)
}

// Vertical metrics in points.
Font_Metrics :: struct {
    ascent: float;
    descent: float;
    line_height: float;
}

// The glyph atlas: a one-channel texture filled row by row, uploaded by the renderer when
// `version` changes.
Atlas :: struct {
    width: int = 1024;
    height: int = 1024;
    pixels: [..] u8;
    pen_x: int;
    pen_y: int;
    row_height: int;
    version: int; // changes whenever pixels do
    full: bool; // ran out of room; the cache is cleared at the start of the next frame
}

// Everything text needs: fonts, the glyph cache and the atlas.
Text_System :: struct {
    library: FT_Library;
    fonts: [..] *Font; // 0 is SANS, 1 is MONO, the rest are fallbacks for missing glyphs
    glyphs: Table(u64, Glyph);
    metrics: Table(u64, Font_Metrics);
    atlas: Atlas;
    scale: float = 1; // device pixels per point
}

// Open the font file at `path` into the next slot. Returns false when it cannot be read or parsed.
//
// - text: the text system to add the font to.
// - path: the font file to read.
load_font :: (text: *Text_System, path: string) -> bool {
    data, ok := read_entire_file(path);
    if !ok return false;

    return load_font_data(text, data, path);
}

// Open a font from bytes (which the text system keeps and frees).
//
// - text: the text system to add the font to.
// - data: the font file's bytes. The call takes ownership: it copies them and frees `data`, even on
//   failure.
// - name: a label for the font, copied.
load_font_data :: (text: *Text_System, data: string, name: string = "") -> bool {
    if !text.library {
        if FT_Init_FreeType(*text.library) != 0 {
            free(data.data);
            return false;
        }
    }

    font := New(Font);

    array_resize(*font.data, data.count);
    memcpy(font.data.data, data.data, data.count);
    free(data.data);
    font.name = copy_string(name);

    if FT_New_Memory_Face(text.library, font.data.data, cast(FT_Long) font.data.count, 0, *font.face) != 0 {
        array_free(font.data);
        free(font);
        return false;
    }

    array_add(*text.fonts, font);

    return true;
}

// Add a fixed-metrics font (no glyphs are drawn) to the next slot.
//
// - text: the text system to add the font to.
add_synthetic_font :: (text: *Text_System) {
    array_add(*text.fonts, New(Font));
}

// Load the platform's usual fonts: a sans font into slot 0, a monospace font into slot 1, then a
// few fallbacks for symbols and CJK. Any that are missing are replaced by synthetic fonts for the
// first two slots. The environment variables `JAI_UI_FONT` and `JAI_UI_MONO_FONT` name font files
// that replace the sans and monospace choices.
//
// - text: the text system to fill.
load_system_fonts :: (text: *Text_System) {
    sans := environment_variable("JAI_UI_FONT");
    mono := environment_variable("JAI_UI_MONO_FONT");

    sans_paths: [..] string;
    mono_paths: [..] string;
    fallback_paths: [..] string;

    defer array_free(sans_paths);
    defer array_free(mono_paths);
    defer array_free(fallback_paths);

    if sans.count array_add(*sans_paths, sans);
    if mono.count array_add(*mono_paths, mono);

    #if OS == .MACOS {
        array_add(*sans_paths, "/System/Library/Fonts/Helvetica.ttc");
        array_add(*sans_paths, "/System/Library/Fonts/SFNS.ttf");
        array_add(*mono_paths, "/System/Library/Fonts/Menlo.ttc");
        array_add(*mono_paths, "/System/Library/Fonts/SFNSMono.ttf");
        array_add(*fallback_paths, "/System/Library/Fonts/Apple Symbols.ttf");
        array_add(*fallback_paths, "/System/Library/Fonts/Supplemental/Arial Unicode.ttf");
    } else #if OS == .WINDOWS {
        array_add(*sans_paths, "C:/Windows/Fonts/segoeui.ttf");
        array_add(*sans_paths, "C:/Windows/Fonts/arial.ttf");
        array_add(*mono_paths, "C:/Windows/Fonts/consola.ttf");
        array_add(*mono_paths, "C:/Windows/Fonts/cour.ttf");
        array_add(*fallback_paths, "C:/Windows/Fonts/seguisym.ttf");
    } else {
        array_add(*sans_paths, "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        array_add(*sans_paths, "/usr/share/fonts/dejavu/DejaVuSans.ttf");
        array_add(*sans_paths, "/usr/share/fonts/TTF/DejaVuSans.ttf");
        array_add(*sans_paths, "/usr/share/fonts/liberation/LiberationSans-Regular.ttf");
        array_add(*mono_paths, "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf");
        array_add(*mono_paths, "/usr/share/fonts/dejavu/DejaVuSansMono.ttf");
        array_add(*mono_paths, "/usr/share/fonts/TTF/DejaVuSansMono.ttf");
        array_add(*mono_paths, "/usr/share/fonts/liberation/LiberationMono-Regular.ttf");
    }

    for sans_paths if load_font(text, it) break;
    if !text.fonts.count add_synthetic_font(text);

    count := text.fonts.count;

    for mono_paths if load_font(text, it) break;
    if text.fonts.count == count add_synthetic_font(text);

    for fallback_paths load_font(text, it);
}

// Release every font and the glyph cache.
//
// - text: the text system to release. It is reset to empty.
free_text_system :: (text: *Text_System) {
    for text.fonts {
        if it.face FT_Done_Face(it.face);
        array_free(it.data);
        free(it.name.data);
        free(it);
    }

    if text.library FT_Done_FreeType(text.library);

    array_free(text.fonts);
    deinit(*text.glyphs);
    deinit(*text.metrics);
    array_free(text.atlas.pixels);
    text.* = .{};
}

// Set the device scale; glyphs rasterized at another scale stay in the cache under their own size.
//
// - text: the text system to change.
// - scale: device pixels per point. Values under 0.25 are raised to 0.25.
set_scale :: (text: *Text_System, scale: float) {
    text.scale = max(scale, 0.25);
}

// Ascent, descent and line height of a font slot at `size` points.
//
// - text: the text system holding the fonts.
// - slot: which font to measure.
// - size: the font size in points.
font_metrics :: (text: *Text_System, slot: Font_Slot, size: float) -> Font_Metrics {
    index := font_index(text, slot);
    pixels := pixel_size(text, size);
    font := text.fonts[index];

    if !font || !font.face {
        scale := size;
        return .{ scale * SYNTHETIC_ASCENT, scale * SYNTHETIC_DESCENT, scale * SYNTHETIC_LINE_HEIGHT };
    }

    key := (cast(u64) index << 40) | (cast(u64) pixels << 24);
    found := table_find_pointer(*text.metrics, key);

    if found return found.*;

    FT_Set_Pixel_Sizes(font.face, 0, cast(FT_UInt) pixels);
    m := font.face.size.metrics;

    result: Font_Metrics;
    result.ascent = ceil(m.ascender / 64.0) / text.scale;
    result.descent = ceil(-m.descender / 64.0) / text.scale;
    result.line_height = ceil(m.height / 64.0) / text.scale;
    table_set(*text.metrics, key, result);

    return result;
}

// The width in points of `string` drawn with `style`, without kerning.
//
// - text: the text system holding the fonts.
// - string_to_measure: the UTF-8 text to measure.
// - style: the font and size to measure with.
text_width :: (text: *Text_System, string_to_measure: string, style: Text_Style) -> float {
    total := 0.0;
    i := 0;

    while i < string_to_measure.count {
        code, size := decode_at(string_to_measure, i);
        total += glyph_advance(text, code, style);
        i += size;
    }

    return total;
}

// Horizontal advance of one code point in points (a tab is four spaces).
//
// - text: the text system holding the fonts.
// - code: the code point.
// - style: the font and size to measure with.
glyph_advance :: (text: *Text_System, code: u32, style: Text_Style) -> float {
    if code == #char "\t" return 4 * glyph_advance(text, #char " ", style);

    font := text.fonts[font_index(text, style.font)];

    if !font || !font.face return style.size * SYNTHETIC_ADVANCE;

    glyph := get_glyph(text, code, style.font, style.size);

    return glyph.advance / text.scale;
}

// The byte offset in `string_to_measure` of the grapheme boundary nearest to horizontal position `x`.
//
// - text: the text system holding the fonts.
// - string_to_measure: the UTF-8 text.
// - style: the font and size the text is drawn with.
// - x: the position in points from the start of the text. Zero or less gives 0.
index_at_x :: (text: *Text_System, string_to_measure: string, style: Text_Style, x: float) -> int {
    if x <= 0 return 0;

    position := 0.0;
    i := 0;

    while i < string_to_measure.count {
        next := next_grapheme(string_to_measure, i);
        width := text_width(text, string.{ next - i, string_to_measure.data + i }, style);

        if x < position + width / 2 return i;

        position += width;
        i = next;
    }

    return string_to_measure.count;
}

// The glyph for `code` in a font slot at `size` points, rasterized into the atlas on first use.
// Falls back through the later fonts when the slot's font has no glyph for it.
//
// - text: the text system holding the fonts and atlas.
// - code: the code point.
// - slot: which font to prefer.
// - size: the font size in points.
get_glyph :: (text: *Text_System, code: u32, slot: Font_Slot, size: float) -> Glyph {
    index := font_index(text, slot);
    pixels := pixel_size(text, size);

    key := (cast(u64) index << 40) | (cast(u64) pixels << 24) | code;
    found := table_find_pointer(*text.glyphs, key);

    if found return found.*;

    glyph := rasterize(text, code, index, pixels);

    table_set(*text.glyphs, key, glyph);

    return glyph;
}

// Forget the cached glyphs and clear the atlas, when it ran out of room. Called at the start of a frame.
//
// - text: the text system whose atlas to check.
reset_atlas_if_full :: (text: *Text_System) {
    if !text.atlas.full return;

    table_reset(*text.glyphs);
    array_reset(*text.atlas.pixels);
    text.atlas.pen_x = 0;
    text.atlas.pen_y = 0;
    text.atlas.row_height = 0;
    text.atlas.full = false;
    text.atlas.version += 1;
}

#scope_module

font_index :: (text: *Text_System, slot: Font_Slot) -> int {
    index := cast(int) slot;
    if index < text.fonts.count return index;
    if text.fonts.count return text.fonts.count - 1;

    // No fonts loaded yet: give the system a synthetic one so every call works.
    add_synthetic_font(text);

    return 0;
}

pixel_size :: (text: *Text_System, size: float) -> int {
    return max(cast(int) round(size * text.scale), 1);
}

rasterize :: (text: *Text_System, code: u32, primary: int, pixels: int) -> Glyph {
    glyph: Glyph;

    // Try the slot's font, then each fallback, for a glyph that exists.
    chosen := primary;
    face_index: FT_UInt;

    for i: 0..text.fonts.count - 1 {
        candidate := ifx i == 0 then primary else i - 1;
        font := text.fonts[candidate];

        if !font || !font.face continue;

        face_index = FT_Get_Char_Index(font.face, code);

        if face_index != 0 {
            chosen = candidate;
            break;
        }
    }

    font := text.fonts[chosen];
    if !font || !font.face return glyph;

    FT_Set_Pixel_Sizes(font.face, 0, cast(FT_UInt) pixels);

    if FT_Load_Glyph(font.face, face_index, FT_LOAD_DEFAULT) != 0 return glyph;

    slot := font.face.glyph;

    glyph.advance = round(slot.advance.x / 64.0);

    if FT_Render_Glyph(slot, FT_Render_Mode.NORMAL) != 0 return glyph;

    bitmap := slot.bitmap;
    if bitmap.width == 0 || bitmap.rows == 0 return glyph;

    atlas := *text.atlas;
    width := cast(int) bitmap.width;
    height := cast(int) bitmap.rows;

    if atlas.pen_x + width + 1 > atlas.width {
        atlas.pen_x = 0;
        atlas.pen_y += atlas.row_height + 1;
        atlas.row_height = 0;
    }

    if atlas.pen_y + height + 1 > atlas.height {
        atlas.full = true;
        return glyph;
    }

    if atlas.pixels.count == 0 {
        array_resize(*atlas.pixels, atlas.width * atlas.height);
        memset(atlas.pixels.data, 0, atlas.pixels.count);
    }

    for row: 0..height - 1 {
        source := bitmap.buffer + row * bitmap.pitch;
        destination := atlas.pixels.data + (atlas.pen_y + row) * atlas.width + atlas.pen_x;
        memcpy(destination, source, width);
    }

    glyph.left = cast(float) slot.bitmap_left;
    glyph.top = cast(float) slot.bitmap_top;
    glyph.width = cast(float) width;
    glyph.height = cast(float) height;
    glyph.u0 = cast(float) atlas.pen_x / atlas.width;
    glyph.v0 = cast(float) atlas.pen_y / atlas.height;
    glyph.u1 = cast(float) (atlas.pen_x + width) / atlas.width;
    glyph.v1 = cast(float) (atlas.pen_y + height) / atlas.height;
    glyph.visible = true;

    atlas.pen_x += width + 1;
    atlas.row_height = max(atlas.row_height, height);
    atlas.version += 1;

    return glyph;
}

// Metrics of a font without a face, as fractions of the size.
SYNTHETIC_ASCENT :: 0.8;
SYNTHETIC_DESCENT :: 0.2;
SYNTHETIC_ADVANCE :: 0.6;
SYNTHETIC_LINE_HEIGHT :: 1.25;
