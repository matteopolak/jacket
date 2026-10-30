# Simp


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `prepare_window :: (window: Window_Type, wanted_msaa: int) -> chosen_msaa: s32`
- `Simp_Coordinate_System :: enum u8`
- `set_render_target :: (window: Window_Type, coords := Simp_Coordinate_System.RIGHT_HANDED)`
- `set_render_target :: (texture: *Texture, coords := Simp_Coordinate_System.RIGHT_HANDED)`
- `update_window :: (window: Window_Type)`
- `draw_text :: (font: *Dynamic_Font, x: s64, y: s64, text: string, color := Vector4.{1,1,1,1}, effects: Font_Effects = 0) -> width: s64`
- `begin_prepare_text :: (font: *Dynamic_Font)`
- `append_prepare_text :: (font: *Dynamic_Font, s: string, effect_flags: Font_Effects = 0)`
- `end_prepare_text :: (font: *Dynamic_Font)`
- `draw_prepared_text :: (font: *Dynamic_Font, x: s64, y: s64, color := Vector4.{1, 1, 1, 1}, effects: Font_Effects = 0)`
- `Window_Info :: struct`
- `window_infos: [..] Window_Info`
- `set_scissor :: (p0: Vector2, p1: Vector2)`
- `get_render_dimensions :: (window: Window_Type) -> (width: s32, height: s32)`
- `Pixel_Read :: struct`
- [render_api != .OPENGL] `disable_window :: (window: Window_Type)`
- [render_api != .OPENGL] `enable_window :: (window: Window_Type)`
- `backend_init :: (info: *Window_Info)`
- `disable_window :: (window: Window_Type)`
- `enable_window :: (window: Window_Type)`
- `Shader_Platform :: struct`
- `set_parameter :: (shader: *Shader, name: *u8, m: Matrix4)`
- `clear_render_target :: (r: float, g: float, b: float, a: float)`
- `set_scissor :: (x0: s32, y0: s32, x1: s32, y1: s32)`
- `clear_scissor :: ()`
- `swap_buffers :: (window: Window_Type, vsync := true)`
- `get_shader_program :: (shader_text: string) -> GLuint`
- `draw_generated_quads :: (font: *Dynamic_Font, color := Vector4.{1, 1, 1, 1})`
- `Pixel_Read_Platform :: struct`
- `pixel_read_begin :: (source: *Texture , dest_format: Texture_Format, width: s32 = -1, height: s32 = -1, srgb := false, flip := false) -> *Pixel_Read`
- `pixel_read_is_done :: (reader: *Pixel_Read) -> bool`
- `pixel_read_end :: (reader: *Pixel_Read, bitmap: *Bitmap)`
- `pixel_read_end :: (reader: *Pixel_Read) -> Bitmap`
- `Backend_Platform_Specific :: struct`
- `Texture_Platform_Specific :: struct`
- `Window_Info_Platform_Specific :: struct`
- `backend_realloc_texture :: (texture: *Texture, bitmap: *Bitmap)`
- `backend_set_render_target :: (info: *Window_Info)`
- `backend_create_render_target :: (texture: *Texture)`
- `backend_set_render_target :: (texture: *Texture)`
- `backend_update_texture :: (texture: *Texture, bitmap: *Bitmap)`
- `backend_use_shader :: (s: *Shader)`
- `backend_update_shader_parameter_locations :: (s: *Shader)`
- `backend_draw_immediate_vertices :: (num_vertices: int)`
- `backend_set_texture :: (shader: *Shader, texture: *Texture)`
- `backend_resize_render_target :: (info: Window_Info)`
- `backend_flush :: ()`
- `is_supported :: (tf: Texture_Format) -> bool`
- `backend_tweak_projection_matrix :: (m: *Matrix4)`
- `Bitmap :: struct`
- `bitmap_alloc :: (using bitmap: *Bitmap, w: s32, h: s32, _format: Texture_Format)`
- `deinit :: (bitmap: *Bitmap)`
- `bitmap_load :: (bitmap: *Bitmap, filepath: string) -> bool`
- `bitmap_load_from_memory :: (bitmap: *Bitmap, memory: [] u8) -> bool`
- `bitmap_save :: (bitmap: *Bitmap, filepath: string, save_alpha := false) -> bool`
- `bitmap_build_mipmaps :: (bitmap: *Bitmap)`
- `Texture_Flags :: enum_flags u16`
- `Texture :: struct`
- `texture_load_from_file :: (map: *Texture, filename: string, srgb:=false, build_mipmaps:=false) -> bool`
- `texture_load_from_memory :: (texture: *Texture, memory: [] u8, srgb:=false, build_mipmaps:=false) -> bool`
- `texture_load_from_bitmap :: (using texture: *Texture, bitmap: *Bitmap) -> bool`
- `texture_create_render_target :: (width: s32, height: s32, format: Texture_Format, flags: Texture_Flags = 0) -> Texture`
- `texture_destroy :: (texture: *Texture)`
- `Texture_Format :: enum u16`
- `get_image_size :: (tf: Texture_Format, w: int, h: int) -> int, stride:int`
- `get_image_bytes_per_texel :: (tf: Texture_Format) -> int`
- `get_image_bytes_per_block :: (tf: Texture_Format) -> int`
- `Shader :: struct`
- `set_projection :: ()`
- `set_shader_for_color :: (enable_blend := false)`
- `set_shader_for_text :: ()`
- `set_shader_for_images :: (texture: *Texture)`
- `shader_text: Shader`
- `shader_color: Shader`
- `shader_color_with_blend: Shader`
- `shader_sprite_right_handed: Shader`
- `Immediate_State :: struct`
- `Immediate_Vertex :: struct`
- `immediate_reset :: ()`
- `immediate_set_texture :: (texture: *Texture)`
- `immediate_set_shader :: (s: *Shader)`
- `immediate_begin :: ()`
- `immediate_flush :: ()`
- `immediate_vertex :: (position: Vector3, color_scale: Vector4, uv: Vector2 = .{}, normal: Vector3 = .{})`
- `immediate_quad :: (x0: float, y0: float, x1: float, y1: float, color: Vector4)`
- `immediate_quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{1,1,1,1}, uv0 := Vector2.{0,0}, uv1 := Vector2.{1,0}, uv2 := Vector2.{1,1}, uv3 := Vector2.{0, 1})`
- `immediate_quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, c0: Vector4, c1: Vector4, c2: Vector4, c3: Vector4, uv0 := Vector2.{0,0}, uv1 := Vector2.{1,0}, uv2 := Vector2.{1,1}, uv3 := Vector2.{0, 1})`
- `immediate_quad :: (p0: Vector3, p1: Vector3, p2: Vector3, p3: Vector3, color: Vector4, uv0 := Vector2.{0,0}, uv1 := Vector2.{1,0}, uv2 := Vector2.{1,1}, uv3 := Vector2.{0, 1}, normal := Vector3.{z=1})`
- `immediate_quad :: (p0: Vector3, p1: Vector3, p2: Vector3, p3: Vector3, color: Vector4, uv0: Vector2, uv1: Vector2, uv2: Vector2, uv3: Vector2, st0: Vector2, st1: Vector2, st2: Vector2, st3: Vector2, normal: Vector3, tangent: Vector4)`
- `immediate_triangle :: (p0: Vector3, p1: Vector3, p2: Vector3, c1: Vector4, c2: Vector4, c3: Vector4, uv0 := Vector2.{}, uv1 := Vector2.{}, uv2 := Vector2.{}, normal := Vector3.{z=1})`
- `immediate_set_2d_projection :: (render_target_width: s32, render_target_height: s32)`
- `glyph_id_to_hash_key :: (x: u32) -> int`
- `isdigit :: (utf32: u32) -> bool`
- `isLegalUTF8 :: (source: *u8, length: s64) -> bool`
- `Font_Effects :: enum_flags u32`
- `Glyph_Data :: struct`
- `Font_Quad :: struct`
- `Dynamic_Font :: struct`
- `Font_Page :: struct`
- `Font_Line :: struct`
- `Loaded_Font :: struct`
- `Font_Range_Record :: struct`
- `Font_Coverage_Table :: struct`
- `Font_Lookup_Subtable :: struct`
- `Font_Sequence_Table_Entry :: struct`
- `Font_Lookup_Table :: struct`
- `Font_Feature :: struct`
- `init_fonts :: (_page_size_x: s32 = -1, _page_size_y: s32 = -1)`
- `get_glyph_width_in_pixels :: (glyph: *Glyph_Data) -> s64`
- `destroy :: (font: *Loaded_Font)`
- `is_latin :: (utf32: u32) -> bool`
- `find_loaded_font :: (filename: string) -> *Loaded_Font`
- `make_loaded_font :: (name: string, data: [] u8) -> *Loaded_Font`
- `make_loaded_font :: (filename: string) -> (font: *Loaded_Font)`
- `ensure_fonts_are_initted :: ()`
- `get_character_width_in_pixels :: (using font: *Dynamic_Font, utf8: *u8, effect_flags: Font_Effects) -> u32`
- `accumulate_glyphs_from_font :: (source: *Dynamic_Font, dest: *Dynamic_Font)`
- `convert_to_temporary_glyphs :: (using font: *Dynamic_Font, s: string, effect_flags: Font_Effects)`
- `set_unknown_character :: (using font: *Dynamic_Font, utf32: u32) -> bool`
- `load_font :: (path: string, basename: string, pixel_height: int) -> font: *Dynamic_Font`
- `load_font_part_2 :: (result: *Dynamic_Font, loaded_font: Loaded_Font, pixel_height: int)`
- `get_font_at_size :: (path: string, name: string, pixel_height: int) -> *Dynamic_Font`
- `get_font_at_size :: (memory: [] u8, pixel_height: int) -> *Dynamic_Font`
- `release_font :: (font: *Dynamic_Font)`
- `get_baseline_height :: (font: *Dynamic_Font) -> s64`
- `prepare_text :: (font: *Dynamic_Font, text: string, effect_flags: Font_Effects = 0) -> (width: s64)`
- `generate_quads_for_prepared_text :: (font: *Dynamic_Font, x: s64, y: s64, effect_flags: Font_Effects)`


- `shader_sprite_left_handed: Shader`

## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Backend_Platform_Specific`

```jai
Backend_Platform_Specific :: struct {}
```
### `Bitmap`

```jai
Bitmap :: struct {
    width:  s32;
    height: s32;

    mipmap_count: s32 = 1;

    data:   [] u8;
    stride: s32;

    flags: enum_flags u32 {
        FREE_DATA_ON_DEINIT :: 0x1;
        IS_SRGB             :: 0x2;
    }

    format := Texture_Format.BGRA8;
}
```
### `Dynamic_Font`

```jai
Dynamic_Font :: struct {
    name: string;
    latin_font: *Dynamic_Font;

    glyph_lookup: Table(int, *Glyph_Data);

    private1: FT_Face;

    character_range_low:  int;
    character_range_high: int;

    character_height:       int;
    default_line_spacing:   int;
    max_ascender:           int;
    max_descender:          int;
    typical_ascender:       int;
    typical_descender:      int;
    em_width:               int;

    y_offset_for_my_latin_characters: float;
    y_offset_for_centering: int;

    use_other_font_for_latin_always: bool;
    reversed: bool;
    glyph_conversion_failed: bool;
    glyph_index_for_unknown_character: u32;
    utf32_for_unknown_character := 0xfffd;

    features: [..] Font_Feature;

    feature_smallcaps:      *Font_Feature;
    feature_ligatures:      *Font_Feature;
    feature_lining_numbers: *Font_Feature;

    temporary_glyphs: [..] *Glyph_Data;
    temporary_glyphs_byte_offsets: [..] int;
    temporary_glyphs_width_in_pixels: s64;

    current_quads: [..] Font_Quad;
}
```
### `Font_Coverage_Table`

```jai
Font_Coverage_Table :: struct {
    format: s64;

    glyphs: [] u16;
    range_records: [] Font_Range_Record;
}
```
### `Font_Effects`

```jai
Font_Effects :: enum_flags u32 {
    SMALLCAPS      :: 0x1;
    LINING_FIGURES :: 0x2;
    NO_KERNING     :: 0x4;
    LEFT_JUSTIFIED :: 0x8;
}
```
### `Font_Feature`

```jai
Font_Feature :: struct {
    feature_tag: u32;
    feature_tables_pointer: *u8;
    expanded: bool;

    lookup_lists:  [] u16;
    lookup_tables: [] Font_Lookup_Table;
}
```
### `Font_Line`

```jai
Font_Line :: struct {
    page: *Font_Page;

    bitmap_cursor_x: s16;
    bitmap_cursor_y: s16;

    height: int;
}
```
### `Font_Lookup_Subtable`

```jai
Font_Lookup_Subtable :: struct {
    format: s64;
    coverage_table: *Font_Coverage_Table;

    delta_glyph_id: s64;
    glyphs: [] u16;
    sequence_table: [] Font_Sequence_Table_Entry;
}
```
### `Font_Lookup_Table`

```jai
Font_Lookup_Table :: struct {
    lookup_type:   s32;
    lookup_flags:  u32;

    subtables: [] Font_Lookup_Subtable;
}
```
### `Font_Page`

```jai
Font_Page :: struct {
    texture:       Texture;
    bitmap_data:   *Bitmap;

    line_cursor_y: s16;
    lines:         [..] *Font_Line;

    dirty := false;
}
```
### `Font_Quad`

```jai
Font_Quad :: struct {
    ul, lr:   Vector2;
    uv0, uv1: Vector2;
    color := Vector4.{ 1, 1, 1, 1 };

    p0, p1, p2, p3: Vector2;
    u0, v0, u1, v1: float;

    glyph: *Glyph_Data;
}
```

`end_prepare_text` leaves one quad per prepared glyph in `font.current_quads`, with the pen at the origin and the baseline at `y = 0`; `begin_prepare_text` clears them. `ul` is the corner with the smaller `y`, `lr` the opposite one, `uv0` and `uv1` the atlas coordinates drawn at them, and `p0` to `p3`, `u0` to `v1` the same box in the older forms. `color` is `context.simp.text_color` (white by default) at the time the quads are made, and `draw_prepared_text` multiplies it into the colour it is given. `draw_prepared_text` lays the run out again at the position it is passed.

```jai
Simp_Context :: struct {
    text_color := Vector4.{ 1, 1, 1, 1 };
}

#add_context simp: Simp_Context;
```
### `Font_Range_Record`

```jai
Font_Range_Record :: struct {
    start, end: s32;
    start_coverage_index: s32;
}
```
### `Font_Sequence_Table_Entry`

```jai
Font_Sequence_Table_Entry :: struct {
    glyphs: [] u16;
}
```
### `Glyph_Data`

```jai
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
```
### `Immediate_State`

```jai
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
```
### `Immediate_Vertex`

```jai
Immediate_Vertex :: struct {
    position:    Vector3;
    color_scale: Vector4;
    uv0:         Vector2;
    uv1:         Vector2;
    normal:      Vector3;
    tangent:     Vector4;
}
```
### `Loaded_Font`

```jai
Loaded_Font :: struct {
    name: string;
    face: FT_Face;
    data: [] u8;
}
```
### `Pixel_Read`

```jai
Pixel_Read :: struct {
    dest_format: Texture_Format;
    dest_srgb:   bool;
    dest_width, dest_height: s32;

    flip := false;

    using platform_specific: Pixel_Read_Platform;
}
```
### `Pixel_Read_Platform`

Variant 1 (common):

```jai
Pixel_Read_Platform :: struct {
    dest_size: s64;

    intermediate_texture: Texture;
    pbo: GLuint;
    src\_fbo_handle: GLuint;
    dest_fbo_handle: GLuint;
}
```
Variant 2 (common):

```jai
Pixel_Read_Platform :: struct {}
```
### `Shader`

```jai
Shader  :: struct {
    using platform: Shader_Platform;
}
```
### `Shader_Platform`

Variant 1 (common):

```jai
Shader_Platform :: struct {
    gl_handle: GLuint;

    position_loc:      GLint = -1;
    color_scale_loc:   GLint = -1;
    normal_loc:        GLint = -1;
    tangent_loc:       GLint = -1;
    uv_0_loc:          GLint = -1;
    uv_1_loc:          GLint = -1;

    diffuse_texture_loc: GLint = -1;

    alpha_blend := false;
}
```
Variant 2 (common):

```jai
Shader_Platform :: struct {}
```
### `Simp_Coordinate_System`

```jai
Simp_Coordinate_System :: enum u8 {
    LEFT_HANDED  :: 0;
    RIGHT_HANDED :: 1;
}
```
### `Texture`

```jai
Texture :: struct {
    width, height, depth: s32 = 1;
    mipmap_count: u16 = 1;
    array_length: u16 = 1;

    format: Texture_Format;
    flags:  Texture_Flags;

    using specific: Texture_Platform_Specific;
}
```
### `Texture_Flags`

```jai
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
```
### `Texture_Format`

```jai
Texture_Format :: enum u16 {
    Unknown :: 0;

    R8       :: 1;
    RG8      :: 2;
    RGB8     :: 3;
    RGBA8    :: 4;
    RGBX8;
    BGRA8;
    B5G6R5;
    RGB10A2;
    R16;

    R16F :: 0x10;
    RG16F;
    RGBA16F;
    R32F;
    RG32F;
    RGBA32F;
    RG11B10F;
    RGB9E5F;

    R16I;
    RG16I;
    RGBA16I;
    R32I;
    RG32I;
    RGBA32I;

    BC1 :: 0x100;
    BC3;
    BC4;
    BC5;
    BC6;
    BC7;

    ETC_RGB;
    ETC_RGBA;
    ETC_R;
    ETC_RG;
    ETC_RGB_A1;

    D16;
    D24S8;
    D32F;
    D32FS8;
}
```
### `Texture_Platform_Specific`

```jai
Texture_Platform_Specific :: struct {}
```
This is the `render_api == .NONE` shape. Under the default `render_api ==
.OPENGL`, the exported field is `gl_handle: GLuint`; `GLuint` is the
platform binding's 32-bit unsigned GL object name. Other render APIs must use
the shape from their declared backend: where no backend shape is supplied, do
not infer a field from the `.NONE` or OpenGL variants.
### `Window_Info`

```jai
Window_Info :: struct {
    window: Window_Type;
    window_width, window_height: s32;
    wanted_msaa: s32;
    msaa:        s32;

    backend_initted_for_this_window := false;
    specific: Window_Info_Platform_Specific;
}
```
### `Window_Info_Platform_Specific`

```jai
Window_Info_Platform_Specific :: struct {}
```
## Contracts and constraints

The texture-format helpers distinguish texel formats from block-compressed formats. `get_image_size` returns total byte size and row stride: texel formats multiply width by bytes per texel, while block formats round dimensions up to 4×4 blocks and multiply by bytes per block. The bytes-per-texel helper returns zero for block or unsupported values, which is the dispatch signal used by `get_image_size`; invalid formats can assert. Integer overflow and negative dimensions are not guarded by this helper.

### Shader source encoding

On the OpenGL backend, `get_shader_program(shader_text)` treats its one string as shared GLSL source for both stages. It compiles that string once with a prepended vertex-stage preamble defining `VERTEX_SHADER` and `OUT_IN` as `out`, and once with a fragment-stage preamble defining `FRAGMENT_SHADER` and `OUT_IN` as `in`; shader code selects stage-specific declarations with those macros. Desktop preambles specify GLSL 330 core. GLES preambles specify GLSL 300 es and `mediump` float precision. The function links both compiled stages into one program and returns its GL name; a compile or link failure logs the driver diagnostic and returns zero. There is no delimiter or embedded stage-label syntax in the input string.

### Bitmap, texture, and mipmap behavior

`Bitmap` starts with one mip level and defaults to `BGRA8`; `bitmap_alloc` sets the requested dimensions and format, allocates owned pixel storage, and sets a tightly packed base-level stride. `bitmap_load_from_memory` rejects input larger than the decoder's signed 32-bit length, then decodes to the source channel count (or four channels for NVN); decoded storage is copied into Bitmap-owned memory. `bitmap_save` accepts only `RGB8` and `RGBA8`, and recognizes PNG, TGA, BMP, and JPG filename extensions; JPG uses quality 80. `FREE_DATA_ON_DEINIT` controls whether `deinit` frees `data.data`.

`bitmap_build_mipmaps` supports `R8`, `RG8`, `RGB8`, and `RGBA8`; other formats assert. It repeatedly halves each dimension, rounding down but never below one, until reaching 1×1. Each generated level uses a row stride rounded up to a four-byte boundary in the backing mip-chain storage. RGBA resampling treats channel 3 as alpha. With `IS_SRGB`, generated levels use sRGB-aware resampling; otherwise they use ordinary 8-bit resampling. The operation replaces prior owned pixel storage, sets `mipmap_count` to the number of levels, and marks the resulting data owned. Texture load helpers default `srgb` and `build_mipmaps` to false. Loading a bitmap copies its dimensions, format, mip count, and sRGB flag into texture state and updates backend storage; a render-target texture always receives the `Render_Target` flag.

### Closed windows

With the OpenGL backend, a window's renderer context is released when the window closes. Presenting such a window with `swap_buffers` does nothing, so a frame already in progress when the close event arrives can finish without error. The window is no longer a render target: call `set_render_target` only for windows that are still open.

### Font effects and metrics

`Font_Effects.SMALLCAPS` and `LINING_FIGURES` request substitutions only when the loaded font exposes the corresponding OpenType features. When both are enabled, ASCII digits skip small-cap substitution. `NO_KERNING` and `LEFT_JUSTIFIED` are declared flags but are not applied by the current text preparation or quad-generation path. `get_glyph_width_in_pixels`, `get_character_width_in_pixels`, and `prepare_text` report glyph advance widths, not bitmap bounding-box widths; drawing uses glyph bitmap width/height for quads and advances the pen by each glyph's advance. The current path does not apply pair kerning or general shaping/ligature processing.

Font glyph pages are shared atlas bitmaps, defaulting to 2048×1024 RGBA8. `init_fonts(page_size_x := -1, page_size_y := -1)` leaves those defaults in place unless either argument is nonnegative; if either is supplied, both must be at least 64. Glyphs are packed along rows, reusing a row only when its height accommodates the glyph and is not more than roughly 10/7 of the glyph height; otherwise a row with about 10% vertical slack is added, or another page is created when the current page cannot fit it. Glyph coverage is stored in alpha with white RGB. `is_latin(utf32)` returns true for code points through U+024F and for U+2000–U+218F; this is a font-routing heuristic, not a Unicode Script property. The default missing-character replacement is U+FFFD.

If a font lacks U+FFFD, loading tries U+2022 and then `?` as the missing-glyph fallback; if none exists, it logs the condition. A missing requested glyph uses that fallback index and sets `glyph_conversion_failed` during text conversion.

`get_font_at_size(path, name, pixel_height)` reuses a dynamic font by the pair `(name, pixel_height)`; the path is not part of that dynamic-font cache key. File-backed `Loaded_Font` faces are cached by their joined filename. The memory overload reuses a loaded face only when both input pointer and byte count match; the supplied byte storage is retained as the face backing data rather than copied, so keep it valid while the face may be used. `release_font` currently does nothing, and the public `destroy(Loaded_Font)` only frees its copied name string; loaded faces and atlas storage are retained by the module's registries/pools. Shared font-face use means changing the face size for a glyph operation also changes it for other dynamic fonts sharing that face; glyph operations set the requested pixel size before loading.

## Module parameters

```jai
#module_parameters (render_api := Render_API.OPENGL)() {
    Render_API :: enum u32 {
        NONE;
        SOFTWARE;
        OPENGL;
        METAL;
        NVN;
    }
}
```
Platform coverage note: Nintendo branches exist, but their declarations are not specified here; exports for those targets are unspecified.
