# pl_mpeg


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `BUFFER_DEFAULT_SIZE :: 128 * 1024`
- `INVALID_TS : float64 : -1.0`
- `Packet :: struct`
- `Plane :: struct`
- `Frame :: struct`
- `Video_Decode_Callback :: #type (self: *Plm, frame: *Frame, user: *void)`
- `SAMPLES_PER_FRAME :: 1152`
- `Samples :: struct`
- `Audio_Decode_Callback :: #type (plm: *Plm, samples: *Samples, user: *void)`
- `Buffer_Load_Callback :: #type (buffer: *Buffer, user: *void)`
- `Plm :: struct`
- `create_with_filename :: (filename: string) -> *Plm`
- `create_with_file :: (fh: File, close_when_done: bool) -> *Plm`
- `create_with_memory :: (memory: [] u8, free_when_done: bool) -> *Plm`
- `create_with_buffer :: (buffer: *Buffer, destroy_when_done: bool) -> *Plm`
- `plm_destroy :: (using self: *Plm)`
- `has_headers :: (using self: *Plm) -> bool`
- `set_audio_enabled :: (using self: *Plm, enabled: bool)`
- `set_audio_stream :: (self: *Plm, stream_index: s32)`
- `set_video_enabled :: (using self: *Plm, enabled: bool)`
- `get_num_video_streams :: (self: *Plm) -> s32`
- `get_size :: (self: *Plm) -> (width: s32, height: s32)`
- `get_framerate :: (self: *Plm) -> float64`
- `get_samplerate :: (self: *Plm) -> s32`
- `rewind :: (using self: *Plm)`
- `set_loop :: (self: *Plm, loop: bool)`
- `set_video_decode_callback :: (self: *Plm, callback: Video_Decode_Callback, user: *void)`
- `set_audio_decode_callback :: (self: *Plm, callback: Audio_Decode_Callback, user: *void)`
- `decode :: (self: *Plm, tick: float64)`
- `decode_video :: (self: *Plm) -> *Frame`
- `decode_audio :: (self: *Plm) -> *Samples`
- `handle_end :: (self: *Plm)`
- `read_video_packet :: (buffer: *Buffer, user: *void)`
- `read_audio_packet :: (buffer: *Buffer, user: *void)`
- `read_packets :: (self: *Plm, requested_type: Start_Code)`
- `seek_frame :: (self: *Plm, time: float64, seek_exact: bool) -> *Frame`
- `seek :: (self: *Plm, time: float64, seek_exact: bool) -> bool`
- `Buffer_Mode :: enum`
- `Buffer :: struct`
- `Vlc :: struct`
- `Vlc_Uint :: struct`
- `buffer_create_with_filename :: (filename: string) -> *Buffer`
- `buffer_create_with_file :: (fh: File, close_when_done: bool) -> *Buffer`
- `buffer_create_with_memory :: (memory: [] u8, free_when_done: bool) -> *Buffer`
- `buffer_create_with_capacity :: (capacity: s64) -> *Buffer`
- `create_for_appending :: (initial_capacity: s64) -> *Buffer`
- `buffer_destroy :: (using buffer: *Buffer)`
- `buffer_get_size :: (buffer: *Buffer) -> s64`
- `buffer_get_remaining :: (using buffer: *Buffer) -> s64`
- `buffer_write :: (using buffer: *Buffer, memory: [] u8) -> s64`
- `buffer_signal_end :: (buffer: *Buffer)`
- `buffer_set_load_callback :: (buffer: *Buffer, callback: Buffer_Load_Callback, user: *void)`
- `buffer_rewind :: (buffer: *Buffer)`
- `buffer_seek :: (buffer: *Buffer, pos: s64)`
- `buffer_tell :: (buffer: *Buffer) -> s64`
- `buffer_discard_read_bytes :: (using buffer: *Buffer)`
- `buffer_load_file_callback :: (using buffer: *Buffer, user: *void)`
- `buffer_has :: (buffer: *Buffer, count: s64) -> bool`
- `buffer_read :: (buffer: *Buffer, count: s64) -> s32`
- `buffer_align :: (buffer: *Buffer)`
- `buffer_skip :: (buffer: *Buffer, count: s64)`
- `buffer_skip_bytes :: (buffer: *Buffer, v: u8) -> s32`
- `buffer_next_start_code :: (buffer: *Buffer) -> Start_Code`
- `buffer_find_start_code :: (buffer: *Buffer, code: Start_Code) -> bool`
- `buffer_has_start_code :: (buffer: *Buffer, code: Start_Code) -> bool`
- `buffer_no_start_code :: (buffer: *Buffer) -> bool`
- `buffer_read_vlc :: (buffer: *Buffer, table: [] Vlc) -> s16`
- `buffer_read_vlc_uint :: (buffer: *Buffer, table: [] Vlc_Uint) -> u16`
- `FRAME_SYNC :: 0x7ff`
- `MPEG_2_5 :: 0x0`
- `MPEG_2 :: 0x2`
- `MPEG_1 :: 0x3`
- `LAYER_III :: 0x1`
- `LAYER_II :: 0x2`
- `LAYER_I :: 0x3`
- `Audio_Mode :: enum u8`
- `SAMPLE_RATE :: u16.[ 44100, 48000, 32000, 0, 22050, 24000, 16000, 0 ]`
- `BIT_RATE :: s16.[ 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160 ]`
- `SCALEFACTOR_BASE :: u32.[ 0x02000000, 0x01965FEA, 0x01428A30 ]`
- `SYNTHESIS_WINDOW :: float.[...]` (512 entries; full constant data: [pl_mpeg public constant tables](pl-mpeg-public-tables.md))
- `QUANT_LUT_STEP_1 :: ([16]u8).[ .[ 0, 0, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0 ], .[ 0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 2, 2, 0, 0 ] ]`
- `QUANT_TAB_A : u8 : (27 | 64)`
- `QUANT_TAB_B : u8 : (30 | 64)`
- `QUANT_TAB_C : u8 : 8`
- `QUANT_TAB_D : u8 : 12`
- `QUANT_LUT_STEP_2 :: ([3] u8).[ .[ QUANT_TAB_C, QUANT_TAB_C, QUANT_TAB_D ], .[ QUANT_TAB_A, QUANT_TAB_A, QUANT_TAB_A ], .[ QUANT_TAB_B, QUANT_TAB_A, QUANT_TAB_B ] ]`
- `QUANT_LUT_STEP_3 :: ([32] u8).[ .[ 0x44,0x44, 0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, ], .[ 0x43,0x43,0x43, 0x42,0x42,0x42,0x42,0x42,0x42,0x42,0x42, 0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31, 0x20,0x20,0x20,0x20,0x20,0x20,0x20, 0x00, 0x00 ], .[ 0x45,0x45,0x45,0x45, 0x34,0x34,0x34,0x34,0x34,0x34,0x34, 0x24,0x24,0x24,0x24,0x24,0x24,0x24,0x24,0x24,0x24, 0x24,0x24,0x24,0x24,0x24,0x24,0x24,0x24,0x24, 0x00, 0x00 ] ]`
- `QUANT_LUT_STEP4 :: ([16] u8).[ .[ 0, 1, 2, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], .[ 0, 1, 2, 3, 4, 5, 6, 17, 0, 0, 0, 0, 0, 0, 0, 0 ], .[ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 17 ], .[ 0, 1, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17 ], .[ 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17 ], .[ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 ] ]`
- `Quantizer_Spec :: struct`
- `QUANT_TAB :: Quantizer_Spec.[ .{ 3, 1, 5 }, .{ 5, 1, 7 }, .{ 7, 0, 3 }, .{ 9, 1, 10 }, .{ 15, 0, 4 }, .{ 31, 0, 5 }, .{ 63, 0, 6 }, .{ 127, 0, 7 }, .{ 255, 0, 8 }, .{ 511, 0, 9 }, .{ 1023, 0, 10 }, .{ 2047, 0, 11 }, .{ 4095, 0, 12 }, .{ 8191, 0, 13 }, .{ 16383, 0, 14 }, .{ 32767, 0, 15 }, .{ 65535, 0, 16 } ]`
- `Audio :: struct`
- `audio_create_with_buffer :: (buffer: *Buffer, destroy_when_done: bool) -> *Audio`
- `audio_destroy :: (audio: *Audio)`
- `audio_has_header :: (audio: *Audio) -> bool`
- `audio_set_time :: (audio: *Audio, time: float64)`
- `audio_rewind :: (audio: *Audio)`
- `audio_has_ended :: (audio: *Audio) -> bool`
- `audio_decode :: (audio: *Audio) -> *Samples`
- `audio_find_frame_sync :: (audio: *Audio) -> bool`
- `audio_decode_header :: (audio: *Audio) -> s32`
- `audio_decode_frame :: (audio: *Audio)`
- `audio_read_allocation :: (audio: *Audio, sb: s32, tab3: u8) -> *Quantizer_Spec`
- `audio_read_samples :: (audio: *Audio, ch: s32, sb: s32, part: s32)`
- `audio_matrix_transform :: (s: [32][3] s32, ss: s32, d: *float, dp: s32)`
- `Picture_Type :: enum`
- `Start_Code :: enum u8`
- `IS_SLICE :: inline (c: Start_Code) -> bool`
- `PICTURE_RATE :: float.[ 0.000, 23.976, 24.000, 25.000, 29.970, 30.000, 50.000, 59.940, 60.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000, 0.000 ]`
- `ZIG_ZAG :: u8.[ 0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63 ]`
- `INTRA_QUANT_MATRIX :: u8.[ 8, 16, 19, 22, 26, 27, 29, 34, 16, 16, 22, 24, 27, 29, 34, 37, 19, 22, 26, 27, 29, 34, 34, 38, 22, 22, 26, 27, 29, 34, 37, 40, 22, 26, 27, 29, 32, 35, 40, 48, 26, 27, 29, 32, 35, 40, 48, 58, 26, 27, 29, 34, 38, 46, 56, 69, 27, 29, 35, 38, 46, 56, 69, 83 ]`
- `NON_INTRA_QUANT_MATRIX :: u8.[ 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16 ]`
- `PREMULTIPLIER_MATRIX :: u8.[ 32, 44, 42, 38, 32, 25, 17, 9, 44, 62, 58, 52, 44, 35, 24, 12, 42, 58, 55, 49, 42, 33, 23, 12, 38, 52, 49, 44, 38, 30, 20, 10, 32, 44, 42, 38, 32, 25, 17, 9, 25, 35, 33, 30, 25, 20, 14, 7, 17, 24, 23, 20, 17, 14, 9, 5, 9, 12, 12, 10, 9, 7, 5, 2 ]`
- `MACROBLOCK_ADDRESS_INCREMENT :: Vlc.[ .{ 1 << 1, 0}, .{ 0, 1}, .{ 2 << 1, 0}, .{ 3 << 1, 0}, .{ 4 << 1, 0}, .{ 5 << 1, 0}, .{ 0, 3}, .{ 0, 2}, .{ 6 << 1, 0}, .{ 7 << 1, 0}, .{ 0, 5}, .{ 0, 4}, .{ 8 << 1, 0}, .{ 9 << 1, 0}, .{ 0, 7}, .{ 0, 6}, .{ 10 << 1, 0}, .{ 11 << 1, 0}, .{ 12 << 1, 0}, .{ 13 << 1, 0}, .{ 14 << 1, 0}, .{ 15 << 1, 0}, .{ 16 << 1, 0}, .{ 17 << 1, 0}, .{ 18 << 1, 0}, .{ 19 << 1, 0}, .{ 0, 9}, .{ 0, 8}, .{ -1, 0}, .{ 20 << 1, 0}, .{ -1, 0}, .{ 21 << 1, 0}, .{ 22 << 1, 0}, .{ 23 << 1, 0}, .{ 0, 15}, .{ 0, 14}, .{ 0, 13}, .{ 0, 12}, .{ 0, 11}, .{ 0, 10}, .{ 24 << 1, 0}, .{ 25 << 1, 0}, .{ 26 << 1, 0}, .{ 27 << 1, 0}, .{ 28 << 1, 0}, .{ 29 << 1, 0}, .{ 30 << 1, 0}, .{ 31 << 1, 0}, .{ 32 << 1, 0}, .{ -1, 0}, .{ -1, 0}, .{ 33 << 1, 0}, .{ 34 << 1, 0}, .{ 35 << 1, 0}, .{ 36 << 1, 0}, .{ 37 << 1, 0}, .{ 38 << 1, 0}, .{ 39 << 1, 0}, .{ 0, 21}, .{ 0, 20}, .{ 0, 19}, .{ 0, 18}, .{ 0, 17}, .{ 0, 16}, .{ 0, 35}, .{ -1, 0}, .{ -1, 0}, .{ 0, 34}, .{ 0, 33}, .{ 0, 32}, .{ 0, 31}, .{ 0, 30}, .{ 0, 29}, .{ 0, 28}, .{ 0, 27}, .{ 0, 26}, .{ 0, 25}, .{ 0, 24}, .{ 0, 23}, .{ 0, 22}, ]`
- `MACROBLOCK_TYPE_INTRA :: Vlc.[ .{ 1 << 1, 0}, .{ 0, 0x01}, .{ -1, 0}, .{ 0, 0x11}, ]`
- `MACROBLOCK_TYPE_PREDICTIVE :: Vlc.[ .{ 1 << 1, 0}, .{ 0, 0x0a}, .{ 2 << 1, 0}, .{ 0, 0x02}, .{ 3 << 1, 0}, .{ 0, 0x08}, .{ 4 << 1, 0}, .{ 5 << 1, 0}, .{ 6 << 1, 0}, .{ 0, 0x12}, .{ 0, 0x1a}, .{ 0, 0x01}, .{ -1, 0}, .{ 0, 0x11}, ]`
- `MACROBLOCK_TYPE_B :: Vlc.[ .{ 1 << 1, 0}, .{ 2 << 1, 0}, .{ 3 << 1, 0}, .{ 4 << 1, 0}, .{ 0, 0x0c}, .{ 0, 0x0e}, .{ 5 << 1, 0}, .{ 6 << 1, 0}, .{ 0, 0x04}, .{ 0, 0x06}, .{ 7 << 1, 0}, .{ 8 << 1, 0}, .{ 0, 0x08}, .{ 0, 0x0a}, .{ 9 << 1, 0}, .{ 10 << 1, 0}, .{ 0, 0x1e}, .{ 0, 0x01}, .{ -1, 0}, .{ 0, 0x11}, .{ 0, 0x16}, .{ 0, 0x1a}, ]`
- `CODE_BLOCK_PATTERN :: Vlc.[ .{ 1 << 1, 0}, .{ 2 << 1, 0}, .{ 3 << 1, 0}, .{ 4 << 1, 0}, .{ 5 << 1, 0}, .{ 6 << 1, 0}, .{ 7 << 1, 0}, .{ 8 << 1, 0}, .{ 9 << 1, 0}, .{ 10 << 1, 0}, .{ 11 << 1, 0}, .{ 12 << 1, 0}, .{ 13 << 1, 0}, .{ 0, 60}, .{ 14 << 1, 0}, .{ 15 << 1, 0}, .{ 16 << 1, 0}, .{ 17 << 1, 0}, .{ 18 << 1, 0}, .{ 19 << 1, 0}, .{ 20 << 1, 0}, .{ 21 << 1, 0}, .{ 22 << 1, 0}, .{ 23 << 1, 0}, .{ 0, 32}, .{ 0, 16}, .{ 0, 8}, .{ 0, 4}, .{ 24 << 1, 0}, .{ 25 << 1, 0}, .{ 26 << 1, 0}, .{ 27 << 1, 0}, .{ 28 << 1, 0}, .{ 29 << 1, 0}, .{ 30 << 1, 0}, .{ 31 << 1, 0}, .{ 0, 62}, .{ 0, 2}, .{ 0, 61}, .{ 0, 1}, .{ 0, 56}, .{ 0, 52}, .{ 0, 44}, .{ 0, 28}, .{ 0, 40}, .{ 0, 20}, .{ 0, 48}, .{ 0, 12}, .{ 32 << 1, 0}, .{ 33 << 1, 0}, .{ 34 << 1, 0}, .{ 35 << 1, 0}, .{ 36 << 1, 0}, .{ 37 << 1, 0}, .{ 38 << 1, 0}, .{ 39 << 1, 0}, .{ 40 << 1, 0}, .{ 41 << 1, 0}, .{ 42 << 1, 0}, .{ 43 << 1, 0}, .{ 0, 63}, .{ 0, 3}, .{ 0, 36}, .{ 0, 24}, .{ 44 << 1, 0}, .{ 45 << 1, 0}, .{ 46 << 1, 0}, .{ 47 << 1, 0}, .{ 48 << 1, 0}, .{ 49 << 1, 0}, .{ 50 << 1, 0}, .{ 51 << 1, 0}, .{ 52 << 1, 0}, .{ 53 << 1, 0}, .{ 54 << 1, 0}, .{ 55 << 1, 0}, .{ 56 << 1, 0}, .{ 57 << 1, 0}, .{ 58 << 1, 0}, .{ 59 << 1, 0}, .{ 0, 34}, .{ 0, 18}, .{ 0, 10}, .{ 0, 6}, .{ 0, 33}, .{ 0, 17}, .{ 0, 9}, .{ 0, 5}, .{ -1, 0}, .{ 60 << 1, 0}, .{ 61 << 1, 0}, .{ 62 << 1, 0}, .{ 0, 58}, .{ 0, 54}, .{ 0, 46}, .{ 0, 30}, .{ 0, 57}, .{ 0, 53}, .{ 0, 45}, .{ 0, 29}, .{ 0, 38}, .{ 0, 26}, .{ 0, 37}, .{ 0, 25}, .{ 0, 43}, .{ 0, 23}, .{ 0, 51}, .{ 0, 15}, .{ 0, 42}, .{ 0, 22}, .{ 0, 50}, .{ 0, 14}, .{ 0, 41}, .{ 0, 21}, .{ 0, 49}, .{ 0, 13}, .{ 0, 35}, .{ 0, 19}, .{ 0, 11}, .{ 0, 7}, .{ 0, 39}, .{ 0, 27}, .{ 0, 59}, .{ 0, 55}, .{ 0, 47}, .{ 0, 31}, ]`
- `MOTION :: Vlc.[ .{ 1 << 1, 0}, .{ 0, 0}, .{ 2 << 1, 0}, .{ 3 << 1, 0}, .{ 4 << 1, 0}, .{ 5 << 1, 0}, .{ 0, 1}, .{ 0, -1}, .{ 6 << 1, 0}, .{ 7 << 1, 0}, .{ 0, 2}, .{ 0, -2}, .{ 8 << 1, 0}, .{ 9 << 1, 0}, .{ 0, 3}, .{ 0, -3}, .{ 10 << 1, 0}, .{ 11 << 1, 0}, .{ 12 << 1, 0}, .{ 13 << 1, 0}, .{ -1, 0}, .{ 14 << 1, 0}, .{ 15 << 1, 0}, .{ 16 << 1, 0}, .{ 17 << 1, 0}, .{ 18 << 1, 0}, .{ 0, 4}, .{ 0, -4}, .{ -1, 0}, .{ 19 << 1, 0}, .{ 20 << 1, 0}, .{ 21 << 1, 0}, .{ 0, 7}, .{ 0, -7}, .{ 0, 6}, .{ 0, -6}, .{ 0, 5}, .{ 0, -5}, .{ 22 << 1, 0}, .{ 23 << 1, 0}, .{ 24 << 1, 0}, .{ 25 << 1, 0}, .{ 26 << 1, 0}, .{ 27 << 1, 0}, .{ 28 << 1, 0}, .{ 29 << 1, 0}, .{ 30 << 1, 0}, .{ 31 << 1, 0}, .{ 32 << 1, 0}, .{ 33 << 1, 0}, .{ 0, 10}, .{ 0, -10}, .{ 0, 9}, .{ 0, -9}, .{ 0, 8}, .{ 0, -8}, .{ 0, 16}, .{ 0, -16}, .{ 0, 15}, .{ 0, -15}, .{ 0, 14}, .{ 0, -14}, .{ 0, 13}, .{ 0, -13}, .{ 0, 12}, .{ 0, -12}, .{ 0, 11}, .{ 0, -11}, ]`
- `DCT_SIZE_LUMINANCE :: Vlc.[ .{ 1 << 1, 0}, .{ 2 << 1, 0}, .{ 0, 1}, .{ 0, 2}, .{ 3 << 1, 0}, .{ 4 << 1, 0}, .{ 0, 0}, .{ 0, 3}, .{ 0, 4}, .{ 5 << 1, 0}, .{ 0, 5}, .{ 6 << 1, 0}, .{ 0, 6}, .{ 7 << 1, 0}, .{ 0, 7}, .{ 8 << 1, 0}, .{ 0, 8}, .{ -1, 0}, ]`
- `DCT_SIZE_CHROMINANCE :: Vlc.[ .{ 1 << 1, 0}, .{ 2 << 1, 0}, .{ 0, 0}, .{ 0, 1}, .{ 0, 2}, .{ 3 << 1, 0}, .{ 0, 3}, .{ 4 << 1, 0}, .{ 0, 4}, .{ 5 << 1, 0}, .{ 0, 5}, .{ 6 << 1, 0}, .{ 0, 6}, .{ 7 << 1, 0}, .{ 0, 7}, .{ 8 << 1, 0}, .{ 0, 8}, .{ -1, 0}, ]`
- `DCT_COEFF :: Vlc_Uint.[...]` (224 entries; full constant data: [pl_mpeg public constant tables](pl-mpeg-public-tables.md))
- `Video_Motion :: struct`
- `Video :: struct`
- `clamp :: inline (n: s32) -> u8`
- `video_create_with_buffer :: (buffer: *Buffer, destroy_when_done: bool) -> *Video`
- `video_destroy :: (video: *Video)`
- `video_get_framerate :: (video: *Video) -> float64`
- `video_set_no_delay :: (video: *Video, no_delay: bool)`
- `video_set_time :: (video: *Video, time: float64)`
- `video_rewind :: (video: *Video)`
- `video_has_ended :: (video: *Video) -> bool`
- `video_decode :: (video: *Video) -> *Frame`
- `video_has_header :: (video: *Video) -> bool`
- `video_decode_sequence_header :: (video: *Video) -> bool`
- `video_init_frame :: (video: *Video, frame: *Frame, base: *u8)`
- `video_decode_picture :: (video: *Video)`
- `video_decode_slice :: (video: *Video, slice: Start_Code)`
- `video_decode_macroblock :: (video: *Video)`
- `video_decode_motion_vectors :: (video: *Video)`
- `video_decode_motion_vector :: (video: *Video, r_size: s32, motion: s32) -> s32`
- `video_predict_macroblock :: (using video: *Video)`
- `video_copy_macroblock :: (video: *Video, motion_h: s32, motion_v: s32, d: *Frame)`
- `video_interpolate_macroblock :: (video: *Video, motion_h: s32, motion_v: s32, d: *Frame)`
- `BLOCK_SET :: (dest: *u8, dest_width: s32, source_width: s32, block_size: s32, code: Code) #expand`
- `video_process_macroblock :: (video: *Video, d: *u8, s: *u8, motion_h: s32, motion_v: s32, block_size: s32, interpolate: bool)`
- `video_decode_block :: (video: *Video, block: s32)`
- `video_idct :: (block: *s32)`
- `PUT_PIXEL :: (RI: s32, GI: s32, BI: s32, y_offset: s32, dest_offset: s32) #expand`
- `convert_function :: (frame: *Frame, dest: *u8, stride: s32, BYTES_PER_PIXEL: s32, RI: s32, GI: s32, BI: s32)`
- `frame_to_rgb :: #bake_arguments convert_function(BYTES_PER_PIXEL=3, RI=0, GI=1, BI=2)`
- `frame_to_bgr :: #bake_arguments convert_function(BYTES_PER_PIXEL=3, RI=2, GI=1, BI=0)`
- `frame_to_rgba :: #bake_arguments convert_function(BYTES_PER_PIXEL=4, RI=0, GI=1, BI=2)`
- `frame_to_bgra :: #bake_arguments convert_function(BYTES_PER_PIXEL=4, RI=2, GI=1, BI=0)`
- `frame_to_argb :: #bake_arguments convert_function(BYTES_PER_PIXEL=4, RI=1, GI=2, BI=3)`
- `frame_to_abgr :: #bake_arguments convert_function(BYTES_PER_PIXEL=4, RI=3, GI=2, BI=1)`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Audio`

```jai
Audio :: struct {
	time: float64;

	samples_decoded: s32;
	samplerate_index: s32;
	bitrate_index: s32;
	version: s32;
	layer: s32;

	mode: Audio_Mode;
	bound: s32;
	v_pos: s32;
	next_frame_data_size: s32;

	has_header := false;

	buffer: *Buffer;
	destroy_buffer_when_done: bool;

    allocation: [2][32] *Quantizer_Spec;

    scale_factor_info: [2][32] u8;

    scale_factor: [2][32][3] s32;
	sample:       [2][32][3] s32;

	samples: Samples;
	D: [1024]    float;
	V: [2][1024] float;
	U: [32]      float;
}
```
### `Audio_Mode`

```jai
Audio_Mode :: enum u8 {
    STEREO       :: 0x0;
    JOINT_STEREO :: 0x1;
    DUAL_CHANNEL :: 0x2;
    MONO         :: 0x3;
}
```
### `Buffer`

```jai
Buffer :: struct {
	bit_index:  s64;
	capacity:   s64;
	length:     s64;
	total_size: s64;

	discard_read_bytes := false;

	has_ended       := false;
	free_when_done  := false;
	close_when_done := false;
    file_initted    := false;

    fh: File;

	load_callback: Buffer_Load_Callback;
	load_callback_user_data: *void;

	bytes: *u8;

    mode: Buffer_Mode;
}
```
### `Buffer_Mode`

```jai
Buffer_Mode :: enum {
	FILE;
	FIXED_MEM;
	RING;
	APPEND;
}
```
### `Frame`

```jai
Frame :: struct {
	time: float64;
	width:  s32;
    height: s32;

    y, cr, cb: Plane;
}
```
### `Packet`

```jai
Packet :: struct {
	type:   Start_Code;
	pts:    float64;
    data:   [] u8;
}
```
### `Picture_Type`

```jai
Picture_Type :: enum {
    INTRA      :: 1;
    PREDICTIVE :: 2;
    B          :: 3;
}
```
### `Plane`

```jai
Plane :: struct {
    width:  s32;
    height: s32;
    data:   *u8;
}
```
### `Plm`

```jai
Plm :: struct {
    demux: *Demux;
	time: float64;

    has_ended:         bool;
    loop:              bool;
    has_decoders:      bool;
    video_enabled   := true;
    video_packet_type: Start_Code;

	video_buffer:  *Buffer;
	video_decoder: *Video;

	audio_enabled   := true;
	audio_stream_index: s32;
	audio_packet_type: Start_Code;

	audio_lead_time: float64;
	audio_buffer: *Buffer;
	audio_decoder: *Audio;

	video_decode_callback: Video_Decode_Callback;
	video_decode_callback_user_data: *void;

	audio_decode_callback: Audio_Decode_Callback;
	audio_decode_callback_user_data: *void;
}
```
### `Quantizer_Spec`

```jai
Quantizer_Spec :: struct {
	levels: u16;
	group:  u8;
	bits:   u8;
}
```
### `Samples`

```jai
Samples :: struct {
	time: float64;
	count: u32;

	interleaved: [SAMPLES_PER_FRAME * 2] float;

}
```
### `Start_Code`

```jai
Start_Code :: enum u8 {
    SEQUENCE    :: 0xB3;
    SLICE_FIRST :: 0x01;
    SLICE_LAST  :: 0xAF;
    PICTURE     :: 0x00;
    EXTENSION   :: 0xB5;
    USER_DATA   :: 0xB2;

    PACK   :: 0xBA;
    END    :: 0xB9;
    SYSTEM :: 0xBB;

    PRIVATE :: 0xBD;
    AUDIO_1 :: 0xC0;
    AUDIO_2 :: 0xC1;
    AUDIO_3 :: 0xC2;
    AUDIO_4 :: 0xC3;
    VIDEO_1 :: 0xE0;

    NOT_FOUND :: 0xFF;
}
```
### `Video`

```jai
Video :: struct {
	framerate: float64;
	time: float64;
	frames_decoded: s32;
	width: s32;
	height: s32;
	mb_width: s32;
	mb_height: s32;
	mb_size: s32;

	luma_width: s32;
	luma_height: s32;

	chroma_width: s32;
	chroma_height: s32;

	start_code:   Start_Code;
	picture_type: Picture_Type;

	motion_forward:  Video_Motion;
	motion_backward: Video_Motion;

	has_sequence_header := false;

	quantizer_scale: s32;
	slice_begin := false;
	macroblock_address: s32;

	mb_row: s32;
	mb_col: s32;

	macroblock_type:  s32;
	macroblock_intra: s32;

	dc_predictor: [3] s32;

	buffer: *Buffer;
	destroy_buffer_when_done: bool;

	frame_current:  Frame;
	frame_forward:  Frame;
	frame_backward: Frame;

	frames_data: *u8;

	block_data: [64] s32;
	intra_quant_matrix:     [64] u8;
	non_intra_quant_matrix: [64] u8;

	has_reference_frame := false;
	assume_no_b_frames  := false;
}
```
### `Video_Motion`

```jai
Video_Motion :: struct {
	full_px: s32;
	is_set: s32;
	r_size: s32;
	h: s32;
	v: s32;
}
```
### `Vlc`

```jai
Vlc :: struct {
	index: s16;
	value: s16;
}
```
### `Vlc_Uint`

```jai
Vlc_Uint :: struct {
	index: s16;
	value: u16;
}
```
## Contracts and constraints

The synthesis-window and DCT-coefficient lookup tables are importer-visible constants; their complete initializers are given in the linked data appendix.

This module exposes an MPEG-1 video/audio decoder API and RGB-family conversion helpers. Decoder state and output buffers are managed through the C library interface; callers must honor decoder callback and buffer lifetimes and check decoder results before reading frame/audio data. The baked conversion procedures select channel order and bytes per pixel (RGB/BGR use three bytes; alpha variants use four); dimensions, stride, and destination capacity must match the selected format. Decoder support and platform availability follow the bundled library/build configuration.

### Public macro contracts

`buffer_no_start_code(buffer)` does not advance the bit cursor. It first ensures 40 bits are available using the buffer's normal availability operation; this can invoke the registered load callback and can update end-of-input state. It returns `false` if availability still fails. Otherwise it checks the byte beginning at the cursor rounded up to the next byte boundary and returns `false` exactly when that byte and the next two bytes are `00 00 01`; it returns `true` for other byte sequences. The 40-bit guard makes those three bytes readable even for an unaligned cursor.

`BLOCK_SET(dest, dest_width, source_width, block_size, code)` writes a square `block_size`-by-`block_size` region to `dest`. The starting destination element offset must be available in the caller's `di` binding, and the code expression may use the caller's `si` binding and any other caller-visible values (the macro does not declare source storage). For each cell, the inserted expression is evaluated and converted by truncation to `u8`; then both offsets advance one element. At each row end, destination offset advances by `dest_width - block_size` and source offset by `source_width - block_size`, so both caller offsets finish positioned after the last processed row's span. Widths must be at least `block_size`, and the caller must provide valid source/destination ranges for every evaluated cell. No clipping, bounds checks, or rollback on an invalid range is promised.

`PUT_PIXEL(RI, GI, BI, y_offset, dest_offset)` is an expression-insertion macro used while converting a frame. It requires caller bindings `frame`, `y_index`, `dest`, `d_index`, and integer color contributions `r`, `g`, `b`. The luma value is the raw 8-bit sample, used directly in byte units; this macro does not subtract video-range black or apply a fixed-point scale. The caller-provided contributions are also integer byte-domain offsets: red is `clamp(y + r)`, green is `clamp(y - g)`, and blue is `clamp(y + b)`, where `y = frame.y.data[y_index + y_offset]` and `clamp` saturates to the `u8` range. It stores those values at `dest[d_index + dest_offset + RI/GI/BI]`. For example, a luma sample of 16 with `r = g = b = 0` writes `(16, 16, 16)` in the selected channel positions. The three channel indices must identify distinct in-range byte positions for the caller's chosen pixel format. It writes no fourth/alpha channel; conversion wrappers using four-byte pixels leave that byte untouched. The macro does not perform bounds checking, initialize padding/alpha, or alter the caller's offsets.
