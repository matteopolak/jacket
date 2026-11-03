// Where the origin of a render target is. `RIGHT_HANDED` puts it at the bottom left with y up;
// `LEFT_HANDED` at the top left with y down.
Simp_Coordinate_System :: enum u8 {
    LEFT_HANDED :: 0;
    RIGHT_HANDED :: 1;
}

// Extra properties of a texture, such as `sRGB` and `Render_Target`.
Texture_Flags :: enum_flags u16 {
    sRGB;
    Cube;
    Array;
    Volume;
    Buffer;
    Render_Target;
    Multisampled;
    No_Skip_Mipmaps;
    Display_Target;
}

// How a texture is sampled when drawn larger or smaller than its size: `LINEAR` blends, `NEAREST` picks one texel.
//
// ```jai,no_run
// texture: Texture;
// texture_load_from_file(*texture, "sprite.png");
// texture_set_filter(*texture, .NEAREST); // crisp pixels
// ```
Texture_Filter :: enum u8 {
    LINEAR;
    NEAREST;
}

// An image on the graphics card. Load one with `texture_load_from_file` or `texture_load_from_memory`,
// draw it after `set_shader_for_images`, and release it with `texture_destroy`.
Texture :: struct {
    width, height, depth: s32 = 1;
    mipmap_count: u16 = 1;
    array_length: u16 = 1;

    format: Texture_Format;
    flags: Texture_Flags;

    using specific: Texture_Platform_Specific;
}

Window_Info_Platform_Specific :: struct {}

// What Simp knows about a window it draws into: its size in native window units, the desktop
// backing/DPI scale and the sample count. Cocoa uses points; Win32 and X11 use device pixels.
Window_Info :: struct {
    window: Window_Type;
    window_width, window_height: s32; // in window coordinate units (what the application draws in)
    pixel_scale: float = 1; // Cocoa pixels per point, or desktop DPI / 96 on Win32 and X11
    wanted_msaa: s32;
    msaa: s32;

    backend_initted_for_this_window := false;
    specific: Window_Info_Platform_Specific;
}

// A shader program. `set_shader_for_color` and `set_shader_for_images` select the built-in ones.
Shader :: struct {
    using platform: Shader_Platform;
}

Backend_Platform_Specific :: struct {}

// One corner in the batch that `immediate_vertex` fills.
Immediate_Vertex :: struct {
    position: Vector3;
    color_scale: Vector4;
    uv0: Vector2;
    uv1: Vector2;
    normal: Vector3;
    tangent: Vector4;
}

// The batch of corners waiting to be drawn, with the current window, shader and coordinate system.
Immediate_State :: struct {
    current_window_info: Window_Info;

    current_shader: *Shader;
    texture_render_target: *Texture;
    num_vertices: s32;

    MAX_VERTICES :: 2400;
    vertices: [MAX_VERTICES] Immediate_Vertex;

    specific: Backend_Platform_Specific;

    coordinate_system := Simp_Coordinate_System.RIGHT_HANDED;

}

// Flags that change how text is laid out: small caps, lining figures, no kerning, left justification.
//
// ```jai
// effects: Font_Effects = .SMALLCAPS | .NO_KERNING;
// assert((effects & .NO_KERNING) != 0);
// ```
Font_Effects :: enum_flags u32 {
    SMALLCAPS :: 0x1;
    LINING_FIGURES :: 0x2;
    NO_KERNING :: 0x4;
    LEFT_JUSTIFIED :: 0x8;
}

// One glyph of a font: its code point, where it sits in the glyph atlas, and its size and advance.
Glyph_Data :: struct {
    utf32: u32;
    glyph_index_within_font: u32;

    x0, y0: s16;
    width, height: u32;

    offset_x, offset_y: s16;

    ascent: s16;
    advance: s16;

    page: *Font_Page;
}

// Per-context Simp settings, read as `context.simp`. `text_color` is multiplied into the colour of text
// being prepared; white leaves it alone.
Simp_Context :: struct {
    // The colour recorded in each `Font_Quad` as text is prepared. `draw_prepared_text` multiplies it into the colour it is
    // given, so white (the default) leaves that colour alone.
    text_color := Vector4.{ 1, 1, 1, 1 };
}

#add_context simp: Simp_Context;

// One glyph of prepared text: where it goes, the part of the glyph atlas it shows and its tint.
// `end_prepare_text` fills `Dynamic_Font.current_quads` with one per glyph, with the pen at the origin.
Font_Quad :: struct {
    ul, lr: Vector2;
    uv0, uv1: Vector2;
    color := Vector4.{ 1, 1, 1, 1 };

    p0, p1, p2, p3: Vector2;
    u0, v0, u1, v1: float;

    glyph: *Glyph_Data;
}

Font_Line :: struct {
    page: *Font_Page;

    bitmap_cursor_x: s16;
    bitmap_cursor_y: s16;

    height: int;
}

Font_Page :: struct {
    texture: Texture;
    bitmap_data: *Bitmap;

    line_cursor_y: s16;
    lines: [..] *Font_Line;

    dirty := false;
}

Font_Range_Record :: struct {
    start, end: s32;
    start_coverage_index: s32;
}

Font_Coverage_Table :: struct {
    format: s64;

    glyphs: [] u16;
    range_records: [] Font_Range_Record;
}

Font_Sequence_Table_Entry :: struct {
    glyphs: [] u16;
}

Font_Lookup_Subtable :: struct {
    format: s64;
    coverage_table: *Font_Coverage_Table;

    delta_glyph_id: s64;
    glyphs: [] u16;
    sequence_table: [] Font_Sequence_Table_Entry;
}

Font_Lookup_Table :: struct {
    lookup_type: s32;
    lookup_flags: u32;

    subtables: [] Font_Lookup_Subtable;
}

Font_Feature :: struct {
    feature_tag: u32;
    feature_tables_pointer: *u8;
    expanded: bool;

    lookup_lists: [] u16;
    lookup_tables: [] Font_Lookup_Table;
}

// A font at one size, ready to draw with. `load_font` returns one; it makes glyphs as text needs them
// and keeps them in an atlas. `character_height` is in device pixels, `pixel_scale` the number of those
// per drawing unit.
Dynamic_Font :: struct {
    name: string;
    latin_font: *Dynamic_Font;

    glyph_lookup: Table(int, *Glyph_Data);

    private1: FT_Face;

    character_range_low: int;
    character_range_high: int;

    character_height: int; // in device pixels: the requested size times `pixel_scale`
    pixel_scale: float = 1; // device pixels per drawing unit; glyphs are rasterized at this scale and drawn at 1/scale
    default_line_spacing: int;
    max_ascender: int;
    max_descender: int;
    typical_ascender: int;
    typical_descender: int;
    em_width: int;

    y_offset_for_my_latin_characters: float;
    y_offset_for_centering: int;

    use_other_font_for_latin_always: bool;
    reversed: bool;
    glyph_conversion_failed: bool;
    glyph_index_for_unknown_character: u32;
    utf32_for_unknown_character := 0xfffd;

    features: [..] Font_Feature;

    feature_smallcaps: *Font_Feature;
    feature_ligatures: *Font_Feature;
    feature_lining_numbers: *Font_Feature;

    temporary_glyphs: [..] *Glyph_Data;
    temporary_glyphs_byte_offsets: [..] int;
    temporary_glyphs_width_in_pixels: s64;

    current_quads: [..] Font_Quad;
}

// A pending read of a render target's pixels back to the CPU. See `pixel_read_begin`.
Pixel_Read :: struct {
    dest_format: Texture_Format;
    dest_srgb: bool;
    dest_width, dest_height: s32;

    flip := false;

    using platform_specific: Pixel_Read_Platform;
}
