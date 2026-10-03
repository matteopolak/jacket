# freetype

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `FT_HAS_HORIZONTAL :: inline (face: FT_Face) -> bool`
- `FT_HAS_VERTICAL :: inline (face: FT_Face) -> bool`
- `FT_HAS_KERNING :: inline (face: FT_Face) -> bool`
- `FT_IS_SCALABLE :: inline (face: FT_Face) -> bool`
- `FT_IS_SFNT :: inline (face: FT_Face) -> bool`
- `FT_IS_FIXED_WIDTH :: inline (face: FT_Face) -> bool`
- `FT_HAS_FIXED_SIZES :: inline (face: FT_Face) -> bool`
- `FT_HAS_GLYPH_NAMES :: inline (face: FT_Face) -> bool`
- `FT_HAS_MULTIPLE_MASTERS :: inline (face: FT_Face) -> bool`
- `FT_IS_NAMED_INSTANCE :: inline (face: FT_Face) -> bool`
- `FT_IS_VARIATION :: inline (face: FT_Face) -> bool`
- `FT_IS_CID_KEYED :: inline (face: FT_Face) -> bool`
- `FT_IS_TRICKY :: inline (face: FT_Face) -> bool`
- `FT_HAS_COLOR :: inline (face: FT_Face) -> bool`
- `FT_HAS_SVG :: inline (face: FT_Face) -> bool`
- `FT_HAS_SBIX :: inline (face: FT_Face) -> bool`
- `FT_HAS_SBIX_OVERLAY :: inline (face: FT_Face) -> bool`
- `TTAG_avar : u32 : 0x61766172`
- `TTAG_BASE : u32 : 0x42415345`
- `TTAG_bdat : u32 : 0x62646174`
- `TTAG_BDF : u32 : 0x42444620`
- `TTAG_bhed : u32 : 0x62686564`
- `TTAG_bloc : u32 : 0x626C6F63`
- `TTAG_bsln : u32 : 0x62736C6E`
- `TTAG_CBDT : u32 : 0x43424454`
- `TTAG_CBLC : u32 : 0x43424C43`
- `TTAG_CFF : u32 : 0x43464620`
- `TTAG_CFF2 : u32 : 0x43464632`
- `TTAG_CID : u32 : 0x43494420`
- `TTAG_cmap : u32 : 0x636D6170`
- `TTAG_COLR : u32 : 0x434F4C52`
- `TTAG_CPAL : u32 : 0x4350414C`
- `TTAG_cvar : u32 : 0x63766172`
- `TTAG_cvt : u32 : 0x63767420`
- `TTAG_DSIG : u32 : 0x44534947`
- `TTAG_EBDT : u32 : 0x45424454`
- `TTAG_EBLC : u32 : 0x45424C43`
- `TTAG_EBSC : u32 : 0x45425343`
- `TTAG_feat : u32 : 0x66656174`
- `TTAG_FOND : u32 : 0x464F4E44`
- `TTAG_fpgm : u32 : 0x6670676D`
- `TTAG_fvar : u32 : 0x66766172`
- `TTAG_gasp : u32 : 0x67617370`
- `TTAG_GDEF : u32 : 0x47444546`
- `TTAG_glyf : u32 : 0x676C7966`
- `TTAG_GPOS : u32 : 0x47504F53`
- `TTAG_GSUB : u32 : 0x47535542`
- `TTAG_gvar : u32 : 0x67766172`
- `TTAG_HVAR : u32 : 0x48564152`
- `TTAG_hdmx : u32 : 0x68646D78`
- `TTAG_head : u32 : 0x68656164`
- `TTAG_hhea : u32 : 0x68686561`
- `TTAG_hmtx : u32 : 0x686D7478`
- `TTAG_JSTF : u32 : 0x4A535446`
- `TTAG_just : u32 : 0x6A757374`
- `TTAG_kern : u32 : 0x6B65726E`
- `TTAG_lcar : u32 : 0x6C636172`
- `TTAG_loca : u32 : 0x6C6F6361`
- `TTAG_LTSH : u32 : 0x4C545348`
- `TTAG_LWFN : u32 : 0x4C57464E`
- `TTAG_MATH : u32 : 0x4D415448`
- `TTAG_maxp : u32 : 0x6D617870`
- `TTAG_META : u32 : 0x4D455441`
- `TTAG_MMFX : u32 : 0x4D4D4658`
- `TTAG_MMSD : u32 : 0x4D4D5344`
- `TTAG_mort : u32 : 0x6D6F7274`
- `TTAG_morx : u32 : 0x6D6F7278`
- `TTAG_MVAR : u32 : 0x4D564152`
- `TTAG_name : u32 : 0x6E616D65`
- `TTAG_opbd : u32 : 0x6F706264`
- `TTAG_OS2 : u32 : 0x4F532F32`
- `TTAG_OTTO : u32 : 0x4F54544F`
- `TTAG_PCLT : u32 : 0x50434C54`
- `TTAG_POST : u32 : 0x504F5354`
- `TTAG_post : u32 : 0x706F7374`
- `TTAG_prep : u32 : 0x70726570`
- `TTAG_prop : u32 : 0x70726F70`
- `TTAG_sbix : u32 : 0x73626978`
- `TTAG_sfnt : u32 : 0x73666E74`
- `TTAG_SING : u32 : 0x53494E47`
- `TTAG_SVG : u32 : 0x53564720`
- `TTAG_trak : u32 : 0x7472616B`
- `TTAG_true : u32 : 0x74727565`
- `TTAG_ttc : u32 : 0x74746320`
- `TTAG_ttcf : u32 : 0x74746366`
- `TTAG_TYP1 : u32 : 0x54595031`
- `TTAG_typ1 : u32 : 0x74797031`
- `TTAG_VDMX : u32 : 0x56444D58`
- `TTAG_vhea : u32 : 0x76686561`
- `TTAG_vmtx : u32 : 0x766D7478`
- `TTAG_VVAR : u32 : 0x56564152`
- `TTAG_wOFF : u32 : 0x774F4646`
- `TTAG_wOF2 : u32 : 0x774F4632`
- `TTAG_0xA5kbd :: 0xA56B6264`
- `TTAG_0xA5lst :: 0xA56C7374`

### Unix-specific declarations

- `FT_RENDER_POOL_SIZE :: 16384`
- `FT_MAX_MODULES :: 32`
- `TT_CONFIG_OPTION_SUBPIXEL_HINTING :: 2`
- `TT_CONFIG_OPTION_MAX_RUNNABLE_OPCODES :: 1000000`
- `T1_MAX_DICT_DEPTH :: 5`
- `T1_MAX_SUBRS_CALLS :: 16`
- `T1_MAX_CHARSTRINGS_OPERANDS :: 256`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_X1 :: 500`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_Y1 :: 400`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_X2 :: 1000`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_Y2 :: 275`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_X3 :: 1667`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_Y3 :: 275`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_X4 :: 2333`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_Y4 :: 0`
- `FT_OUTLINE_NONE :: 0x0`
- `FT_OUTLINE_OWNER :: 0x1`
- `FT_OUTLINE_EVEN_ODD_FILL :: 0x2`
- `FT_OUTLINE_REVERSE_FILL :: 0x4`
- `FT_OUTLINE_IGNORE_DROPOUTS :: 0x8`
- `FT_OUTLINE_SMART_DROPOUTS :: 0x10`
- `FT_OUTLINE_INCLUDE_STUBS :: 0x20`
- `FT_OUTLINE_OVERLAP :: 0x40`
- `FT_OUTLINE_HIGH_PRECISION :: 0x100`
- `FT_OUTLINE_SINGLE_PASS :: 0x200`
- `ft_outline_none :: FT_OUTLINE_NONE`
- `ft_outline_owner :: FT_OUTLINE_OWNER`
- `ft_outline_even_odd_fill :: FT_OUTLINE_EVEN_ODD_FILL`
- `ft_outline_reverse_fill :: FT_OUTLINE_REVERSE_FILL`
- `ft_outline_ignore_dropouts :: FT_OUTLINE_IGNORE_DROPOUTS`
- `ft_outline_high_precision :: FT_OUTLINE_HIGH_PRECISION`
- `ft_outline_single_pass :: FT_OUTLINE_SINGLE_PASS`
- `FT_CURVE_TAG_ON :: 0x01`
- `FT_CURVE_TAG_CONIC :: 0x00`
- `FT_CURVE_TAG_CUBIC :: 0x02`
- `FT_CURVE_TAG_HAS_SCANMODE :: 0x04`
- `FT_CURVE_TAG_TOUCH_X :: 0x08`
- `FT_CURVE_TAG_TOUCH_Y :: 0x10`
- `FT_CURVE_TAG_TOUCH_BOTH :: FT_CURVE_TAG_TOUCH_X | FT_CURVE_TAG_TOUCH_Y`
- `FT_Curve_Tag_On :: FT_CURVE_TAG_ON`
- `FT_Curve_Tag_Conic :: FT_CURVE_TAG_CONIC`
- `FT_Curve_Tag_Cubic :: FT_CURVE_TAG_CUBIC`
- `FT_Curve_Tag_Touch_X :: FT_CURVE_TAG_TOUCH_X`
- `FT_Curve_Tag_Touch_Y :: FT_CURVE_TAG_TOUCH_Y`
- `FT_Outline_MoveTo_Func :: FT_Outline_MoveToFunc`
- `FT_Outline_LineTo_Func :: FT_Outline_LineToFunc`
- `FT_Outline_ConicTo_Func :: FT_Outline_ConicToFunc`
- `FT_Outline_CubicTo_Func :: FT_Outline_CubicToFunc`
- `FT_Raster_Span_Func :: FT_SpanFunc`
- `FT_RASTER_FLAG_DEFAULT :: 0x0`
- `FT_RASTER_FLAG_AA :: 0x1`
- `FT_RASTER_FLAG_DIRECT :: 0x2`
- `FT_RASTER_FLAG_CLIP :: 0x4`
- `FT_RASTER_FLAG_SDF :: 0x8`
- `ft_raster_flag_default :: FT_RASTER_FLAG_DEFAULT`
- `ft_raster_flag_aa :: FT_RASTER_FLAG_AA`
- `ft_raster_flag_direct :: FT_RASTER_FLAG_DIRECT`
- `ft_raster_flag_clip :: FT_RASTER_FLAG_CLIP`
- `FT_Raster_New_Func :: FT_Raster_NewFunc`
- `FT_Raster_Done_Func :: FT_Raster_DoneFunc`
- `FT_Raster_Reset_Func :: FT_Raster_ResetFunc`
- `FT_Raster_Set_Mode_Func :: FT_Raster_SetModeFunc`
- `FT_Raster_Render_Func :: FT_Raster_RenderFunc`
- `FT_ERR_BASE :: 0`
- `FT_FACE_FLAG_SCALABLE :: 1 << 0`
- `FT_FACE_FLAG_FIXED_SIZES :: 1 << 1`
- `FT_FACE_FLAG_FIXED_WIDTH :: 1 << 2`
- `FT_FACE_FLAG_SFNT :: 1 << 3`
- `FT_FACE_FLAG_HORIZONTAL :: 1 << 4`
- `FT_FACE_FLAG_VERTICAL :: 1 << 5`
- `FT_FACE_FLAG_KERNING :: 1 << 6`
- `FT_FACE_FLAG_FAST_GLYPHS :: 1 << 7`
- `FT_FACE_FLAG_MULTIPLE_MASTERS :: 1 << 8`
- `FT_FACE_FLAG_GLYPH_NAMES :: 1 << 9`
- `FT_FACE_FLAG_EXTERNAL_STREAM :: 1 << 10`
- `FT_FACE_FLAG_HINTER :: 1 << 11`
- `FT_FACE_FLAG_CID_KEYED :: 1 << 12`
- `FT_FACE_FLAG_TRICKY :: 1 << 13`
- `FT_FACE_FLAG_COLOR :: 1 << 14`
- `FT_FACE_FLAG_VARIATION :: 1 << 15`
- `FT_FACE_FLAG_SVG :: 1 << 16`
- `FT_FACE_FLAG_SBIX :: 1 << 17`
- `FT_FACE_FLAG_SBIX_OVERLAY :: 1 << 18`
- `FT_STYLE_FLAG_ITALIC :: 1 << 0`
- `FT_STYLE_FLAG_BOLD :: 1 << 1`
- `FT_OPEN_MEMORY :: 0x1`
- `FT_OPEN_STREAM :: 0x2`
- `FT_OPEN_PATHNAME :: 0x4`
- `FT_OPEN_DRIVER :: 0x8`
- `FT_OPEN_PARAMS :: 0x10`
- `ft_open_memory :: FT_OPEN_MEMORY`
- `ft_open_stream :: FT_OPEN_STREAM`
- `ft_open_pathname :: FT_OPEN_PATHNAME`
- `ft_open_driver :: FT_OPEN_DRIVER`
- `ft_open_params :: FT_OPEN_PARAMS`
- `FT_LOAD_DEFAULT :: 0x0`
- `FT_LOAD_NO_SCALE :: 1 << 0`
- `FT_LOAD_NO_HINTING :: 1 << 1`
- `FT_LOAD_RENDER :: 1 << 2`
- `FT_LOAD_NO_BITMAP :: 1 << 3`
- `FT_LOAD_VERTICAL_LAYOUT :: 1 << 4`
- `FT_LOAD_FORCE_AUTOHINT :: 1 << 5`
- `FT_LOAD_CROP_BITMAP :: 1 << 6`
- `FT_LOAD_PEDANTIC :: 1 << 7`
- `FT_LOAD_IGNORE_GLOBAL_ADVANCE_WIDTH :: 1 << 9`
- `FT_LOAD_NO_RECURSE :: 1 << 10`
- `FT_LOAD_IGNORE_TRANSFORM :: 1 << 11`
- `FT_LOAD_MONOCHROME :: 1 << 12`
- `FT_LOAD_LINEAR_DESIGN :: 1 << 13`
- `FT_LOAD_SBITS_ONLY :: 1 << 14`
- `FT_LOAD_NO_AUTOHINT :: 1 << 15`
- `FT_LOAD_COLOR :: 1 << 20`
- `FT_LOAD_COMPUTE_METRICS :: 1 << 21`
- `FT_LOAD_BITMAP_METRICS_ONLY :: 1 << 22`
- `FT_LOAD_ADVANCE_ONLY :: 1 << 8`
- `FT_LOAD_SVG_ONLY :: 1 << 23`
- `FT_SUBGLYPH_FLAG_ARGS_ARE_WORDS :: 1`
- `FT_SUBGLYPH_FLAG_ARGS_ARE_XY_VALUES :: 2`
- `FT_SUBGLYPH_FLAG_ROUND_XY_TO_GRID :: 4`
- `FT_SUBGLYPH_FLAG_SCALE :: 8`
- `FT_SUBGLYPH_FLAG_XY_SCALE :: 0x40`
- `FT_SUBGLYPH_FLAG_2X2 :: 0x80`
- `FT_SUBGLYPH_FLAG_USE_MY_METRICS :: 0x200`
- `FT_FSTYPE_INSTALLABLE_EMBEDDING :: 0x0000`
- `FT_FSTYPE_RESTRICTED_LICENSE_EMBEDDING :: 0x0002`
- `FT_FSTYPE_PREVIEW_AND_PRINT_EMBEDDING :: 0x0004`
- `FT_FSTYPE_EDITABLE_EMBEDDING :: 0x0008`
- `FT_FSTYPE_NO_SUBSETTING :: 0x0100`
- `FT_FSTYPE_BITMAP_EMBEDDING_ONLY :: 0x0200`
- `FREETYPE_MAJOR :: 2`
- `FREETYPE_MINOR :: 12`
- `FREETYPE_PATCH :: 1`
- `FT_Int16 :: s16`
- `FT_UInt16 :: u16`
- `FT_Int32 :: s32`
- `FT_UInt32 :: u32`
- `FT_Fast :: s32`
- `FT_UFast :: u32`
- `FT_Int64 :: s64`
- `FT_UInt64 :: u64`
- `FT_Memory :: *FT_MemoryRec_`
- `FT_Alloc_Func :: #type (memory: FT_Memory, size: s64) -> *void #c_call`
- `FT_Free_Func :: #type (memory: FT_Memory, block: *void) -> void #c_call`
- `FT_Realloc_Func :: #type (memory: FT_Memory, cur_size: s64, new_size: s64, block: *void) -> *void #c_call`
- `FT_MemoryRec_ :: struct`
- `FT_Stream :: *FT_StreamRec_`
- `FT_StreamDesc_ :: struct`
- `FT_StreamDesc :: FT_StreamDesc_`
- `FT_Stream_IoFunc :: #type (stream: FT_Stream, offset: u64, buffer: *u8, count: u64) -> u64 #c_call`
- `FT_Stream_CloseFunc :: #type (stream: FT_Stream) -> void #c_call`
- `FT_StreamRec_ :: struct`
- `FT_StreamRec :: FT_StreamRec_`
- `FT_Pos :: s64`
- `FT_Vector_ :: struct`
- `FT_Vector :: FT_Vector_`
- `FT_BBox_ :: struct`
- `FT_BBox :: FT_BBox_`
- `FT_Pixel_Mode_ :: enum u32`
- `FT_Pixel_Mode :: FT_Pixel_Mode_`
- `FT_Bitmap_ :: struct`
- `FT_Bitmap :: FT_Bitmap_`
- `FT_Outline_ :: struct`
- `FT_Outline :: FT_Outline_`
- `FT_Outline_MoveToFunc :: #type (to: *FT_Vector, user: *void) -> s32 #c_call`
- `FT_Outline_LineToFunc :: #type (to: *FT_Vector, user: *void) -> s32 #c_call`
- `FT_Outline_ConicToFunc :: #type (control: *FT_Vector, to: *FT_Vector, user: *void) -> s32 #c_call`
- `FT_Outline_CubicToFunc :: #type (control1: *FT_Vector, control2: *FT_Vector, to: *FT_Vector, user: *void) -> s32 #c_call`
- `FT_Outline_Funcs_ :: struct`
- `FT_Outline_Funcs :: FT_Outline_Funcs_`
- `FT_Glyph_Format_ :: enum u32`
- `FT_Glyph_Format :: FT_Glyph_Format_`
- `FT_Span_ :: struct`
- `FT_Span :: FT_Span_`
- `FT_SpanFunc :: #type (y: s32, count: s32, spans: *FT_Span, user: *void) -> void #c_call`
- `FT_Raster_BitTest_Func :: #type (y: s32, x: s32, user: *void) -> s32 #c_call`
- `FT_Raster_BitSet_Func :: #type (y: s32, x: s32, user: *void) -> void #c_call`
- `FT_Raster_Params_ :: struct`
- `FT_Raster_Params :: FT_Raster_Params_`
- `FT_RasterRec_ :: struct`
- `FT_Raster :: *FT_RasterRec_`
- `FT_Raster_NewFunc :: #type (memory: *void, raster: *FT_Raster) -> s32 #c_call`
- `FT_Raster_DoneFunc :: #type (raster: FT_Raster) -> void #c_call`
- `FT_Raster_ResetFunc :: #type (raster: FT_Raster, pool_base: *u8, pool_size: u64) -> void #c_call`
- `FT_Raster_SetModeFunc :: #type (raster: FT_Raster, mode: u64, args: *void) -> s32 #c_call`
- `FT_Raster_RenderFunc :: #type (raster: FT_Raster, params: *FT_Raster_Params) -> s32 #c_call`
- `FT_Raster_Funcs_ :: struct`
- `FT_Raster_Funcs :: FT_Raster_Funcs_`
- `FT_Bool :: u8`
- `FT_FWord :: s16`
- `FT_UFWord :: u16`
- `FT_Char :: s8`
- `FT_Byte :: u8`
- `FT_Bytes :: *FT_Byte`
- `FT_Tag :: FT_UInt32`
- `FT_String :: u8`
- `FT_Short :: s16`
- `FT_UShort :: u16`
- `FT_Int :: s32`
- `FT_UInt :: u32`
- `FT_Long :: s64`
- `FT_ULong :: u64`
- `FT_F2Dot14 :: s16`
- `FT_F26Dot6 :: s64`
- `FT_Fixed :: s64`
- `FT_Error :: s32`
- `FT_Pointer :: *void`
- `FT_Offset :: u64`
- `FT_PtrDist :: ptrdiff_t`
- `FT_UnitVector_ :: struct`
- `FT_UnitVector :: FT_UnitVector_`
- `FT_Matrix_ :: struct`
- `FT_Matrix :: FT_Matrix_`
- `FT_Data_ :: struct`
- `FT_Data :: FT_Data_`
- `FT_Generic_Finalizer :: #type (object: *void) -> void #c_call`
- `FT_Generic_ :: struct`
- `FT_Generic :: FT_Generic_`
- `FT_ListNode :: *FT_ListNodeRec_`
- `FT_List :: *FT_ListRec_`
- `FT_ListNodeRec_ :: struct`
- `FT_ListNodeRec :: FT_ListNodeRec_`
- `FT_ListRec_ :: struct`
- `FT_ListRec :: FT_ListRec_`
- `FT_Mod_Err :: enum u32`
- `FT_Err :: enum u32`
- `FT_Error_String :: (error_code: FT_Error) -> *u8 #foreign freetype`
- `FT_Glyph_Metrics_ :: struct`
- `FT_Glyph_Metrics :: FT_Glyph_Metrics_`
- `FT_Bitmap_Size_ :: struct`
- `FT_Bitmap_Size :: FT_Bitmap_Size_`
- `FT_LibraryRec_ :: struct`
- `FT_Library :: *FT_LibraryRec_`
- `FT_ModuleRec_ :: struct`
- `FT_Module :: *FT_ModuleRec_`
- `FT_DriverRec_ :: struct`
- `FT_Driver :: *FT_DriverRec_`
- `FT_RendererRec_ :: struct`
- `FT_Renderer :: *FT_RendererRec_`
- `FT_Face :: *FT_FaceRec_`
- `FT_Size :: *FT_SizeRec_`
- `FT_GlyphSlot :: *FT_GlyphSlotRec_`
- `FT_CharMap :: *FT_CharMapRec_`
- `FT_Encoding_ :: enum u32`
- `FT_Encoding :: FT_Encoding_`
- `FT_CharMapRec_ :: struct`
- `FT_CharMapRec :: FT_CharMapRec_`
- `FT_Face_InternalRec_ :: struct`
- `FT_Face_Internal :: *FT_Face_InternalRec_`
- `FT_FaceRec_ :: struct`
- `FT_FaceRec :: FT_FaceRec_`
- `FT_Size_InternalRec_ :: struct`
- `FT_Size_Internal :: *FT_Size_InternalRec_`
- `FT_Size_Metrics_ :: struct`
- `FT_Size_Metrics :: FT_Size_Metrics_`
- `FT_SizeRec_ :: struct`
- `FT_SizeRec :: FT_SizeRec_`
- `FT_SubGlyphRec_ :: struct`
- `FT_SubGlyph :: *FT_SubGlyphRec_`
- `FT_Slot_InternalRec_ :: struct`
- `FT_Slot_Internal :: *FT_Slot_InternalRec_`
- `FT_GlyphSlotRec_ :: struct`
- `FT_GlyphSlotRec :: FT_GlyphSlotRec_`
- `FT_Init_FreeType :: (alibrary: *FT_Library) -> FT_Error #foreign freetype`
- `FT_Done_FreeType :: (library: FT_Library) -> FT_Error #foreign freetype`
- `FT_Parameter_ :: struct`
- `FT_Parameter :: FT_Parameter_`
- `FT_Open_Args_ :: struct`
- `FT_Open_Args :: FT_Open_Args_`
- `FT_New_Face :: (library: FT_Library, filepathname: *u8, face_index: FT_Long, aface: *FT_Face) -> FT_Error #foreign freetype`
- `FT_New_Memory_Face :: (library: FT_Library, file_base: *FT_Byte, file_size: FT_Long, face_index: FT_Long, aface: *FT_Face) -> FT_Error #foreign freetype`
- `FT_Open_Face :: (library: FT_Library, args: *FT_Open_Args, face_index: FT_Long, aface: *FT_Face) -> FT_Error #foreign freetype`
- `FT_Attach_File :: (face: FT_Face, filepathname: *u8) -> FT_Error #foreign freetype`
- `FT_Attach_Stream :: (face: FT_Face, parameters: *FT_Open_Args) -> FT_Error #foreign freetype`
- `FT_Reference_Face :: (face: FT_Face) -> FT_Error #foreign freetype`
- `FT_Done_Face :: (face: FT_Face) -> FT_Error #foreign freetype`
- `FT_Select_Size :: (face: FT_Face, strike_index: FT_Int) -> FT_Error #foreign freetype`
- `FT_Size_Request_Type_ :: enum u32`
- `FT_Size_Request_Type :: FT_Size_Request_Type_`
- `FT_Size_RequestRec_ :: struct`
- `FT_Size_RequestRec :: FT_Size_RequestRec_`
- `FT_Size_Request :: *FT_Size_RequestRec_`
- `FT_Request_Size :: (face: FT_Face, req: FT_Size_Request) -> FT_Error #foreign freetype`
- `FT_Set_Char_Size :: (face: FT_Face, char_width: FT_F26Dot6, char_height: FT_F26Dot6, horz_resolution: FT_UInt, vert_resolution: FT_UInt) -> FT_Error #foreign freetype`
- `FT_Set_Pixel_Sizes :: (face: FT_Face, pixel_width: FT_UInt, pixel_height: FT_UInt) -> FT_Error #foreign freetype`
- `FT_Load_Glyph :: (face: FT_Face, glyph_index: FT_UInt, load_flags: FT_Int32) -> FT_Error #foreign freetype`
- `FT_Load_Char :: (face: FT_Face, char_code: FT_ULong, load_flags: FT_Int32) -> FT_Error #foreign freetype`
- `FT_Set_Transform :: (face: FT_Face, matrix: *FT_Matrix, delta: *FT_Vector) -> void #foreign freetype`
- `FT_Get_Transform :: (face: FT_Face, matrix: *FT_Matrix, delta: *FT_Vector) -> void #foreign freetype`
- `FT_Render_Mode_ :: enum u32`
- `FT_Render_Mode :: FT_Render_Mode_`
- `FT_Render_Glyph :: (slot: FT_GlyphSlot, render_mode: FT_Render_Mode) -> FT_Error #foreign freetype`
- `FT_Kerning_Mode_ :: enum u32`
- `FT_Kerning_Mode :: FT_Kerning_Mode_`
- `FT_Get_Kerning :: (face: FT_Face, left_glyph: FT_UInt, right_glyph: FT_UInt, kern_mode: FT_UInt, akerning: *FT_Vector) -> FT_Error #foreign freetype`
- `FT_Get_Track_Kerning :: (face: FT_Face, point_size: FT_Fixed, degree: FT_Int, akerning: *FT_Fixed) -> FT_Error #foreign freetype`
- `FT_Get_Glyph_Name :: (face: FT_Face, glyph_index: FT_UInt, buffer: FT_Pointer, buffer_max: FT_UInt) -> FT_Error #foreign freetype`
- `FT_Get_Postscript_Name :: (face: FT_Face) -> *u8 #foreign freetype`
- `FT_Select_Charmap :: (face: FT_Face, encoding: FT_Encoding) -> FT_Error #foreign freetype`
- `FT_Set_Charmap :: (face: FT_Face, charmap: FT_CharMap) -> FT_Error #foreign freetype`
- `FT_Get_Charmap_Index :: (charmap: FT_CharMap) -> FT_Int #foreign freetype`
- `FT_Get_Char_Index :: (face: FT_Face, charcode: FT_ULong) -> FT_UInt #foreign freetype`
- `FT_Get_First_Char :: (face: FT_Face, agindex: *FT_UInt) -> FT_ULong #foreign freetype`
- `FT_Get_Next_Char :: (face: FT_Face, char_code: FT_ULong, agindex: *FT_UInt) -> FT_ULong #foreign freetype`
- `FT_Face_Properties :: (face: FT_Face, num_properties: FT_UInt, properties: *FT_Parameter) -> FT_Error #foreign freetype`
- `FT_Get_Name_Index :: (face: FT_Face, glyph_name: *FT_String) -> FT_UInt #foreign freetype`
- `FT_Get_SubGlyph_Info :: (glyph: FT_GlyphSlot, sub_index: FT_UInt, p_index: *FT_Int, p_flags: *FT_UInt, p_arg1: *FT_Int, p_arg2: *FT_Int, p_transform: *FT_Matrix) -> FT_Error #foreign freetype`
- `FT_Get_FSType_Flags :: (face: FT_Face) -> FT_UShort #foreign freetype`
- `FT_Face_GetCharVariantIndex :: (face: FT_Face, charcode: FT_ULong, variantSelector: FT_ULong) -> FT_UInt #foreign freetype`
- `FT_Face_GetCharVariantIsDefault :: (face: FT_Face, charcode: FT_ULong, variantSelector: FT_ULong) -> FT_Int #foreign freetype`
- `FT_Face_GetVariantSelectors :: (face: FT_Face) -> *FT_UInt32 #foreign freetype`
- `FT_Face_GetVariantsOfChar :: (face: FT_Face, charcode: FT_ULong) -> *FT_UInt32 #foreign freetype`
- `FT_Face_GetCharsOfVariant :: (face: FT_Face, variantSelector: FT_ULong) -> *FT_UInt32 #foreign freetype`
- `FT_MulDiv :: (a: FT_Long, b: FT_Long, c: FT_Long) -> FT_Long #foreign freetype`
- `FT_MulFix :: (a: FT_Long, b: FT_Long) -> FT_Long #foreign freetype`
- `FT_DivFix :: (a: FT_Long, b: FT_Long) -> FT_Long #foreign freetype`
- `FT_RoundFix :: (a: FT_Fixed) -> FT_Fixed #foreign freetype`
- `FT_CeilFix :: (a: FT_Fixed) -> FT_Fixed #foreign freetype`
- `FT_FloorFix :: (a: FT_Fixed) -> FT_Fixed #foreign freetype`
- `FT_Vector_Transform :: (vector: *FT_Vector, matrix: *FT_Matrix) -> void #foreign freetype`
- `FT_Library_Version :: (library: FT_Library, amajor: *FT_Int, aminor: *FT_Int, apatch: *FT_Int) -> void #foreign freetype`
- `FT_Face_CheckTrueTypePatents :: (face: FT_Face) -> FT_Bool #foreign freetype`
- `FT_Face_SetUnpatentedHinting :: (face: FT_Face, value: FT_Bool) -> FT_Bool #foreign freetype`
- `TT_Header_ :: struct`
- `TT_Header :: TT_Header_`
- `TT_HoriHeader_ :: struct`
- `TT_HoriHeader :: TT_HoriHeader_`
- `TT_VertHeader_ :: struct`
- `TT_VertHeader :: TT_VertHeader_`
- `TT_OS2_ :: struct`
- `TT_OS2 :: TT_OS2_`
- `TT_Postscript_ :: struct`
- `TT_Postscript :: TT_Postscript_`
- `TT_PCLT_ :: struct`
- `TT_PCLT :: TT_PCLT_`
- `TT_MaxProfile_ :: struct`
- `TT_MaxProfile :: TT_MaxProfile_`
- `FT_Sfnt_Tag_ :: enum u32`
- `FT_Sfnt_Tag :: FT_Sfnt_Tag_`
- `FT_Get_Sfnt_Table :: (face: FT_Face, tag: FT_Sfnt_Tag) -> *void #foreign freetype`
- `FT_Load_Sfnt_Table :: (face: FT_Face, tag: FT_ULong, offset: FT_Long, buffer: *FT_Byte, length: *FT_ULong) -> FT_Error #foreign freetype`
- `FT_Sfnt_Table_Info :: (face: FT_Face, table_index: FT_UInt, tag: *FT_ULong, length: *FT_ULong) -> FT_Error #foreign freetype`
- `FT_Get_CMap_Language_ID :: (charmap: FT_CharMap) -> FT_ULong #foreign freetype`
- `FT_Get_CMap_Format :: (charmap: FT_CharMap) -> FT_Long #foreign freetype`

### Windows-specific declarations

- `FT_RENDER_POOL_SIZE :: 16384`
- `FT_MAX_MODULES :: 32`
- `TT_CONFIG_OPTION_SUBPIXEL_HINTING :: 2`
- `TT_CONFIG_OPTION_MAX_RUNNABLE_OPCODES :: 1000000`
- `T1_MAX_DICT_DEPTH :: 5`
- `T1_MAX_SUBRS_CALLS :: 16`
- `T1_MAX_CHARSTRINGS_OPERANDS :: 256`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_X1 :: 500`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_Y1 :: 400`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_X2 :: 1000`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_Y2 :: 275`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_X3 :: 1667`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_Y3 :: 275`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_X4 :: 2333`
- `CFF_CONFIG_OPTION_DARKENING_PARAMETER_Y4 :: 0`
- `FT_OUTLINE_NONE :: 0x0`
- `FT_OUTLINE_OWNER :: 0x1`
- `FT_OUTLINE_EVEN_ODD_FILL :: 0x2`
- `FT_OUTLINE_REVERSE_FILL :: 0x4`
- `FT_OUTLINE_IGNORE_DROPOUTS :: 0x8`
- `FT_OUTLINE_SMART_DROPOUTS :: 0x10`
- `FT_OUTLINE_INCLUDE_STUBS :: 0x20`
- `FT_OUTLINE_OVERLAP :: 0x40`
- `FT_OUTLINE_HIGH_PRECISION :: 0x100`
- `FT_OUTLINE_SINGLE_PASS :: 0x200`
- `ft_outline_none :: FT_OUTLINE_NONE`
- `ft_outline_owner :: FT_OUTLINE_OWNER`
- `ft_outline_even_odd_fill :: FT_OUTLINE_EVEN_ODD_FILL`
- `ft_outline_reverse_fill :: FT_OUTLINE_REVERSE_FILL`
- `ft_outline_ignore_dropouts :: FT_OUTLINE_IGNORE_DROPOUTS`
- `ft_outline_high_precision :: FT_OUTLINE_HIGH_PRECISION`
- `ft_outline_single_pass :: FT_OUTLINE_SINGLE_PASS`
- `FT_CURVE_TAG_ON :: 0x01`
- `FT_CURVE_TAG_CONIC :: 0x00`
- `FT_CURVE_TAG_CUBIC :: 0x02`
- `FT_CURVE_TAG_HAS_SCANMODE :: 0x04`
- `FT_CURVE_TAG_TOUCH_X :: 0x08`
- `FT_CURVE_TAG_TOUCH_Y :: 0x10`
- `FT_CURVE_TAG_TOUCH_BOTH :: FT_CURVE_TAG_TOUCH_X | FT_CURVE_TAG_TOUCH_Y`
- `FT_Curve_Tag_On :: FT_CURVE_TAG_ON`
- `FT_Curve_Tag_Conic :: FT_CURVE_TAG_CONIC`
- `FT_Curve_Tag_Cubic :: FT_CURVE_TAG_CUBIC`
- `FT_Curve_Tag_Touch_X :: FT_CURVE_TAG_TOUCH_X`
- `FT_Curve_Tag_Touch_Y :: FT_CURVE_TAG_TOUCH_Y`
- `FT_Outline_MoveTo_Func :: FT_Outline_MoveToFunc`
- `FT_Outline_LineTo_Func :: FT_Outline_LineToFunc`
- `FT_Outline_ConicTo_Func :: FT_Outline_ConicToFunc`
- `FT_Outline_CubicTo_Func :: FT_Outline_CubicToFunc`
- `FT_Raster_Span_Func :: FT_SpanFunc`
- `FT_RASTER_FLAG_DEFAULT :: 0x0`
- `FT_RASTER_FLAG_AA :: 0x1`
- `FT_RASTER_FLAG_DIRECT :: 0x2`
- `FT_RASTER_FLAG_CLIP :: 0x4`
- `FT_RASTER_FLAG_SDF :: 0x8`
- `ft_raster_flag_default :: FT_RASTER_FLAG_DEFAULT`
- `ft_raster_flag_aa :: FT_RASTER_FLAG_AA`
- `ft_raster_flag_direct :: FT_RASTER_FLAG_DIRECT`
- `ft_raster_flag_clip :: FT_RASTER_FLAG_CLIP`
- `FT_Raster_New_Func :: FT_Raster_NewFunc`
- `FT_Raster_Done_Func :: FT_Raster_DoneFunc`
- `FT_Raster_Reset_Func :: FT_Raster_ResetFunc`
- `FT_Raster_Set_Mode_Func :: FT_Raster_SetModeFunc`
- `FT_Raster_Render_Func :: FT_Raster_RenderFunc`
- `FT_ERR_BASE :: 0`
- `FT_FACE_FLAG_SCALABLE :: 1 << 0`
- `FT_FACE_FLAG_FIXED_SIZES :: 1 << 1`
- `FT_FACE_FLAG_FIXED_WIDTH :: 1 << 2`
- `FT_FACE_FLAG_SFNT :: 1 << 3`
- `FT_FACE_FLAG_HORIZONTAL :: 1 << 4`
- `FT_FACE_FLAG_VERTICAL :: 1 << 5`
- `FT_FACE_FLAG_KERNING :: 1 << 6`
- `FT_FACE_FLAG_FAST_GLYPHS :: 1 << 7`
- `FT_FACE_FLAG_MULTIPLE_MASTERS :: 1 << 8`
- `FT_FACE_FLAG_GLYPH_NAMES :: 1 << 9`
- `FT_FACE_FLAG_EXTERNAL_STREAM :: 1 << 10`
- `FT_FACE_FLAG_HINTER :: 1 << 11`
- `FT_FACE_FLAG_CID_KEYED :: 1 << 12`
- `FT_FACE_FLAG_TRICKY :: 1 << 13`
- `FT_FACE_FLAG_COLOR :: 1 << 14`
- `FT_FACE_FLAG_VARIATION :: 1 << 15`
- `FT_FACE_FLAG_SVG :: 1 << 16`
- `FT_FACE_FLAG_SBIX :: 1 << 17`
- `FT_FACE_FLAG_SBIX_OVERLAY :: 1 << 18`
- `FT_STYLE_FLAG_ITALIC :: 1 << 0`
- `FT_STYLE_FLAG_BOLD :: 1 << 1`
- `FT_OPEN_MEMORY :: 0x1`
- `FT_OPEN_STREAM :: 0x2`
- `FT_OPEN_PATHNAME :: 0x4`
- `FT_OPEN_DRIVER :: 0x8`
- `FT_OPEN_PARAMS :: 0x10`
- `ft_open_memory :: FT_OPEN_MEMORY`
- `ft_open_stream :: FT_OPEN_STREAM`
- `ft_open_pathname :: FT_OPEN_PATHNAME`
- `ft_open_driver :: FT_OPEN_DRIVER`
- `ft_open_params :: FT_OPEN_PARAMS`
- `FT_LOAD_DEFAULT :: 0x0`
- `FT_LOAD_NO_SCALE :: 1 << 0`
- `FT_LOAD_NO_HINTING :: 1 << 1`
- `FT_LOAD_RENDER :: 1 << 2`
- `FT_LOAD_NO_BITMAP :: 1 << 3`
- `FT_LOAD_VERTICAL_LAYOUT :: 1 << 4`
- `FT_LOAD_FORCE_AUTOHINT :: 1 << 5`
- `FT_LOAD_CROP_BITMAP :: 1 << 6`
- `FT_LOAD_PEDANTIC :: 1 << 7`
- `FT_LOAD_IGNORE_GLOBAL_ADVANCE_WIDTH :: 1 << 9`
- `FT_LOAD_NO_RECURSE :: 1 << 10`
- `FT_LOAD_IGNORE_TRANSFORM :: 1 << 11`
- `FT_LOAD_MONOCHROME :: 1 << 12`
- `FT_LOAD_LINEAR_DESIGN :: 1 << 13`
- `FT_LOAD_SBITS_ONLY :: 1 << 14`
- `FT_LOAD_NO_AUTOHINT :: 1 << 15`
- `FT_LOAD_COLOR :: 1 << 20`
- `FT_LOAD_COMPUTE_METRICS :: 1 << 21`
- `FT_LOAD_BITMAP_METRICS_ONLY :: 1 << 22`
- `FT_LOAD_ADVANCE_ONLY :: 1 << 8`
- `FT_LOAD_SVG_ONLY :: 1 << 23`
- `FT_SUBGLYPH_FLAG_ARGS_ARE_WORDS :: 1`
- `FT_SUBGLYPH_FLAG_ARGS_ARE_XY_VALUES :: 2`
- `FT_SUBGLYPH_FLAG_ROUND_XY_TO_GRID :: 4`
- `FT_SUBGLYPH_FLAG_SCALE :: 8`
- `FT_SUBGLYPH_FLAG_XY_SCALE :: 0x40`
- `FT_SUBGLYPH_FLAG_2X2 :: 0x80`
- `FT_SUBGLYPH_FLAG_USE_MY_METRICS :: 0x200`
- `FT_FSTYPE_INSTALLABLE_EMBEDDING :: 0x0000`
- `FT_FSTYPE_RESTRICTED_LICENSE_EMBEDDING :: 0x0002`
- `FT_FSTYPE_PREVIEW_AND_PRINT_EMBEDDING :: 0x0004`
- `FT_FSTYPE_EDITABLE_EMBEDDING :: 0x0008`
- `FT_FSTYPE_NO_SUBSETTING :: 0x0100`
- `FT_FSTYPE_BITMAP_EMBEDDING_ONLY :: 0x0200`
- `FREETYPE_MAJOR :: 2`
- `FREETYPE_MINOR :: 12`
- `FREETYPE_PATCH :: 1`
- `FT_Int16 :: s16`
- `FT_UInt16 :: u16`
- `FT_Int32 :: s32`
- `FT_UInt32 :: u32`
- `FT_Fast :: s32`
- `FT_UFast :: u32`
- `FT_Int64 :: s64`
- `FT_UInt64 :: u64`
- `FT_Memory :: *FT_MemoryRec_`
- `FT_Alloc_Func :: #type (memory: FT_Memory, size: s32) -> *void #c_call`
- `FT_Free_Func :: #type (memory: FT_Memory, block: *void) -> void #c_call`
- `FT_Realloc_Func :: #type (memory: FT_Memory, cur_size: s32, new_size: s32, block: *void) -> *void #c_call`
- `FT_MemoryRec_ :: struct`
- `FT_Stream :: *FT_StreamRec_`
- `FT_StreamDesc_ :: struct`
- `FT_StreamDesc :: FT_StreamDesc_`
- `FT_Stream_IoFunc :: #type (stream: FT_Stream, offset: u32, buffer: *u8, count: u32) -> u32 #c_call`
- `FT_Stream_CloseFunc :: #type (stream: FT_Stream) -> void #c_call`
- `FT_StreamRec_ :: struct`
- `FT_StreamRec :: FT_StreamRec_`
- `FT_Pos :: s32`
- `FT_Vector_ :: struct`
- `FT_Vector :: FT_Vector_`
- `FT_BBox_ :: struct`
- `FT_BBox :: FT_BBox_`
- `FT_Pixel_Mode_ :: enum s32`
- `FT_Pixel_Mode :: FT_Pixel_Mode_`
- `FT_Bitmap_ :: struct`
- `FT_Bitmap :: FT_Bitmap_`
- `FT_Outline_ :: struct`
- `FT_Outline :: FT_Outline_`
- `FT_Outline_MoveToFunc :: #type (to: *FT_Vector, user: *void) -> s32 #c_call`
- `FT_Outline_LineToFunc :: #type (to: *FT_Vector, user: *void) -> s32 #c_call`
- `FT_Outline_ConicToFunc :: #type (control: *FT_Vector, to: *FT_Vector, user: *void) -> s32 #c_call`
- `FT_Outline_CubicToFunc :: #type (control1: *FT_Vector, control2: *FT_Vector, to: *FT_Vector, user: *void) -> s32 #c_call`
- `FT_Outline_Funcs_ :: struct`
- `FT_Outline_Funcs :: FT_Outline_Funcs_`
- `FT_Glyph_Format_ :: enum s32`
- `FT_Glyph_Format :: FT_Glyph_Format_`
- `FT_Span_ :: struct`
- `FT_Span :: FT_Span_`
- `FT_SpanFunc :: #type (y: s32, count: s32, spans: *FT_Span, user: *void) -> void #c_call`
- `FT_Raster_BitTest_Func :: #type (y: s32, x: s32, user: *void) -> s32 #c_call`
- `FT_Raster_BitSet_Func :: #type (y: s32, x: s32, user: *void) -> void #c_call`
- `FT_Raster_Params_ :: struct`
- `FT_Raster_Params :: FT_Raster_Params_`
- `FT_RasterRec_ :: struct`
- `FT_Raster :: *FT_RasterRec_`
- `FT_Raster_NewFunc :: #type (memory: *void, raster: *FT_Raster) -> s32 #c_call`
- `FT_Raster_DoneFunc :: #type (raster: FT_Raster) -> void #c_call`
- `FT_Raster_ResetFunc :: #type (raster: FT_Raster, pool_base: *u8, pool_size: u32) -> void #c_call`
- `FT_Raster_SetModeFunc :: #type (raster: FT_Raster, mode: u32, args: *void) -> s32 #c_call`
- `FT_Raster_RenderFunc :: #type (raster: FT_Raster, params: *FT_Raster_Params) -> s32 #c_call`
- `FT_Raster_Funcs_ :: struct`
- `FT_Raster_Funcs :: FT_Raster_Funcs_`
- `FT_Bool :: u8`
- `FT_FWord :: s16`
- `FT_UFWord :: u16`
- `FT_Char :: s8`
- `FT_Byte :: u8`
- `FT_Bytes :: *FT_Byte`
- `FT_Tag :: FT_UInt32`
- `FT_String :: u8`
- `FT_Short :: s16`
- `FT_UShort :: u16`
- `FT_Int :: s32`
- `FT_UInt :: u32`
- `FT_Long :: s32`
- `FT_ULong :: u32`
- `FT_F2Dot14 :: s16`
- `FT_F26Dot6 :: s32`
- `FT_Fixed :: s32`
- `FT_Error :: s32`
- `FT_Pointer :: *void`
- `FT_Offset :: u64`
- `FT_PtrDist :: ptrdiff_t`
- `FT_UnitVector_ :: struct`
- `FT_UnitVector :: FT_UnitVector_`
- `FT_Matrix_ :: struct`
- `FT_Matrix :: FT_Matrix_`
- `FT_Data_ :: struct`
- `FT_Data :: FT_Data_`
- `FT_Generic_Finalizer :: #type (object: *void) -> void #c_call`
- `FT_Generic_ :: struct`
- `FT_Generic :: FT_Generic_`
- `FT_ListNode :: *FT_ListNodeRec_`
- `FT_List :: *FT_ListRec_`
- `FT_ListNodeRec_ :: struct`
- `FT_ListNodeRec :: FT_ListNodeRec_`
- `FT_ListRec_ :: struct`
- `FT_ListRec :: FT_ListRec_`
- `FT_Mod_Err :: enum s32`
- `FT_Err :: enum s32`
- `FT_Error_String :: (error_code: FT_Error) -> *u8 #foreign freetype`
- `FT_Glyph_Metrics_ :: struct`
- `FT_Glyph_Metrics :: FT_Glyph_Metrics_`
- `FT_Bitmap_Size_ :: struct`
- `FT_Bitmap_Size :: FT_Bitmap_Size_`
- `FT_LibraryRec_ :: struct`
- `FT_Library :: *FT_LibraryRec_`
- `FT_ModuleRec_ :: struct`
- `FT_Module :: *FT_ModuleRec_`
- `FT_DriverRec_ :: struct`
- `FT_Driver :: *FT_DriverRec_`
- `FT_RendererRec_ :: struct`
- `FT_Renderer :: *FT_RendererRec_`
- `FT_Face :: *FT_FaceRec_`
- `FT_Size :: *FT_SizeRec_`
- `FT_GlyphSlot :: *FT_GlyphSlotRec_`
- `FT_CharMap :: *FT_CharMapRec_`
- `FT_Encoding_ :: enum s32`
- `FT_Encoding :: FT_Encoding_`
- `FT_CharMapRec_ :: struct`
- `FT_CharMapRec :: FT_CharMapRec_`
- `FT_Face_InternalRec_ :: struct`
- `FT_Face_Internal :: *FT_Face_InternalRec_`
- `FT_FaceRec_ :: struct`
- `FT_FaceRec :: FT_FaceRec_`
- `FT_Size_InternalRec_ :: struct`
- `FT_Size_Internal :: *FT_Size_InternalRec_`
- `FT_Size_Metrics_ :: struct`
- `FT_Size_Metrics :: FT_Size_Metrics_`
- `FT_SizeRec_ :: struct`
- `FT_SizeRec :: FT_SizeRec_`
- `FT_SubGlyphRec_ :: struct`
- `FT_SubGlyph :: *FT_SubGlyphRec_`
- `FT_Slot_InternalRec_ :: struct`
- `FT_Slot_Internal :: *FT_Slot_InternalRec_`
- `FT_GlyphSlotRec_ :: struct`
- `FT_GlyphSlotRec :: FT_GlyphSlotRec_`
- `FT_Init_FreeType :: (alibrary: *FT_Library) -> FT_Error #foreign freetype`
- `FT_Done_FreeType :: (library: FT_Library) -> FT_Error #foreign freetype`
- `FT_Parameter_ :: struct`
- `FT_Parameter :: FT_Parameter_`
- `FT_Open_Args_ :: struct`
- `FT_Open_Args :: FT_Open_Args_`
- `FT_New_Face :: (library: FT_Library, filepathname: *u8, face_index: FT_Long, aface: *FT_Face) -> FT_Error #foreign freetype`
- `FT_New_Memory_Face :: (library: FT_Library, file_base: *FT_Byte, file_size: FT_Long, face_index: FT_Long, aface: *FT_Face) -> FT_Error #foreign freetype`
- `FT_Open_Face :: (library: FT_Library, args: *FT_Open_Args, face_index: FT_Long, aface: *FT_Face) -> FT_Error #foreign freetype`
- `FT_Attach_File :: (face: FT_Face, filepathname: *u8) -> FT_Error #foreign freetype`
- `FT_Attach_Stream :: (face: FT_Face, parameters: *FT_Open_Args) -> FT_Error #foreign freetype`
- `FT_Reference_Face :: (face: FT_Face) -> FT_Error #foreign freetype`
- `FT_Done_Face :: (face: FT_Face) -> FT_Error #foreign freetype`
- `FT_Select_Size :: (face: FT_Face, strike_index: FT_Int) -> FT_Error #foreign freetype`
- `FT_Size_Request_Type_ :: enum s32`
- `FT_Size_Request_Type :: FT_Size_Request_Type_`
- `FT_Size_RequestRec_ :: struct`
- `FT_Size_RequestRec :: FT_Size_RequestRec_`
- `FT_Size_Request :: *FT_Size_RequestRec_`
- `FT_Request_Size :: (face: FT_Face, req: FT_Size_Request) -> FT_Error #foreign freetype`
- `FT_Set_Char_Size :: (face: FT_Face, char_width: FT_F26Dot6, char_height: FT_F26Dot6, horz_resolution: FT_UInt, vert_resolution: FT_UInt) -> FT_Error #foreign freetype`
- `FT_Set_Pixel_Sizes :: (face: FT_Face, pixel_width: FT_UInt, pixel_height: FT_UInt) -> FT_Error #foreign freetype`
- `FT_Load_Glyph :: (face: FT_Face, glyph_index: FT_UInt, load_flags: FT_Int32) -> FT_Error #foreign freetype`
- `FT_Load_Char :: (face: FT_Face, char_code: FT_ULong, load_flags: FT_Int32) -> FT_Error #foreign freetype`
- `FT_Set_Transform :: (face: FT_Face, matrix: *FT_Matrix, delta: *FT_Vector) -> void #foreign freetype`
- `FT_Get_Transform :: (face: FT_Face, matrix: *FT_Matrix, delta: *FT_Vector) -> void #foreign freetype`
- `FT_Render_Mode_ :: enum s32`
- `FT_Render_Mode :: FT_Render_Mode_`
- `FT_Render_Glyph :: (slot: FT_GlyphSlot, render_mode: FT_Render_Mode) -> FT_Error #foreign freetype`
- `FT_Kerning_Mode_ :: enum s32`
- `FT_Kerning_Mode :: FT_Kerning_Mode_`
- `FT_Get_Kerning :: (face: FT_Face, left_glyph: FT_UInt, right_glyph: FT_UInt, kern_mode: FT_UInt, akerning: *FT_Vector) -> FT_Error #foreign freetype`
- `FT_Get_Track_Kerning :: (face: FT_Face, point_size: FT_Fixed, degree: FT_Int, akerning: *FT_Fixed) -> FT_Error #foreign freetype`
- `FT_Get_Glyph_Name :: (face: FT_Face, glyph_index: FT_UInt, buffer: FT_Pointer, buffer_max: FT_UInt) -> FT_Error #foreign freetype`
- `FT_Get_Postscript_Name :: (face: FT_Face) -> *u8 #foreign freetype`
- `FT_Select_Charmap :: (face: FT_Face, encoding: FT_Encoding) -> FT_Error #foreign freetype`
- `FT_Set_Charmap :: (face: FT_Face, charmap: FT_CharMap) -> FT_Error #foreign freetype`
- `FT_Get_Charmap_Index :: (charmap: FT_CharMap) -> FT_Int #foreign freetype`
- `FT_Get_Char_Index :: (face: FT_Face, charcode: FT_ULong) -> FT_UInt #foreign freetype`
- `FT_Get_First_Char :: (face: FT_Face, agindex: *FT_UInt) -> FT_ULong #foreign freetype`
- `FT_Get_Next_Char :: (face: FT_Face, char_code: FT_ULong, agindex: *FT_UInt) -> FT_ULong #foreign freetype`
- `FT_Face_Properties :: (face: FT_Face, num_properties: FT_UInt, properties: *FT_Parameter) -> FT_Error #foreign freetype`
- `FT_Get_Name_Index :: (face: FT_Face, glyph_name: *FT_String) -> FT_UInt #foreign freetype`
- `FT_Get_SubGlyph_Info :: (glyph: FT_GlyphSlot, sub_index: FT_UInt, p_index: *FT_Int, p_flags: *FT_UInt, p_arg1: *FT_Int, p_arg2: *FT_Int, p_transform: *FT_Matrix) -> FT_Error #foreign freetype`
- `FT_Get_FSType_Flags :: (face: FT_Face) -> FT_UShort #foreign freetype`
- `FT_Face_GetCharVariantIndex :: (face: FT_Face, charcode: FT_ULong, variantSelector: FT_ULong) -> FT_UInt #foreign freetype`
- `FT_Face_GetCharVariantIsDefault :: (face: FT_Face, charcode: FT_ULong, variantSelector: FT_ULong) -> FT_Int #foreign freetype`
- `FT_Face_GetVariantSelectors :: (face: FT_Face) -> *FT_UInt32 #foreign freetype`
- `FT_Face_GetVariantsOfChar :: (face: FT_Face, charcode: FT_ULong) -> *FT_UInt32 #foreign freetype`
- `FT_Face_GetCharsOfVariant :: (face: FT_Face, variantSelector: FT_ULong) -> *FT_UInt32 #foreign freetype`
- `FT_MulDiv :: (a: FT_Long, b: FT_Long, c: FT_Long) -> FT_Long #foreign freetype`
- `FT_MulFix :: (a: FT_Long, b: FT_Long) -> FT_Long #foreign freetype`
- `FT_DivFix :: (a: FT_Long, b: FT_Long) -> FT_Long #foreign freetype`
- `FT_RoundFix :: (a: FT_Fixed) -> FT_Fixed #foreign freetype`
- `FT_CeilFix :: (a: FT_Fixed) -> FT_Fixed #foreign freetype`
- `FT_FloorFix :: (a: FT_Fixed) -> FT_Fixed #foreign freetype`
- `FT_Vector_Transform :: (vector: *FT_Vector, matrix: *FT_Matrix) -> void #foreign freetype`
- `FT_Library_Version :: (library: FT_Library, amajor: *FT_Int, aminor: *FT_Int, apatch: *FT_Int) -> void #foreign freetype`
- `FT_Face_CheckTrueTypePatents :: (face: FT_Face) -> FT_Bool #foreign freetype`
- `FT_Face_SetUnpatentedHinting :: (face: FT_Face, value: FT_Bool) -> FT_Bool #foreign freetype`
- `TT_Header_ :: struct`
- `TT_Header :: TT_Header_`
- `TT_HoriHeader_ :: struct`
- `TT_HoriHeader :: TT_HoriHeader_`
- `TT_VertHeader_ :: struct`
- `TT_VertHeader :: TT_VertHeader_`
- `TT_OS2_ :: struct`
- `TT_OS2 :: TT_OS2_`
- `TT_Postscript_ :: struct`
- `TT_Postscript :: TT_Postscript_`
- `TT_PCLT_ :: struct`
- `TT_PCLT :: TT_PCLT_`
- `TT_MaxProfile_ :: struct`
- `TT_MaxProfile :: TT_MaxProfile_`
- `FT_Sfnt_Tag_ :: enum s32`
- `FT_Sfnt_Tag :: FT_Sfnt_Tag_`
- `FT_Get_Sfnt_Table :: (face: FT_Face, tag: FT_Sfnt_Tag) -> *void #foreign freetype`
- `FT_Load_Sfnt_Table :: (face: FT_Face, tag: FT_ULong, offset: FT_Long, buffer: *FT_Byte, length: *FT_ULong) -> FT_Error #foreign freetype`
- `FT_Sfnt_Table_Info :: (face: FT_Face, table_index: FT_UInt, tag: *FT_ULong, length: *FT_ULong) -> FT_Error #foreign freetype`
- `FT_Get_CMap_Language_ID :: (charmap: FT_CharMap) -> FT_ULong #foreign freetype`
- `FT_Get_CMap_Format :: (charmap: FT_CharMap) -> FT_Long #foreign freetype`

## Public type members

#### `FT_DriverRec_` (empty opaque external struct)

```jai
FT_DriverRec_ :: struct {}
```
#### `FT_Face_InternalRec_` (empty opaque external struct)

```jai
FT_Face_InternalRec_ :: struct {}
```
#### `FT_LibraryRec_` (empty opaque external struct)

```jai
FT_LibraryRec_ :: struct {}
```
#### `FT_ModuleRec_` (empty opaque external struct)

```jai
FT_ModuleRec_ :: struct {}
```
#### `FT_RasterRec_` (empty opaque external struct)

```jai
FT_RasterRec_ :: struct {}
```
#### `FT_RendererRec_` (empty opaque external struct)

```jai
FT_RendererRec_ :: struct {}
```
#### `FT_Size_InternalRec_` (empty opaque external struct)

```jai
FT_Size_InternalRec_ :: struct {}
```
#### `FT_Slot_InternalRec_` (empty opaque external struct)

```jai
FT_Slot_InternalRec_ :: struct {}
```
#### `FT_SubGlyphRec_` (empty opaque external struct)

```jai
FT_SubGlyphRec_ :: struct {}
```
### Unix-specific type declarations

#### `FT_MemoryRec_` (struct)

- `user:    *void`
- `alloc:   FT_Alloc_Func`
- `free:    FT_Free_Func`
- `realloc: FT_Realloc_Func`

#### `FT_StreamDesc_` (struct)

- `value: s64`

#### `FT_StreamRec_` (struct)

- `base:       *u8`
- `size:       u64`
- `pos:        u64`
- `descriptor: FT_StreamDesc`
- `pathname:   FT_StreamDesc`
- `read:       FT_Stream_IoFunc`
- `close:      FT_Stream_CloseFunc`
- `memory:     FT_Memory`
- `cursor:     *u8`
- `limit:      *u8`

#### `FT_Vector_` (struct)

- `x: FT_Pos`
- `y: FT_Pos`

#### `FT_BBox_` (struct)

- `xMin: FT_Pos`
- `yMin: FT_Pos`
- `xMax: FT_Pos`
- `yMax: FT_Pos`

#### `FT_Pixel_Mode_` (enum)

- `NONE  :: 0`
- `MONO  :: 1`
- `GRAY  :: 2`
- `GRAY2 :: 3`
- `GRAY4 :: 4`
- `LCD   :: 5`
- `LCD_V :: 6`
- `BGRA  :: 7`
- `MAX   :: 8`
- `FT_PIXEL_MODE_NONE  :: NONE`
- `FT_PIXEL_MODE_MONO  :: MONO`
- `FT_PIXEL_MODE_GRAY  :: GRAY`
- `FT_PIXEL_MODE_GRAY2 :: GRAY2`
- `FT_PIXEL_MODE_GRAY4 :: GRAY4`
- `FT_PIXEL_MODE_LCD   :: LCD`
- `FT_PIXEL_MODE_LCD_V :: LCD_V`
- `FT_PIXEL_MODE_BGRA  :: BGRA`
- `FT_PIXEL_MODE_MAX   :: MAX`

#### `FT_Bitmap_` (struct)

- `rows:         u32`
- `width:        u32`
- `pitch:        s32`
- `buffer:       *u8`
- `num_grays:    u16`
- `pixel_mode:   u8`
- `palette_mode: u8`
- `palette:      *void`

#### `FT_Outline_` (struct)

- `n_contours: s16`
- `n_points:   s16`
- `points:     *FT_Vector`
- `tags:       *u8`
- `contours:   *s16`
- `flags:      s32`

#### `FT_Outline_Funcs_` (struct)

- `move_to:  FT_Outline_MoveToFunc`
- `line_to:  FT_Outline_LineToFunc`
- `conic_to: FT_Outline_ConicToFunc`
- `cubic_to: FT_Outline_CubicToFunc`
- `shift:    s32`
- `delta:    FT_Pos`

#### `FT_Glyph_Format_` (enum)

- `NONE      :: 0`
- `COMPOSITE :: 1668246896`
- `BITMAP    :: 1651078259`
- `OUTLINE   :: 1869968492`
- `PLOTTER   :: 1886154612`
- `SVG       :: 1398163232`
- `FT_GLYPH_FORMAT_NONE      :: NONE`
- `FT_GLYPH_FORMAT_COMPOSITE :: COMPOSITE`
- `FT_GLYPH_FORMAT_BITMAP    :: BITMAP`
- `FT_GLYPH_FORMAT_OUTLINE   :: OUTLINE`
- `FT_GLYPH_FORMAT_PLOTTER   :: PLOTTER`
- `FT_GLYPH_FORMAT_SVG       :: SVG`

#### `FT_Span_` (struct)

- `x:        s16`
- `len:      u16`
- `coverage: u8`

#### `FT_Raster_Params_` (struct)

- `target:      *FT_Bitmap`
- `source:      *void`
- `flags:       s32`
- `gray_spans:  FT_SpanFunc`
- `black_spans: FT_SpanFunc`
- `bit_test:    FT_Raster_BitTest_Func`
- `bit_set:     FT_Raster_BitSet_Func`
- `user:        *void`
- `clip_box:    FT_BBox`

#### `FT_Raster_Funcs_` (struct)

- `glyph_format:    FT_Glyph_Format`
- `raster_new:      FT_Raster_NewFunc`
- `raster_reset:    FT_Raster_ResetFunc`
- `raster_set_mode: FT_Raster_SetModeFunc`
- `raster_render:   FT_Raster_RenderFunc`
- `raster_done:     FT_Raster_DoneFunc`

#### `FT_UnitVector_` (struct)

- `x: FT_F2Dot14`
- `y: FT_F2Dot14`

#### `FT_Matrix_` (struct)

- `_xx: FT_Fixed`
- `xy:  FT_Fixed`
- `yx:  FT_Fixed`
- `yy:  FT_Fixed`

#### `FT_Data_` (struct)

- `pointer: *FT_Byte`
- `length:  FT_UInt`

#### `FT_Generic_` (struct)

- `data:      *void`
- `finalizer: FT_Generic_Finalizer`

#### `FT_ListNodeRec_` (struct)

- `prev: FT_ListNode`
- `next: FT_ListNode`
- `data: *void`

#### `FT_ListRec_` (struct)

- `head: FT_ListNode`
- `tail: FT_ListNode`

#### `FT_Mod_Err` (enum)

- `Base     :: 0`
- `Autofit  :: 0`
- `BDF      :: 0`
- `Bzip2    :: 0`
- `Cache    :: 0`
- `CFF      :: 0`
- `CID      :: 0`
- `Gzip     :: 0`
- `LZW      :: 0`
- `OTvalid  :: 0`
- `PCF      :: 0`
- `PFR      :: 0`
- `PSaux    :: 0`
- `PShinter :: 0`
- `PSnames  :: 0`
- `Raster   :: 0`
- `SFNT     :: 0`
- `Smooth   :: 0`
- `TrueType :: 0`
- `Type1    :: 0`
- `Type42   :: 0`
- `Winfonts :: 0`
- `GXvalid  :: 0`
- `Sdf      :: 0`
- `Max      :: 1`
- `FT_Mod_Err_Base     :: Base`
- `FT_Mod_Err_Autofit  :: Autofit`
- `FT_Mod_Err_BDF      :: BDF`
- `FT_Mod_Err_Bzip2    :: Bzip2`
- `FT_Mod_Err_Cache    :: Cache`
- `FT_Mod_Err_CFF      :: CFF`
- `FT_Mod_Err_CID      :: CID`
- `FT_Mod_Err_Gzip     :: Gzip`
- `FT_Mod_Err_LZW      :: LZW`
- `FT_Mod_Err_OTvalid  :: OTvalid`
- `FT_Mod_Err_PCF      :: PCF`
- `FT_Mod_Err_PFR      :: PFR`
- `FT_Mod_Err_PSaux    :: PSaux`
- `FT_Mod_Err_PShinter :: PShinter`
- `FT_Mod_Err_PSnames  :: PSnames`
- `FT_Mod_Err_Raster   :: Raster`
- `FT_Mod_Err_SFNT     :: SFNT`
- `FT_Mod_Err_Smooth   :: Smooth`
- `FT_Mod_Err_TrueType :: TrueType`
- `FT_Mod_Err_Type1    :: Type1`
- `FT_Mod_Err_Type42   :: Type42`
- `FT_Mod_Err_Winfonts :: Winfonts`
- `FT_Mod_Err_GXvalid  :: GXvalid`
- `FT_Mod_Err_Sdf      :: Sdf`
- `FT_Mod_Err_Max      :: Max`

#### `FT_Err` (enum)

- `Ok                            :: 0`
- `Cannot_Open_Resource          :: 1`
- `Unknown_File_Format           :: 2`
- `Invalid_File_Format           :: 3`
- `Invalid_Version               :: 4`
- `Lower_Module_Version          :: 5`
- `Invalid_Argument              :: 6`
- `Unimplemented_Feature         :: 7`
- `Invalid_Table                 :: 8`
- `Invalid_Offset                :: 9`
- `Array_Too_Large               :: 10`
- `Missing_Module                :: 11`
- `Missing_Property              :: 12`
- `Invalid_Glyph_Index           :: 16`
- `Invalid_Character_Code        :: 17`
- `Invalid_Glyph_Format          :: 18`
- `Cannot_Render_Glyph           :: 19`
- `Invalid_Outline               :: 20`
- `Invalid_Composite             :: 21`
- `Too_Many_Hints                :: 22`
- `Invalid_Pixel_Size            :: 23`
- `Invalid_SVG_Document          :: 24`
- `Invalid_Handle                :: 32`
- `Invalid_Library_Handle        :: 33`
- `Invalid_Driver_Handle         :: 34`
- `Invalid_Face_Handle           :: 35`
- `Invalid_Size_Handle           :: 36`
- `Invalid_Slot_Handle           :: 37`
- `Invalid_CharMap_Handle        :: 38`
- `Invalid_Cache_Handle          :: 39`
- `Invalid_Stream_Handle         :: 40`
- `Too_Many_Drivers              :: 48`
- `Too_Many_Extensions           :: 49`
- `Out_Of_Memory                 :: 64`
- `Unlisted_Object               :: 65`
- `Cannot_Open_Stream            :: 81`
- `Invalid_Stream_Seek           :: 82`
- `Invalid_Stream_Skip           :: 83`
- `Invalid_Stream_Read           :: 84`
- `Invalid_Stream_Operation      :: 85`
- `Invalid_Frame_Operation       :: 86`
- `Nested_Frame_Access           :: 87`
- `Invalid_Frame_Read            :: 88`
- `Raster_Uninitialized          :: 96`
- `Raster_Corrupted              :: 97`
- `Raster_Overflow               :: 98`
- `Raster_Negative_Height        :: 99`
- `Too_Many_Caches               :: 112`
- `Invalid_Opcode                :: 128`
- `Too_Few_Arguments             :: 129`
- `Stack_Overflow                :: 130`
- `Code_Overflow                 :: 131`
- `Bad_Argument                  :: 132`
- `Divide_By_Zero                :: 133`
- `Invalid_Reference             :: 134`
- `Debug_OpCode                  :: 135`
- `ENDF_In_Exec_Stream           :: 136`
- `Nested_DEFS                   :: 137`
- `Invalid_CodeRange             :: 138`
- `Execution_Too_Long            :: 139`
- `Too_Many_Function_Defs        :: 140`
- `Too_Many_Instruction_Defs     :: 141`
- `Table_Missing                 :: 142`
- `Horiz_Header_Missing          :: 143`
- `Locations_Missing             :: 144`
- `Name_Table_Missing            :: 145`
- `CMap_Table_Missing            :: 146`
- `Hmtx_Table_Missing            :: 147`
- `Post_Table_Missing            :: 148`
- `Invalid_Horiz_Metrics         :: 149`
- `Invalid_CharMap_Format        :: 150`
- `Invalid_PPem                  :: 151`
- `Invalid_Vert_Metrics          :: 152`
- `Could_Not_Find_Context        :: 153`
- `Invalid_Post_Table_Format     :: 154`
- `Invalid_Post_Table            :: 155`
- `DEF_In_Glyf_Bytecode          :: 156`
- `Missing_Bitmap                :: 157`
- `Missing_SVG_Hooks             :: 158`
- `Syntax_Error                  :: 160`
- `Stack_Underflow               :: 161`
- `Ignore                        :: 162`
- `No_Unicode_Glyph_Name         :: 163`
- `Glyph_Too_Big                 :: 164`
- `Missing_Startfont_Field       :: 176`
- `Missing_Font_Field            :: 177`
- `Missing_Size_Field            :: 178`
- `Missing_Fontboundingbox_Field :: 179`
- `Missing_Chars_Field           :: 180`
- `Missing_Startchar_Field       :: 181`
- `Missing_Encoding_Field        :: 182`
- `Missing_Bbx_Field             :: 183`
- `Bbx_Too_Big                   :: 184`
- `Corrupted_Font_Header         :: 185`
- `Corrupted_Font_Glyphs         :: 186`
- `Max                           :: 187`
- `FT_Err_Ok                            :: Ok`
- `FT_Err_Cannot_Open_Resource          :: Cannot_Open_Resource`
- `FT_Err_Unknown_File_Format           :: Unknown_File_Format`
- `FT_Err_Invalid_File_Format           :: Invalid_File_Format`
- `FT_Err_Invalid_Version               :: Invalid_Version`
- `FT_Err_Lower_Module_Version          :: Lower_Module_Version`
- `FT_Err_Invalid_Argument              :: Invalid_Argument`
- `FT_Err_Unimplemented_Feature         :: Unimplemented_Feature`
- `FT_Err_Invalid_Table                 :: Invalid_Table`
- `FT_Err_Invalid_Offset                :: Invalid_Offset`
- `FT_Err_Array_Too_Large               :: Array_Too_Large`
- `FT_Err_Missing_Module                :: Missing_Module`
- `FT_Err_Missing_Property              :: Missing_Property`
- `FT_Err_Invalid_Glyph_Index           :: Invalid_Glyph_Index`
- `FT_Err_Invalid_Character_Code        :: Invalid_Character_Code`
- `FT_Err_Invalid_Glyph_Format          :: Invalid_Glyph_Format`
- `FT_Err_Cannot_Render_Glyph           :: Cannot_Render_Glyph`
- `FT_Err_Invalid_Outline               :: Invalid_Outline`
- `FT_Err_Invalid_Composite             :: Invalid_Composite`
- `FT_Err_Too_Many_Hints                :: Too_Many_Hints`
- `FT_Err_Invalid_Pixel_Size            :: Invalid_Pixel_Size`
- `FT_Err_Invalid_SVG_Document          :: Invalid_SVG_Document`
- `FT_Err_Invalid_Handle                :: Invalid_Handle`
- `FT_Err_Invalid_Library_Handle        :: Invalid_Library_Handle`
- `FT_Err_Invalid_Driver_Handle         :: Invalid_Driver_Handle`
- `FT_Err_Invalid_Face_Handle           :: Invalid_Face_Handle`
- `FT_Err_Invalid_Size_Handle           :: Invalid_Size_Handle`
- `FT_Err_Invalid_Slot_Handle           :: Invalid_Slot_Handle`
- `FT_Err_Invalid_CharMap_Handle        :: Invalid_CharMap_Handle`
- `FT_Err_Invalid_Cache_Handle          :: Invalid_Cache_Handle`
- `FT_Err_Invalid_Stream_Handle         :: Invalid_Stream_Handle`
- `FT_Err_Too_Many_Drivers              :: Too_Many_Drivers`
- `FT_Err_Too_Many_Extensions           :: Too_Many_Extensions`
- `FT_Err_Out_Of_Memory                 :: Out_Of_Memory`
- `FT_Err_Unlisted_Object               :: Unlisted_Object`
- `FT_Err_Cannot_Open_Stream            :: Cannot_Open_Stream`
- `FT_Err_Invalid_Stream_Seek           :: Invalid_Stream_Seek`
- `FT_Err_Invalid_Stream_Skip           :: Invalid_Stream_Skip`
- `FT_Err_Invalid_Stream_Read           :: Invalid_Stream_Read`
- `FT_Err_Invalid_Stream_Operation      :: Invalid_Stream_Operation`
- `FT_Err_Invalid_Frame_Operation       :: Invalid_Frame_Operation`
- `FT_Err_Nested_Frame_Access           :: Nested_Frame_Access`
- `FT_Err_Invalid_Frame_Read            :: Invalid_Frame_Read`
- `FT_Err_Raster_Uninitialized          :: Raster_Uninitialized`
- `FT_Err_Raster_Corrupted              :: Raster_Corrupted`
- `FT_Err_Raster_Overflow               :: Raster_Overflow`
- `FT_Err_Raster_Negative_Height        :: Raster_Negative_Height`
- `FT_Err_Too_Many_Caches               :: Too_Many_Caches`
- `FT_Err_Invalid_Opcode                :: Invalid_Opcode`
- `FT_Err_Too_Few_Arguments             :: Too_Few_Arguments`
- `FT_Err_Stack_Overflow                :: Stack_Overflow`
- `FT_Err_Code_Overflow                 :: Code_Overflow`
- `FT_Err_Bad_Argument                  :: Bad_Argument`
- `FT_Err_Divide_By_Zero                :: Divide_By_Zero`
- `FT_Err_Invalid_Reference             :: Invalid_Reference`
- `FT_Err_Debug_OpCode                  :: Debug_OpCode`
- `FT_Err_ENDF_In_Exec_Stream           :: ENDF_In_Exec_Stream`
- `FT_Err_Nested_DEFS                   :: Nested_DEFS`
- `FT_Err_Invalid_CodeRange             :: Invalid_CodeRange`
- `FT_Err_Execution_Too_Long            :: Execution_Too_Long`
- `FT_Err_Too_Many_Function_Defs        :: Too_Many_Function_Defs`
- `FT_Err_Too_Many_Instruction_Defs     :: Too_Many_Instruction_Defs`
- `FT_Err_Table_Missing                 :: Table_Missing`
- `FT_Err_Horiz_Header_Missing          :: Horiz_Header_Missing`
- `FT_Err_Locations_Missing             :: Locations_Missing`
- `FT_Err_Name_Table_Missing            :: Name_Table_Missing`
- `FT_Err_CMap_Table_Missing            :: CMap_Table_Missing`
- `FT_Err_Hmtx_Table_Missing            :: Hmtx_Table_Missing`
- `FT_Err_Post_Table_Missing            :: Post_Table_Missing`
- `FT_Err_Invalid_Horiz_Metrics         :: Invalid_Horiz_Metrics`
- `FT_Err_Invalid_CharMap_Format        :: Invalid_CharMap_Format`
- `FT_Err_Invalid_PPem                  :: Invalid_PPem`
- `FT_Err_Invalid_Vert_Metrics          :: Invalid_Vert_Metrics`
- `FT_Err_Could_Not_Find_Context        :: Could_Not_Find_Context`
- `FT_Err_Invalid_Post_Table_Format     :: Invalid_Post_Table_Format`
- `FT_Err_Invalid_Post_Table            :: Invalid_Post_Table`
- `FT_Err_DEF_In_Glyf_Bytecode          :: DEF_In_Glyf_Bytecode`
- `FT_Err_Missing_Bitmap                :: Missing_Bitmap`
- `FT_Err_Missing_SVG_Hooks             :: Missing_SVG_Hooks`
- `FT_Err_Syntax_Error                  :: Syntax_Error`
- `FT_Err_Stack_Underflow               :: Stack_Underflow`
- `FT_Err_Ignore                        :: Ignore`
- `FT_Err_No_Unicode_Glyph_Name         :: No_Unicode_Glyph_Name`
- `FT_Err_Glyph_Too_Big                 :: Glyph_Too_Big`
- `FT_Err_Missing_Startfont_Field       :: Missing_Startfont_Field`
- `FT_Err_Missing_Font_Field            :: Missing_Font_Field`
- `FT_Err_Missing_Size_Field            :: Missing_Size_Field`
- `FT_Err_Missing_Fontboundingbox_Field :: Missing_Fontboundingbox_Field`
- `FT_Err_Missing_Chars_Field           :: Missing_Chars_Field`
- `FT_Err_Missing_Startchar_Field       :: Missing_Startchar_Field`
- `FT_Err_Missing_Encoding_Field        :: Missing_Encoding_Field`
- `FT_Err_Missing_Bbx_Field             :: Missing_Bbx_Field`
- `FT_Err_Bbx_Too_Big                   :: Bbx_Too_Big`
- `FT_Err_Corrupted_Font_Header         :: Corrupted_Font_Header`
- `FT_Err_Corrupted_Font_Glyphs         :: Corrupted_Font_Glyphs`
- `FT_Err_Max                           :: Max`

#### `FT_Glyph_Metrics_` (struct)

- `width:        FT_Pos`
- `height:       FT_Pos`
- `horiBearingX: FT_Pos`
- `horiBearingY: FT_Pos`
- `horiAdvance:  FT_Pos`
- `vertBearingX: FT_Pos`
- `vertBearingY: FT_Pos`
- `vertAdvance:  FT_Pos`

#### `FT_Bitmap_Size_` (struct)

- `height: FT_Short`
- `width:  FT_Short`
- `size:   FT_Pos`
- `x_ppem: FT_Pos`
- `y_ppem: FT_Pos`

#### `FT_Encoding_` (enum)

- `NONE           :: 0`
- `MS_SYMBOL      :: 1937337698`
- `UNICODE        :: 1970170211`
- `SJIS           :: 1936353651`
- `PRC            :: 1734484000`
- `BIG5           :: 1651074869`
- `WANSUNG        :: 2002873971`
- `JOHAB          :: 1785686113`
- `GB2312         :: 1734484000`
- `MS_SJIS        :: 1936353651`
- `MS_GB2312      :: 1734484000`
- `MS_BIG5        :: 1651074869`
- `MS_WANSUNG     :: 2002873971`
- `MS_JOHAB       :: 1785686113`
- `ADOBE_STANDARD :: 1094995778`
- `ADOBE_EXPERT   :: 1094992453`
- `ADOBE_CUSTOM   :: 1094992451`
- `ADOBE_LATIN_1  :: 1818326065`
- `OLD_LATIN_2    :: 1818326066`
- `APPLE_ROMAN    :: 1634889070`
- `FT_ENCODING_NONE           :: NONE`
- `FT_ENCODING_MS_SYMBOL      :: MS_SYMBOL`
- `FT_ENCODING_UNICODE        :: UNICODE`
- `FT_ENCODING_SJIS           :: SJIS`
- `FT_ENCODING_PRC            :: PRC`
- `FT_ENCODING_BIG5           :: BIG5`
- `FT_ENCODING_WANSUNG        :: WANSUNG`
- `FT_ENCODING_JOHAB          :: JOHAB`
- `FT_ENCODING_GB2312         :: GB2312`
- `FT_ENCODING_MS_SJIS        :: MS_SJIS`
- `FT_ENCODING_MS_GB2312      :: MS_GB2312`
- `FT_ENCODING_MS_BIG5        :: MS_BIG5`
- `FT_ENCODING_MS_WANSUNG     :: MS_WANSUNG`
- `FT_ENCODING_MS_JOHAB       :: MS_JOHAB`
- `FT_ENCODING_ADOBE_STANDARD :: ADOBE_STANDARD`
- `FT_ENCODING_ADOBE_EXPERT   :: ADOBE_EXPERT`
- `FT_ENCODING_ADOBE_CUSTOM   :: ADOBE_CUSTOM`
- `FT_ENCODING_ADOBE_LATIN_1  :: ADOBE_LATIN_1`
- `FT_ENCODING_OLD_LATIN_2    :: OLD_LATIN_2`
- `FT_ENCODING_APPLE_ROMAN    :: APPLE_ROMAN`

#### `FT_CharMapRec_` (struct)

- `face:        FT_Face`
- `encoding:    FT_Encoding`
- `platform_id: FT_UShort`
- `encoding_id: FT_UShort`

#### `FT_FaceRec_` (struct)

- `num_faces:           FT_Long`
- `face_index:          FT_Long`
- `face_flags:          FT_Long`
- `style_flags:         FT_Long`
- `num_glyphs:          FT_Long`
- `family_name:         *FT_String`
- `style_name:          *FT_String`
- `num_fixed_sizes:     FT_Int`
- `available_sizes:     *FT_Bitmap_Size`
- `num_charmaps:        FT_Int`
- `charmaps:            *FT_CharMap`
- `generic:             FT_Generic`
- `bbox:                FT_BBox`
- `units_per_EM:        FT_UShort`
- `ascender:            FT_Short`
- `descender:           FT_Short`
- `height:              FT_Short`
- `max_advance_width:   FT_Short`
- `max_advance_height:  FT_Short`
- `underline_position:  FT_Short`
- `underline_thickness: FT_Short`
- `glyph:               FT_GlyphSlot`
- `size:                FT_Size`
- `charmap:             FT_CharMap`
- `driver:              FT_Driver`
- `memory:              FT_Memory`
- `stream:              FT_Stream`
- `sizes_list:          FT_ListRec`
- `autohint:            FT_Generic`
- `extensions:          *void`
- `internal:            FT_Face_Internal`

#### `FT_Size_Metrics_` (struct)

- `x_ppem:      FT_UShort`
- `y_ppem:      FT_UShort`
- `x_scale:     FT_Fixed`
- `y_scale:     FT_Fixed`
- `ascender:    FT_Pos`
- `descender:   FT_Pos`
- `height:      FT_Pos`
- `max_advance: FT_Pos`

#### `FT_SizeRec_` (struct)

- `face:     FT_Face`
- `generic:  FT_Generic`
- `metrics:  FT_Size_Metrics`
- `internal: FT_Size_Internal`

#### `FT_GlyphSlotRec_` (struct)

- `library:           FT_Library`
- `face:              FT_Face`
- `next:              FT_GlyphSlot`
- `glyph_index:       FT_UInt`
- `generic:           FT_Generic`
- `metrics:           FT_Glyph_Metrics`
- `linearHoriAdvance: FT_Fixed`
- `linearVertAdvance: FT_Fixed`
- `advance:           FT_Vector`
- `format:            FT_Glyph_Format`
- `bitmap:            FT_Bitmap`
- `bitmap_left:       FT_Int`
- `bitmap_top:        FT_Int`
- `outline:           FT_Outline`
- `num_subglyphs:     FT_UInt`
- `subglyphs:         FT_SubGlyph`
- `control_data:      *void`
- `control_len:       s64`
- `lsb_delta:         FT_Pos`
- `rsb_delta:         FT_Pos`
- `other:             *void`
- `internal:          FT_Slot_Internal`

#### `FT_Parameter_` (struct)

- `tag:  FT_ULong`
- `data: FT_Pointer`

#### `FT_Open_Args_` (struct)

- `flags:       FT_UInt`
- `memory_base: *FT_Byte`
- `memory_size: FT_Long`
- `pathname:    *FT_String`
- `stream:      FT_Stream`
- `driver:      FT_Module`
- `num_params:  FT_Int`
- `params:      *FT_Parameter`

#### `FT_Size_Request_Type_` (enum)

- `NOMINAL  :: 0`
- `REAL_DIM :: 1`
- `BBOX     :: 2`
- `CELL     :: 3`
- `SCALES   :: 4`
- `MAX      :: 5`
- `FT_SIZE_REQUEST_TYPE_NOMINAL  :: NOMINAL`
- `FT_SIZE_REQUEST_TYPE_REAL_DIM :: REAL_DIM`
- `FT_SIZE_REQUEST_TYPE_BBOX     :: BBOX`
- `FT_SIZE_REQUEST_TYPE_CELL     :: CELL`
- `FT_SIZE_REQUEST_TYPE_SCALES   :: SCALES`
- `FT_SIZE_REQUEST_TYPE_MAX      :: MAX`

#### `FT_Size_RequestRec_` (struct)

- `type:           FT_Size_Request_Type`
- `width:          FT_Long`
- `height:         FT_Long`
- `horiResolution: FT_UInt`
- `vertResolution: FT_UInt`

#### `FT_Render_Mode_` (enum)

- `NORMAL :: 0`
- `LIGHT  :: 1`
- `MONO   :: 2`
- `LCD    :: 3`
- `LCD_V  :: 4`
- `SDF    :: 5`
- `MAX    :: 6`
- `FT_RENDER_MODE_NORMAL :: NORMAL`
- `FT_RENDER_MODE_LIGHT  :: LIGHT`
- `FT_RENDER_MODE_MONO   :: MONO`
- `FT_RENDER_MODE_LCD    :: LCD`
- `FT_RENDER_MODE_LCD_V  :: LCD_V`
- `FT_RENDER_MODE_SDF    :: SDF`
- `FT_RENDER_MODE_MAX    :: MAX`

#### `FT_Kerning_Mode_` (enum)

- `DEFAULT  :: 0`
- `UNFITTED :: 1`
- `UNSCALED :: 2`
- `FT_KERNING_DEFAULT  :: DEFAULT`
- `FT_KERNING_UNFITTED :: UNFITTED`
- `FT_KERNING_UNSCALED :: UNSCALED`

#### `TT_Header_` (struct)

- `Table_Version:       FT_Fixed`
- `Font_Revision:       FT_Fixed`
- `CheckSum_Adjust:     FT_Long`
- `Magic_Number:        FT_Long`
- `Flags:               FT_UShort`
- `Units_Per_EM:        FT_UShort`
- `Created:             [2] FT_ULong`
- `Modified:            [2] FT_ULong`
- `xMin:                FT_Short`
- `yMin:                FT_Short`
- `xMax:                FT_Short`
- `yMax:                FT_Short`
- `Mac_Style:           FT_UShort`
- `Lowest_Rec_PPEM:     FT_UShort`
- `Font_Direction:      FT_Short`
- `Index_To_Loc_Format: FT_Short`
- `Glyph_Data_Format:   FT_Short`

#### `TT_HoriHeader_` (struct)

- `Version:                FT_Fixed`
- `Ascender:               FT_Short`
- `Descender:              FT_Short`
- `Line_Gap:               FT_Short`
- `advance_Width_Max:      FT_UShort`
- `min_Left_Side_Bearing:  FT_Short`
- `min_Right_Side_Bearing: FT_Short`
- `xMax_Extent:            FT_Short`
- `caret_Slope_Rise:       FT_Short`
- `caret_Slope_Run:        FT_Short`
- `caret_Offset:           FT_Short`
- `Reserved:               [4] FT_Short`
- `metric_Data_Format:     FT_Short`
- `number_Of_HMetrics:     FT_UShort`
- `long_metrics:           *void`
- `short_metrics:          *void`

#### `TT_VertHeader_` (struct)

- `Version:                 FT_Fixed`
- `Ascender:                FT_Short`
- `Descender:               FT_Short`
- `Line_Gap:                FT_Short`
- `advance_Height_Max:      FT_UShort`
- `min_Top_Side_Bearing:    FT_Short`
- `min_Bottom_Side_Bearing: FT_Short`
- `yMax_Extent:             FT_Short`
- `caret_Slope_Rise:        FT_Short`
- `caret_Slope_Run:         FT_Short`
- `caret_Offset:            FT_Short`
- `Reserved:                [4] FT_Short`
- `metric_Data_Format:      FT_Short`
- `number_Of_VMetrics:      FT_UShort`
- `long_metrics:            *void`
- `short_metrics:           *void`

#### `TT_OS2_` (struct)

- `version:                 FT_UShort`
- `xAvgCharWidth:           FT_Short`
- `usWeightClass:           FT_UShort`
- `usWidthClass:            FT_UShort`
- `fsType:                  FT_UShort`
- `ySubscriptXSize:         FT_Short`
- `ySubscriptYSize:         FT_Short`
- `ySubscriptXOffset:       FT_Short`
- `ySubscriptYOffset:       FT_Short`
- `ySuperscriptXSize:       FT_Short`
- `ySuperscriptYSize:       FT_Short`
- `ySuperscriptXOffset:     FT_Short`
- `ySuperscriptYOffset:     FT_Short`
- `yStrikeoutSize:          FT_Short`
- `yStrikeoutPosition:      FT_Short`
- `sFamilyClass:            FT_Short`
- `panose:                  [10] FT_Byte`
- `ulUnicodeRange1:         FT_ULong`
- `ulUnicodeRange2:         FT_ULong`
- `ulUnicodeRange3:         FT_ULong`
- `ulUnicodeRange4:         FT_ULong`
- `achVendID:               [4] FT_Char`
- `fsSelection:             FT_UShort`
- `usFirstCharIndex:        FT_UShort`
- `usLastCharIndex:         FT_UShort`
- `sTypoAscender:           FT_Short`
- `sTypoDescender:          FT_Short`
- `sTypoLineGap:            FT_Short`
- `usWinAscent:             FT_UShort`
- `usWinDescent:            FT_UShort`
- `ulCodePageRange1:        FT_ULong`
- `ulCodePageRange2:        FT_ULong`
- `sxHeight:                FT_Short`
- `sCapHeight:              FT_Short`
- `usDefaultChar:           FT_UShort`
- `usBreakChar:             FT_UShort`
- `usMaxContext:            FT_UShort`
- `usLowerOpticalPointSize: FT_UShort`
- `usUpperOpticalPointSize: FT_UShort`

#### `TT_Postscript_` (struct)

- `FormatType:         FT_Fixed`
- `italicAngle:        FT_Fixed`
- `underlinePosition:  FT_Short`
- `underlineThickness: FT_Short`
- `isFixedPitch:       FT_ULong`
- `minMemType42:       FT_ULong`
- `maxMemType42:       FT_ULong`
- `minMemType1:        FT_ULong`
- `maxMemType1:        FT_ULong`

#### `TT_PCLT_` (struct)

- `Version:             FT_Fixed`
- `FontNumber:          FT_ULong`
- `Pitch:               FT_UShort`
- `xHeight:             FT_UShort`
- `Style:               FT_UShort`
- `TypeFamily:          FT_UShort`
- `CapHeight:           FT_UShort`
- `SymbolSet:           FT_UShort`
- `TypeFace:            [16] FT_Char`
- `CharacterComplement: [8] FT_Char`
- `FileName:            [6] FT_Char`
- `StrokeWeight:        FT_Char`
- `WidthType:           FT_Char`
- `SerifStyle:          FT_Byte`
- `Reserved:            FT_Byte`

#### `TT_MaxProfile_` (struct)

- `version:               FT_Fixed`
- `numGlyphs:             FT_UShort`
- `maxPoints:             FT_UShort`
- `maxContours:           FT_UShort`
- `maxCompositePoints:    FT_UShort`
- `maxCompositeContours:  FT_UShort`
- `maxZones:              FT_UShort`
- `maxTwilightPoints:     FT_UShort`
- `maxStorage:            FT_UShort`
- `maxFunctionDefs:       FT_UShort`
- `maxInstructionDefs:    FT_UShort`
- `maxStackElements:      FT_UShort`
- `maxSizeOfInstructions: FT_UShort`
- `maxComponentElements:  FT_UShort`
- `maxComponentDepth:     FT_UShort`

#### `FT_Sfnt_Tag_` (enum)

- `HEAD :: 0`
- `MAXP :: 1`
- `OS2  :: 2`
- `HHEA :: 3`
- `VHEA :: 4`
- `POST :: 5`
- `PCLT :: 6`
- `MAX  :: 7`
- `FT_SFNT_HEAD :: HEAD`
- `FT_SFNT_MAXP :: MAXP`
- `FT_SFNT_OS2  :: OS2`
- `FT_SFNT_HHEA :: HHEA`
- `FT_SFNT_VHEA :: VHEA`
- `FT_SFNT_POST :: POST`
- `FT_SFNT_PCLT :: PCLT`
- `FT_SFNT_MAX  :: MAX`

### Windows-specific type declarations

#### `FT_MemoryRec_` (struct)

- `user:    *void`
- `alloc:   FT_Alloc_Func`
- `free:    FT_Free_Func`
- `realloc: FT_Realloc_Func`

#### `FT_StreamDesc_` (struct)

- `pointer: *void`

#### `FT_StreamRec_` (struct)

- `base:       *u8`
- `size:       u32`
- `pos:        u32`
- `descriptor: FT_StreamDesc`
- `pathname:   FT_StreamDesc`
- `read:       FT_Stream_IoFunc`
- `close:      FT_Stream_CloseFunc`
- `memory:     FT_Memory`
- `cursor:     *u8`
- `limit:      *u8`

#### `FT_Vector_` (struct)

- `x: FT_Pos`
- `y: FT_Pos`

#### `FT_BBox_` (struct)

- `xMin: FT_Pos`
- `yMin: FT_Pos`
- `xMax: FT_Pos`
- `yMax: FT_Pos`

#### `FT_Pixel_Mode_` (enum)

- `NONE  :: 0`
- `MONO  :: 1`
- `GRAY  :: 2`
- `GRAY2 :: 3`
- `GRAY4 :: 4`
- `LCD   :: 5`
- `LCD_V :: 6`
- `BGRA  :: 7`
- `MAX   :: 8`
- `FT_PIXEL_MODE_NONE  :: NONE`
- `FT_PIXEL_MODE_MONO  :: MONO`
- `FT_PIXEL_MODE_GRAY  :: GRAY`
- `FT_PIXEL_MODE_GRAY2 :: GRAY2`
- `FT_PIXEL_MODE_GRAY4 :: GRAY4`
- `FT_PIXEL_MODE_LCD   :: LCD`
- `FT_PIXEL_MODE_LCD_V :: LCD_V`
- `FT_PIXEL_MODE_BGRA  :: BGRA`
- `FT_PIXEL_MODE_MAX   :: MAX`

#### `FT_Bitmap_` (struct)

- `rows:         u32`
- `width:        u32`
- `pitch:        s32`
- `buffer:       *u8`
- `num_grays:    u16`
- `pixel_mode:   u8`
- `palette_mode: u8`
- `palette:      *void`

#### `FT_Outline_` (struct)

- `n_contours: s16`
- `n_points:   s16`
- `points:     *FT_Vector`
- `tags:       *u8`
- `contours:   *s16`
- `flags:      s32`

#### `FT_Outline_Funcs_` (struct)

- `move_to:  FT_Outline_MoveToFunc`
- `line_to:  FT_Outline_LineToFunc`
- `conic_to: FT_Outline_ConicToFunc`
- `cubic_to: FT_Outline_CubicToFunc`
- `shift:    s32`
- `delta:    FT_Pos`

#### `FT_Glyph_Format_` (enum)

- `NONE      :: 0`
- `COMPOSITE :: 1668246896`
- `BITMAP    :: 1651078259`
- `OUTLINE   :: 1869968492`
- `PLOTTER   :: 1886154612`
- `SVG       :: 1398163232`
- `FT_GLYPH_FORMAT_NONE      :: NONE`
- `FT_GLYPH_FORMAT_COMPOSITE :: COMPOSITE`
- `FT_GLYPH_FORMAT_BITMAP    :: BITMAP`
- `FT_GLYPH_FORMAT_OUTLINE   :: OUTLINE`
- `FT_GLYPH_FORMAT_PLOTTER   :: PLOTTER`
- `FT_GLYPH_FORMAT_SVG       :: SVG`

#### `FT_Span_` (struct)

- `x:        s16`
- `len:      u16`
- `coverage: u8`

#### `FT_Raster_Params_` (struct)

- `target:      *FT_Bitmap`
- `source:      *void`
- `flags:       s32`
- `gray_spans:  FT_SpanFunc`
- `black_spans: FT_SpanFunc`
- `bit_test:    FT_Raster_BitTest_Func`
- `bit_set:     FT_Raster_BitSet_Func`
- `user:        *void`
- `clip_box:    FT_BBox`

#### `FT_Raster_Funcs_` (struct)

- `glyph_format:    FT_Glyph_Format`
- `raster_new:      FT_Raster_NewFunc`
- `raster_reset:    FT_Raster_ResetFunc`
- `raster_set_mode: FT_Raster_SetModeFunc`
- `raster_render:   FT_Raster_RenderFunc`
- `raster_done:     FT_Raster_DoneFunc`

#### `FT_UnitVector_` (struct)

- `x: FT_F2Dot14`
- `y: FT_F2Dot14`

#### `FT_Matrix_` (struct)

- `_xx: FT_Fixed`
- `xy:  FT_Fixed`
- `yx:  FT_Fixed`
- `yy:  FT_Fixed`

#### `FT_Data_` (struct)

- `pointer: *FT_Byte`
- `length:  FT_UInt`

#### `FT_Generic_` (struct)

- `data:      *void`
- `finalizer: FT_Generic_Finalizer`

#### `FT_ListNodeRec_` (struct)

- `prev: FT_ListNode`
- `next: FT_ListNode`
- `data: *void`

#### `FT_ListRec_` (struct)

- `head: FT_ListNode`
- `tail: FT_ListNode`

#### `FT_Mod_Err` (enum)

- `Base     :: 0`
- `Autofit  :: 0`
- `BDF      :: 0`
- `Bzip2    :: 0`
- `Cache    :: 0`
- `CFF      :: 0`
- `CID      :: 0`
- `Gzip     :: 0`
- `LZW      :: 0`
- `OTvalid  :: 0`
- `PCF      :: 0`
- `PFR      :: 0`
- `PSaux    :: 0`
- `PShinter :: 0`
- `PSnames  :: 0`
- `Raster   :: 0`
- `SFNT     :: 0`
- `Smooth   :: 0`
- `TrueType :: 0`
- `Type1    :: 0`
- `Type42   :: 0`
- `Winfonts :: 0`
- `GXvalid  :: 0`
- `Sdf      :: 0`
- `Max      :: 1`
- `FT_Mod_Err_Base     :: Base`
- `FT_Mod_Err_Autofit  :: Autofit`
- `FT_Mod_Err_BDF      :: BDF`
- `FT_Mod_Err_Bzip2    :: Bzip2`
- `FT_Mod_Err_Cache    :: Cache`
- `FT_Mod_Err_CFF      :: CFF`
- `FT_Mod_Err_CID      :: CID`
- `FT_Mod_Err_Gzip     :: Gzip`
- `FT_Mod_Err_LZW      :: LZW`
- `FT_Mod_Err_OTvalid  :: OTvalid`
- `FT_Mod_Err_PCF      :: PCF`
- `FT_Mod_Err_PFR      :: PFR`
- `FT_Mod_Err_PSaux    :: PSaux`
- `FT_Mod_Err_PShinter :: PShinter`
- `FT_Mod_Err_PSnames  :: PSnames`
- `FT_Mod_Err_Raster   :: Raster`
- `FT_Mod_Err_SFNT     :: SFNT`
- `FT_Mod_Err_Smooth   :: Smooth`
- `FT_Mod_Err_TrueType :: TrueType`
- `FT_Mod_Err_Type1    :: Type1`
- `FT_Mod_Err_Type42   :: Type42`
- `FT_Mod_Err_Winfonts :: Winfonts`
- `FT_Mod_Err_GXvalid  :: GXvalid`
- `FT_Mod_Err_Sdf      :: Sdf`
- `FT_Mod_Err_Max      :: Max`

#### `FT_Err` (enum)

- `Ok                            :: 0`
- `Cannot_Open_Resource          :: 1`
- `Unknown_File_Format           :: 2`
- `Invalid_File_Format           :: 3`
- `Invalid_Version               :: 4`
- `Lower_Module_Version          :: 5`
- `Invalid_Argument              :: 6`
- `Unimplemented_Feature         :: 7`
- `Invalid_Table                 :: 8`
- `Invalid_Offset                :: 9`
- `Array_Too_Large               :: 10`
- `Missing_Module                :: 11`
- `Missing_Property              :: 12`
- `Invalid_Glyph_Index           :: 16`
- `Invalid_Character_Code        :: 17`
- `Invalid_Glyph_Format          :: 18`
- `Cannot_Render_Glyph           :: 19`
- `Invalid_Outline               :: 20`
- `Invalid_Composite             :: 21`
- `Too_Many_Hints                :: 22`
- `Invalid_Pixel_Size            :: 23`
- `Invalid_SVG_Document          :: 24`
- `Invalid_Handle                :: 32`
- `Invalid_Library_Handle        :: 33`
- `Invalid_Driver_Handle         :: 34`
- `Invalid_Face_Handle           :: 35`
- `Invalid_Size_Handle           :: 36`
- `Invalid_Slot_Handle           :: 37`
- `Invalid_CharMap_Handle        :: 38`
- `Invalid_Cache_Handle          :: 39`
- `Invalid_Stream_Handle         :: 40`
- `Too_Many_Drivers              :: 48`
- `Too_Many_Extensions           :: 49`
- `Out_Of_Memory                 :: 64`
- `Unlisted_Object               :: 65`
- `Cannot_Open_Stream            :: 81`
- `Invalid_Stream_Seek           :: 82`
- `Invalid_Stream_Skip           :: 83`
- `Invalid_Stream_Read           :: 84`
- `Invalid_Stream_Operation      :: 85`
- `Invalid_Frame_Operation       :: 86`
- `Nested_Frame_Access           :: 87`
- `Invalid_Frame_Read            :: 88`
- `Raster_Uninitialized          :: 96`
- `Raster_Corrupted              :: 97`
- `Raster_Overflow               :: 98`
- `Raster_Negative_Height        :: 99`
- `Too_Many_Caches               :: 112`
- `Invalid_Opcode                :: 128`
- `Too_Few_Arguments             :: 129`
- `Stack_Overflow                :: 130`
- `Code_Overflow                 :: 131`
- `Bad_Argument                  :: 132`
- `Divide_By_Zero                :: 133`
- `Invalid_Reference             :: 134`
- `Debug_OpCode                  :: 135`
- `ENDF_In_Exec_Stream           :: 136`
- `Nested_DEFS                   :: 137`
- `Invalid_CodeRange             :: 138`
- `Execution_Too_Long            :: 139`
- `Too_Many_Function_Defs        :: 140`
- `Too_Many_Instruction_Defs     :: 141`
- `Table_Missing                 :: 142`
- `Horiz_Header_Missing          :: 143`
- `Locations_Missing             :: 144`
- `Name_Table_Missing            :: 145`
- `CMap_Table_Missing            :: 146`
- `Hmtx_Table_Missing            :: 147`
- `Post_Table_Missing            :: 148`
- `Invalid_Horiz_Metrics         :: 149`
- `Invalid_CharMap_Format        :: 150`
- `Invalid_PPem                  :: 151`
- `Invalid_Vert_Metrics          :: 152`
- `Could_Not_Find_Context        :: 153`
- `Invalid_Post_Table_Format     :: 154`
- `Invalid_Post_Table            :: 155`
- `DEF_In_Glyf_Bytecode          :: 156`
- `Missing_Bitmap                :: 157`
- `Missing_SVG_Hooks             :: 158`
- `Syntax_Error                  :: 160`
- `Stack_Underflow               :: 161`
- `Ignore                        :: 162`
- `No_Unicode_Glyph_Name         :: 163`
- `Glyph_Too_Big                 :: 164`
- `Missing_Startfont_Field       :: 176`
- `Missing_Font_Field            :: 177`
- `Missing_Size_Field            :: 178`
- `Missing_Fontboundingbox_Field :: 179`
- `Missing_Chars_Field           :: 180`
- `Missing_Startchar_Field       :: 181`
- `Missing_Encoding_Field        :: 182`
- `Missing_Bbx_Field             :: 183`
- `Bbx_Too_Big                   :: 184`
- `Corrupted_Font_Header         :: 185`
- `Corrupted_Font_Glyphs         :: 186`
- `Max                           :: 187`
- `FT_Err_Ok                            :: Ok`
- `FT_Err_Cannot_Open_Resource          :: Cannot_Open_Resource`
- `FT_Err_Unknown_File_Format           :: Unknown_File_Format`
- `FT_Err_Invalid_File_Format           :: Invalid_File_Format`
- `FT_Err_Invalid_Version               :: Invalid_Version`
- `FT_Err_Lower_Module_Version          :: Lower_Module_Version`
- `FT_Err_Invalid_Argument              :: Invalid_Argument`
- `FT_Err_Unimplemented_Feature         :: Unimplemented_Feature`
- `FT_Err_Invalid_Table                 :: Invalid_Table`
- `FT_Err_Invalid_Offset                :: Invalid_Offset`
- `FT_Err_Array_Too_Large               :: Array_Too_Large`
- `FT_Err_Missing_Module                :: Missing_Module`
- `FT_Err_Missing_Property              :: Missing_Property`
- `FT_Err_Invalid_Glyph_Index           :: Invalid_Glyph_Index`
- `FT_Err_Invalid_Character_Code        :: Invalid_Character_Code`
- `FT_Err_Invalid_Glyph_Format          :: Invalid_Glyph_Format`
- `FT_Err_Cannot_Render_Glyph           :: Cannot_Render_Glyph`
- `FT_Err_Invalid_Outline               :: Invalid_Outline`
- `FT_Err_Invalid_Composite             :: Invalid_Composite`
- `FT_Err_Too_Many_Hints                :: Too_Many_Hints`
- `FT_Err_Invalid_Pixel_Size            :: Invalid_Pixel_Size`
- `FT_Err_Invalid_SVG_Document          :: Invalid_SVG_Document`
- `FT_Err_Invalid_Handle                :: Invalid_Handle`
- `FT_Err_Invalid_Library_Handle        :: Invalid_Library_Handle`
- `FT_Err_Invalid_Driver_Handle         :: Invalid_Driver_Handle`
- `FT_Err_Invalid_Face_Handle           :: Invalid_Face_Handle`
- `FT_Err_Invalid_Size_Handle           :: Invalid_Size_Handle`
- `FT_Err_Invalid_Slot_Handle           :: Invalid_Slot_Handle`
- `FT_Err_Invalid_CharMap_Handle        :: Invalid_CharMap_Handle`
- `FT_Err_Invalid_Cache_Handle          :: Invalid_Cache_Handle`
- `FT_Err_Invalid_Stream_Handle         :: Invalid_Stream_Handle`
- `FT_Err_Too_Many_Drivers              :: Too_Many_Drivers`
- `FT_Err_Too_Many_Extensions           :: Too_Many_Extensions`
- `FT_Err_Out_Of_Memory                 :: Out_Of_Memory`
- `FT_Err_Unlisted_Object               :: Unlisted_Object`
- `FT_Err_Cannot_Open_Stream            :: Cannot_Open_Stream`
- `FT_Err_Invalid_Stream_Seek           :: Invalid_Stream_Seek`
- `FT_Err_Invalid_Stream_Skip           :: Invalid_Stream_Skip`
- `FT_Err_Invalid_Stream_Read           :: Invalid_Stream_Read`
- `FT_Err_Invalid_Stream_Operation      :: Invalid_Stream_Operation`
- `FT_Err_Invalid_Frame_Operation       :: Invalid_Frame_Operation`
- `FT_Err_Nested_Frame_Access           :: Nested_Frame_Access`
- `FT_Err_Invalid_Frame_Read            :: Invalid_Frame_Read`
- `FT_Err_Raster_Uninitialized          :: Raster_Uninitialized`
- `FT_Err_Raster_Corrupted              :: Raster_Corrupted`
- `FT_Err_Raster_Overflow               :: Raster_Overflow`
- `FT_Err_Raster_Negative_Height        :: Raster_Negative_Height`
- `FT_Err_Too_Many_Caches               :: Too_Many_Caches`
- `FT_Err_Invalid_Opcode                :: Invalid_Opcode`
- `FT_Err_Too_Few_Arguments             :: Too_Few_Arguments`
- `FT_Err_Stack_Overflow                :: Stack_Overflow`
- `FT_Err_Code_Overflow                 :: Code_Overflow`
- `FT_Err_Bad_Argument                  :: Bad_Argument`
- `FT_Err_Divide_By_Zero                :: Divide_By_Zero`
- `FT_Err_Invalid_Reference             :: Invalid_Reference`
- `FT_Err_Debug_OpCode                  :: Debug_OpCode`
- `FT_Err_ENDF_In_Exec_Stream           :: ENDF_In_Exec_Stream`
- `FT_Err_Nested_DEFS                   :: Nested_DEFS`
- `FT_Err_Invalid_CodeRange             :: Invalid_CodeRange`
- `FT_Err_Execution_Too_Long            :: Execution_Too_Long`
- `FT_Err_Too_Many_Function_Defs        :: Too_Many_Function_Defs`
- `FT_Err_Too_Many_Instruction_Defs     :: Too_Many_Instruction_Defs`
- `FT_Err_Table_Missing                 :: Table_Missing`
- `FT_Err_Horiz_Header_Missing          :: Horiz_Header_Missing`
- `FT_Err_Locations_Missing             :: Locations_Missing`
- `FT_Err_Name_Table_Missing            :: Name_Table_Missing`
- `FT_Err_CMap_Table_Missing            :: CMap_Table_Missing`
- `FT_Err_Hmtx_Table_Missing            :: Hmtx_Table_Missing`
- `FT_Err_Post_Table_Missing            :: Post_Table_Missing`
- `FT_Err_Invalid_Horiz_Metrics         :: Invalid_Horiz_Metrics`
- `FT_Err_Invalid_CharMap_Format        :: Invalid_CharMap_Format`
- `FT_Err_Invalid_PPem                  :: Invalid_PPem`
- `FT_Err_Invalid_Vert_Metrics          :: Invalid_Vert_Metrics`
- `FT_Err_Could_Not_Find_Context        :: Could_Not_Find_Context`
- `FT_Err_Invalid_Post_Table_Format     :: Invalid_Post_Table_Format`
- `FT_Err_Invalid_Post_Table            :: Invalid_Post_Table`
- `FT_Err_DEF_In_Glyf_Bytecode          :: DEF_In_Glyf_Bytecode`
- `FT_Err_Missing_Bitmap                :: Missing_Bitmap`
- `FT_Err_Missing_SVG_Hooks             :: Missing_SVG_Hooks`
- `FT_Err_Syntax_Error                  :: Syntax_Error`
- `FT_Err_Stack_Underflow               :: Stack_Underflow`
- `FT_Err_Ignore                        :: Ignore`
- `FT_Err_No_Unicode_Glyph_Name         :: No_Unicode_Glyph_Name`
- `FT_Err_Glyph_Too_Big                 :: Glyph_Too_Big`
- `FT_Err_Missing_Startfont_Field       :: Missing_Startfont_Field`
- `FT_Err_Missing_Font_Field            :: Missing_Font_Field`
- `FT_Err_Missing_Size_Field            :: Missing_Size_Field`
- `FT_Err_Missing_Fontboundingbox_Field :: Missing_Fontboundingbox_Field`
- `FT_Err_Missing_Chars_Field           :: Missing_Chars_Field`
- `FT_Err_Missing_Startchar_Field       :: Missing_Startchar_Field`
- `FT_Err_Missing_Encoding_Field        :: Missing_Encoding_Field`
- `FT_Err_Missing_Bbx_Field             :: Missing_Bbx_Field`
- `FT_Err_Bbx_Too_Big                   :: Bbx_Too_Big`
- `FT_Err_Corrupted_Font_Header         :: Corrupted_Font_Header`
- `FT_Err_Corrupted_Font_Glyphs         :: Corrupted_Font_Glyphs`
- `FT_Err_Max                           :: Max`

#### `FT_Glyph_Metrics_` (struct)

- `width:        FT_Pos`
- `height:       FT_Pos`
- `horiBearingX: FT_Pos`
- `horiBearingY: FT_Pos`
- `horiAdvance:  FT_Pos`
- `vertBearingX: FT_Pos`
- `vertBearingY: FT_Pos`
- `vertAdvance:  FT_Pos`

#### `FT_Bitmap_Size_` (struct)

- `height: FT_Short`
- `width:  FT_Short`
- `size:   FT_Pos`
- `x_ppem: FT_Pos`
- `y_ppem: FT_Pos`

#### `FT_Encoding_` (enum)

- `NONE           :: 0`
- `MS_SYMBOL      :: 1937337698`
- `UNICODE        :: 1970170211`
- `SJIS           :: 1936353651`
- `PRC            :: 1734484000`
- `BIG5           :: 1651074869`
- `WANSUNG        :: 2002873971`
- `JOHAB          :: 1785686113`
- `GB2312         :: 1734484000`
- `MS_SJIS        :: 1936353651`
- `MS_GB2312      :: 1734484000`
- `MS_BIG5        :: 1651074869`
- `MS_WANSUNG     :: 2002873971`
- `MS_JOHAB       :: 1785686113`
- `ADOBE_STANDARD :: 1094995778`
- `ADOBE_EXPERT   :: 1094992453`
- `ADOBE_CUSTOM   :: 1094992451`
- `ADOBE_LATIN_1  :: 1818326065`
- `OLD_LATIN_2    :: 1818326066`
- `APPLE_ROMAN    :: 1634889070`
- `FT_ENCODING_NONE           :: NONE`
- `FT_ENCODING_MS_SYMBOL      :: MS_SYMBOL`
- `FT_ENCODING_UNICODE        :: UNICODE`
- `FT_ENCODING_SJIS           :: SJIS`
- `FT_ENCODING_PRC            :: PRC`
- `FT_ENCODING_BIG5           :: BIG5`
- `FT_ENCODING_WANSUNG        :: WANSUNG`
- `FT_ENCODING_JOHAB          :: JOHAB`
- `FT_ENCODING_GB2312         :: GB2312`
- `FT_ENCODING_MS_SJIS        :: MS_SJIS`
- `FT_ENCODING_MS_GB2312      :: MS_GB2312`
- `FT_ENCODING_MS_BIG5        :: MS_BIG5`
- `FT_ENCODING_MS_WANSUNG     :: MS_WANSUNG`
- `FT_ENCODING_MS_JOHAB       :: MS_JOHAB`
- `FT_ENCODING_ADOBE_STANDARD :: ADOBE_STANDARD`
- `FT_ENCODING_ADOBE_EXPERT   :: ADOBE_EXPERT`
- `FT_ENCODING_ADOBE_CUSTOM   :: ADOBE_CUSTOM`
- `FT_ENCODING_ADOBE_LATIN_1  :: ADOBE_LATIN_1`
- `FT_ENCODING_OLD_LATIN_2    :: OLD_LATIN_2`
- `FT_ENCODING_APPLE_ROMAN    :: APPLE_ROMAN`

#### `FT_CharMapRec_` (struct)

- `face:        FT_Face`
- `encoding:    FT_Encoding`
- `platform_id: FT_UShort`
- `encoding_id: FT_UShort`

#### `FT_FaceRec_` (struct)

- `num_faces:           FT_Long`
- `face_index:          FT_Long`
- `face_flags:          FT_Long`
- `style_flags:         FT_Long`
- `num_glyphs:          FT_Long`
- `family_name:         *FT_String`
- `style_name:          *FT_String`
- `num_fixed_sizes:     FT_Int`
- `available_sizes:     *FT_Bitmap_Size`
- `num_charmaps:        FT_Int`
- `charmaps:            *FT_CharMap`
- `generic:             FT_Generic`
- `bbox:                FT_BBox`
- `units_per_EM:        FT_UShort`
- `ascender:            FT_Short`
- `descender:           FT_Short`
- `height:              FT_Short`
- `max_advance_width:   FT_Short`
- `max_advance_height:  FT_Short`
- `underline_position:  FT_Short`
- `underline_thickness: FT_Short`
- `glyph:               FT_GlyphSlot`
- `size:                FT_Size`
- `charmap:             FT_CharMap`
- `driver:              FT_Driver`
- `memory:              FT_Memory`
- `stream:              FT_Stream`
- `sizes_list:          FT_ListRec`
- `autohint:            FT_Generic`
- `extensions:          *void`
- `internal:            FT_Face_Internal`

#### `FT_Size_Metrics_` (struct)

- `x_ppem:      FT_UShort`
- `y_ppem:      FT_UShort`
- `x_scale:     FT_Fixed`
- `y_scale:     FT_Fixed`
- `ascender:    FT_Pos`
- `descender:   FT_Pos`
- `height:      FT_Pos`
- `max_advance: FT_Pos`

#### `FT_SizeRec_` (struct)

- `face:     FT_Face`
- `generic:  FT_Generic`
- `metrics:  FT_Size_Metrics`
- `internal: FT_Size_Internal`

#### `FT_GlyphSlotRec_` (struct)

- `library:           FT_Library`
- `face:              FT_Face`
- `next:              FT_GlyphSlot`
- `glyph_index:       FT_UInt`
- `generic:           FT_Generic`
- `metrics:           FT_Glyph_Metrics`
- `linearHoriAdvance: FT_Fixed`
- `linearVertAdvance: FT_Fixed`
- `advance:           FT_Vector`
- `format:            FT_Glyph_Format`
- `bitmap:            FT_Bitmap`
- `bitmap_left:       FT_Int`
- `bitmap_top:        FT_Int`
- `outline:           FT_Outline`
- `num_subglyphs:     FT_UInt`
- `subglyphs:         FT_SubGlyph`
- `control_data:      *void`
- `control_len:       s32`
- `lsb_delta:         FT_Pos`
- `rsb_delta:         FT_Pos`
- `other:             *void`
- `internal:          FT_Slot_Internal`

#### `FT_Parameter_` (struct)

- `tag:  FT_ULong`
- `data: FT_Pointer`

#### `FT_Open_Args_` (struct)

- `flags:       FT_UInt`
- `memory_base: *FT_Byte`
- `memory_size: FT_Long`
- `pathname:    *FT_String`
- `stream:      FT_Stream`
- `driver:      FT_Module`
- `num_params:  FT_Int`
- `params:      *FT_Parameter`

#### `FT_Size_Request_Type_` (enum)

- `NOMINAL  :: 0`
- `REAL_DIM :: 1`
- `BBOX     :: 2`
- `CELL     :: 3`
- `SCALES   :: 4`
- `MAX      :: 5`
- `FT_SIZE_REQUEST_TYPE_NOMINAL  :: NOMINAL`
- `FT_SIZE_REQUEST_TYPE_REAL_DIM :: REAL_DIM`
- `FT_SIZE_REQUEST_TYPE_BBOX     :: BBOX`
- `FT_SIZE_REQUEST_TYPE_CELL     :: CELL`
- `FT_SIZE_REQUEST_TYPE_SCALES   :: SCALES`
- `FT_SIZE_REQUEST_TYPE_MAX      :: MAX`

#### `FT_Size_RequestRec_` (struct)

- `type:           FT_Size_Request_Type`
- `width:          FT_Long`
- `height:         FT_Long`
- `horiResolution: FT_UInt`
- `vertResolution: FT_UInt`

#### `FT_Render_Mode_` (enum)

- `NORMAL :: 0`
- `LIGHT  :: 1`
- `MONO   :: 2`
- `LCD    :: 3`
- `LCD_V  :: 4`
- `SDF    :: 5`
- `MAX    :: 6`
- `FT_RENDER_MODE_NORMAL :: NORMAL`
- `FT_RENDER_MODE_LIGHT  :: LIGHT`
- `FT_RENDER_MODE_MONO   :: MONO`
- `FT_RENDER_MODE_LCD    :: LCD`
- `FT_RENDER_MODE_LCD_V  :: LCD_V`
- `FT_RENDER_MODE_SDF    :: SDF`
- `FT_RENDER_MODE_MAX    :: MAX`

#### `FT_Kerning_Mode_` (enum)

- `DEFAULT  :: 0`
- `UNFITTED :: 1`
- `UNSCALED :: 2`
- `FT_KERNING_DEFAULT  :: DEFAULT`
- `FT_KERNING_UNFITTED :: UNFITTED`
- `FT_KERNING_UNSCALED :: UNSCALED`

#### `TT_Header_` (struct)

- `Table_Version:       FT_Fixed`
- `Font_Revision:       FT_Fixed`
- `CheckSum_Adjust:     FT_Long`
- `Magic_Number:        FT_Long`
- `Flags:               FT_UShort`
- `Units_Per_EM:        FT_UShort`
- `Created:             [2] FT_ULong`
- `Modified:            [2] FT_ULong`
- `xMin:                FT_Short`
- `yMin:                FT_Short`
- `xMax:                FT_Short`
- `yMax:                FT_Short`
- `Mac_Style:           FT_UShort`
- `Lowest_Rec_PPEM:     FT_UShort`
- `Font_Direction:      FT_Short`
- `Index_To_Loc_Format: FT_Short`
- `Glyph_Data_Format:   FT_Short`

#### `TT_HoriHeader_` (struct)

- `Version:                FT_Fixed`
- `Ascender:               FT_Short`
- `Descender:              FT_Short`
- `Line_Gap:               FT_Short`
- `advance_Width_Max:      FT_UShort`
- `min_Left_Side_Bearing:  FT_Short`
- `min_Right_Side_Bearing: FT_Short`
- `xMax_Extent:            FT_Short`
- `caret_Slope_Rise:       FT_Short`
- `caret_Slope_Run:        FT_Short`
- `caret_Offset:           FT_Short`
- `Reserved:               [4] FT_Short`
- `metric_Data_Format:     FT_Short`
- `number_Of_HMetrics:     FT_UShort`
- `long_metrics:           *void`
- `short_metrics:          *void`

#### `TT_VertHeader_` (struct)

- `Version:                 FT_Fixed`
- `Ascender:                FT_Short`
- `Descender:               FT_Short`
- `Line_Gap:                FT_Short`
- `advance_Height_Max:      FT_UShort`
- `min_Top_Side_Bearing:    FT_Short`
- `min_Bottom_Side_Bearing: FT_Short`
- `yMax_Extent:             FT_Short`
- `caret_Slope_Rise:        FT_Short`
- `caret_Slope_Run:         FT_Short`
- `caret_Offset:            FT_Short`
- `Reserved:                [4] FT_Short`
- `metric_Data_Format:      FT_Short`
- `number_Of_VMetrics:      FT_UShort`
- `long_metrics:            *void`
- `short_metrics:           *void`

#### `TT_OS2_` (struct)

- `version:                 FT_UShort`
- `xAvgCharWidth:           FT_Short`
- `usWeightClass:           FT_UShort`
- `usWidthClass:            FT_UShort`
- `fsType:                  FT_UShort`
- `ySubscriptXSize:         FT_Short`
- `ySubscriptYSize:         FT_Short`
- `ySubscriptXOffset:       FT_Short`
- `ySubscriptYOffset:       FT_Short`
- `ySuperscriptXSize:       FT_Short`
- `ySuperscriptYSize:       FT_Short`
- `ySuperscriptXOffset:     FT_Short`
- `ySuperscriptYOffset:     FT_Short`
- `yStrikeoutSize:          FT_Short`
- `yStrikeoutPosition:      FT_Short`
- `sFamilyClass:            FT_Short`
- `panose:                  [10] FT_Byte`
- `ulUnicodeRange1:         FT_ULong`
- `ulUnicodeRange2:         FT_ULong`
- `ulUnicodeRange3:         FT_ULong`
- `ulUnicodeRange4:         FT_ULong`
- `achVendID:               [4] FT_Char`
- `fsSelection:             FT_UShort`
- `usFirstCharIndex:        FT_UShort`
- `usLastCharIndex:         FT_UShort`
- `sTypoAscender:           FT_Short`
- `sTypoDescender:          FT_Short`
- `sTypoLineGap:            FT_Short`
- `usWinAscent:             FT_UShort`
- `usWinDescent:            FT_UShort`
- `ulCodePageRange1:        FT_ULong`
- `ulCodePageRange2:        FT_ULong`
- `sxHeight:                FT_Short`
- `sCapHeight:              FT_Short`
- `usDefaultChar:           FT_UShort`
- `usBreakChar:             FT_UShort`
- `usMaxContext:            FT_UShort`
- `usLowerOpticalPointSize: FT_UShort`
- `usUpperOpticalPointSize: FT_UShort`

#### `TT_Postscript_` (struct)

- `FormatType:         FT_Fixed`
- `italicAngle:        FT_Fixed`
- `underlinePosition:  FT_Short`
- `underlineThickness: FT_Short`
- `isFixedPitch:       FT_ULong`
- `minMemType42:       FT_ULong`
- `maxMemType42:       FT_ULong`
- `minMemType1:        FT_ULong`
- `maxMemType1:        FT_ULong`

#### `TT_PCLT_` (struct)

- `Version:             FT_Fixed`
- `FontNumber:          FT_ULong`
- `Pitch:               FT_UShort`
- `xHeight:             FT_UShort`
- `Style:               FT_UShort`
- `TypeFamily:          FT_UShort`
- `CapHeight:           FT_UShort`
- `SymbolSet:           FT_UShort`
- `TypeFace:            [16] FT_Char`
- `CharacterComplement: [8] FT_Char`
- `FileName:            [6] FT_Char`
- `StrokeWeight:        FT_Char`
- `WidthType:           FT_Char`
- `SerifStyle:          FT_Byte`
- `Reserved:            FT_Byte`

#### `TT_MaxProfile_` (struct)

- `version:               FT_Fixed`
- `numGlyphs:             FT_UShort`
- `maxPoints:             FT_UShort`
- `maxContours:           FT_UShort`
- `maxCompositePoints:    FT_UShort`
- `maxCompositeContours:  FT_UShort`
- `maxZones:              FT_UShort`
- `maxTwilightPoints:     FT_UShort`
- `maxStorage:            FT_UShort`
- `maxFunctionDefs:       FT_UShort`
- `maxInstructionDefs:    FT_UShort`
- `maxStackElements:      FT_UShort`
- `maxSizeOfInstructions: FT_UShort`
- `maxComponentElements:  FT_UShort`
- `maxComponentDepth:     FT_UShort`

#### `FT_Sfnt_Tag_` (enum)

- `HEAD :: 0`
- `MAXP :: 1`
- `OS2  :: 2`
- `HHEA :: 3`
- `VHEA :: 4`
- `POST :: 5`
- `PCLT :: 6`
- `MAX  :: 7`
- `FT_SFNT_HEAD :: HEAD`
- `FT_SFNT_MAXP :: MAXP`
- `FT_SFNT_OS2  :: OS2`
- `FT_SFNT_HHEA :: HHEA`
- `FT_SFNT_VHEA :: VHEA`
- `FT_SFNT_POST :: POST`
- `FT_SFNT_PCLT :: PCLT`
- `FT_SFNT_MAX  :: MAX`

## Behavioral contract and constraints

This package exposes FreeType C API records, constants, and foreign calls, with platform-specific library bindings. It does not ship FreeType source; the binding notes that environment-property configuration can affect builds. Handles and allocated glyph/face data follow FreeType's ownership and destroy/release calls; error values are FreeType status codes. The exact linked FreeType version is platform-dependent except where a versioned compatibility module says otherwise.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: `freetype`.


Platform coverage note: exports for PS5 and Switch 2 targets are unspecified.
