F :: #import "freetype";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

probe :: #library "../../.work/native/abi_probe";
check_size :: (which: s32) -> u64 #foreign probe "binding_size";
check_offset :: (which: s32) -> u64 #foreign probe "binding_offset";

main :: () {
    check(type_info(F.FT_FaceRec).name == "FT_FaceRec_");
    check(size_of(F.FT_FaceRec) == check_size(10));
    check(size_of(F.FT_GlyphSlotRec) == check_size(11));
    check(size_of(F.FT_Bitmap) == check_size(12));
    check(size_of(F.FT_Vector) == check_size(13));
    record: F.FT_FaceRec;
    glyph: F.FT_GlyphSlotRec;

    check(cast(u64) *record.glyph - cast(u64) *record == check_offset(5));
    check(cast(u64) *glyph.bitmap - cast(u64) *glyph == check_offset(6));
    lib: F.FT_Library;

    check(F.FT_Init_FreeType(*lib) == 0 && lib != null);
    defer check(F.FT_Done_FreeType(lib) == 0);
    face: F.FT_Face;

    check(F.FT_New_Face(lib, "/System/Library/Fonts/Helvetica.ttc", 0, *face) == 0);
    defer check(F.FT_Done_Face(face) == 0);
    check(F.FT_IS_SCALABLE(face));
    check(F.FT_HAS_HORIZONTAL(face));
    check(F.FT_Set_Pixel_Sizes(face, 0, 24) == 0);
    check(F.FT_Load_Char(face, 65, F.FT_LOAD_RENDER) == 0);
    check(face.glyph != null && face.glyph.bitmap.width > 0 && face.glyph.bitmap.rows > 0);
    check(face.glyph.bitmap.buffer != null);
    invalid: F.FT_Face;

    check(F.FT_New_Face(lib, "/no/such/font", 0, *invalid) != 0);
}
