// FreeType against the pinned library (tools/build_native_libs.py): open a bitmap font held in
// memory, look up and render a glyph, and check the pixels. The font is a BDF file written out
// below (a single 8x8 letter A), so no font file has to ship with the repository, and BDF needs no
// system font. Also checks the library version and that unreadable data is refused.
#import "Basic";
#import "freetype";

// One glyph, 'A', as eight rows of eight pixels.
FONT :: #string BDF
STARTFONT 2.1
FONT -test-Tiny-Medium-R-Normal--8-80-75-75-C-80-ISO10646-1
SIZE 8 75 75
FONTBOUNDINGBOX 8 8 0 0
STARTPROPERTIES 4
FAMILY_NAME "Tiny"
FONT_ASCENT 8
FONT_DESCENT 0
DEFAULT_CHAR 65
ENDPROPERTIES
CHARS 1
STARTCHAR A
ENCODING 65
SWIDTH 1000 0
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
18
3C
66
66
7E
66
66
66
ENDCHAR
ENDFONT
BDF;

ROWS :: u8.[0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66];

main :: () {
    library: FT_Library;
    assert(FT_Init_FreeType(*library) == 0, "FT_Init_FreeType");

    major, minor, patch: FT_Int;
    FT_Library_Version(library, *major, *minor, *patch);
    assert(major == 2 && minor >= 10, "FreeType version %.%.%", major, minor, patch);

    face: FT_Face;
    error := FT_New_Memory_Face(library, FONT.data, cast(FT_Long) FONT.count, 0, *face);
    assert(error == 0, "FT_New_Memory_Face: error %", error);
    assert(face.num_glyphs >= 1, "% glyphs", face.num_glyphs);
    assert(to_string(face.family_name) == "Tiny", "family '%'", to_string(face.family_name));

    assert(FT_Set_Pixel_Sizes(face, 0, 8) == 0, "FT_Set_Pixel_Sizes");
    assert(FT_Get_Char_Index(face, #char "A") != 0, "'A' is in the font");
    assert(FT_Load_Char(face, #char "A", FT_LOAD_RENDER) == 0, "FT_Load_Char");

    slot := face.glyph;
    bitmap := slot.bitmap;
    assert(bitmap.rows == 8 && bitmap.width == 8, "bitmap %x%", bitmap.width, bitmap.rows);
    assert(bitmap.pixel_mode == 1, "pixel mode % (monochrome is 1)", bitmap.pixel_mode);
    assert(bitmap.pitch >= 1, "pitch %", bitmap.pitch);
    for row: 0..7 {
        got := bitmap.buffer[row * bitmap.pitch];
        assert(got == ROWS[row], "row %: got %, want %", row, got, ROWS[row]);
    }
    assert(slot.advance.x >> 6 == 8, "advance %", slot.advance.x >> 6);

    // A character the font lacks maps to the missing glyph, not to a real one.
    assert(FT_Get_Char_Index(face, #char "Z") == 0, "'Z' is not in the font");

    assert(FT_Done_Face(face) == 0, "FT_Done_Face");

    // Data that is no font at all is refused.
    junk := "this is not a font";
    bad: FT_Face;
    assert(FT_New_Memory_Face(library, junk.data, cast(FT_Long) junk.count, 0, *bad) != 0, "junk was accepted");

    assert(FT_Done_FreeType(library) == 0, "FT_Done_FreeType");
    print("freetype ok\n");
}
