// Generated from docs/specs/stdlib/modules/stb-image.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
FILE :: struct {};
stb_image :: #library,system "stb_image";
#scope_export

// Types and layouts

STBI :: enum u32 {
    default :: 0;

    grey :: 1;
    grey_alpha :: 2;
    rgb :: 3;
    rgb_alpha :: 4;

    STBI_default :: default;

    STBI_grey :: grey;
    STBI_grey_alpha :: grey_alpha;
    STBI_rgb :: rgb;
    STBI_rgb_alpha :: rgb_alpha;
}

stbi_io_callbacks :: struct {
    read: #type (user: *void, data: *u8, size: s32) -> s32 #c_call;
    skip: #type (user: *void, n: s32) -> void #c_call;
    eof: #type (user: *void) -> s32 #c_call;
}

// Aliases and constants

STBI_VERSION :: 1;

// Native entry points

stbi_convert_iphone_png_to_rgb :: (flag_true_if_should_convert: s32) -> void #foreign stb_image;

stbi_convert_iphone_png_to_rgb_thread :: (flag_true_if_should_convert: s32) -> void #foreign stb_image;

stbi_failure_reason :: () -> *u8 #foreign stb_image;

stbi_hdr_to_ldr_gamma :: (gamma: float) -> void #foreign stb_image;

stbi_hdr_to_ldr_scale :: (scale: float) -> void #foreign stb_image;

stbi_image_free :: (retval_from_stbi_load: *void) -> void #foreign stb_image;

stbi_info :: (filename: *u8, x: *s32, y: *s32, comp: *s32) -> s32 #foreign stb_image;

stbi_info_from_callbacks :: (clbk: *stbi_io_callbacks, user: *void, x: *s32, y: *s32, comp: *s32) -> s32 #foreign stb_image;

stbi_info_from_file :: (f: *FILE, x: *s32, y: *s32, comp: *s32) -> s32 #foreign stb_image;

stbi_info_from_memory :: (buffer: *u8, len: s32, x: *s32, y: *s32, comp: *s32) -> s32 #foreign stb_image;

stbi_is_16_bit :: (filename: *u8) -> s32 #foreign stb_image;

stbi_is_16_bit_from_callbacks :: (clbk: *stbi_io_callbacks, user: *void) -> s32 #foreign stb_image;

stbi_is_16_bit_from_file :: (f: *FILE) -> s32 #foreign stb_image;

stbi_is_16_bit_from_memory :: (buffer: *u8, len: s32) -> s32 #foreign stb_image;

stbi_is_hdr :: (filename: *u8) -> s32 #foreign stb_image;

stbi_is_hdr_from_callbacks :: (clbk: *stbi_io_callbacks, user: *void) -> s32 #foreign stb_image;

stbi_is_hdr_from_file :: (f: *FILE) -> s32 #foreign stb_image;

stbi_is_hdr_from_memory :: (buffer: *u8, len: s32) -> s32 #foreign stb_image;

stbi_ldr_to_hdr_gamma :: (gamma: float) -> void #foreign stb_image;

stbi_ldr_to_hdr_scale :: (scale: float) -> void #foreign stb_image;

stbi_load :: (filename: *u8, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *u8 #foreign stb_image;

stbi_load_16 :: (filename: *u8, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *u16 #foreign stb_image;

stbi_load_16_from_callbacks :: (clbk: *stbi_io_callbacks, user: *void, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *u16 #foreign stb_image;

stbi_load_16_from_memory :: (buffer: *u8, len: s32, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *u16 #foreign stb_image;

stbi_load_from_callbacks :: (clbk: *stbi_io_callbacks, user: *void, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *u8 #foreign stb_image;

stbi_load_from_file :: (f: *FILE, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *u8 #foreign stb_image;

stbi_load_from_file_16 :: (f: *FILE, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *u16 #foreign stb_image;

stbi_load_from_memory :: (buffer: *u8, len: s32, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *u8 #foreign stb_image;

stbi_load_gif_from_memory :: (buffer: *u8, len: s32, delays: **s32, x: *s32, y: *s32, z: *s32, comp: *s32, req_comp: s32) -> *u8 #foreign stb_image;

stbi_loadf :: (filename: *u8, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *float #foreign stb_image;

stbi_loadf_from_callbacks :: (clbk: *stbi_io_callbacks, user: *void, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *float #foreign stb_image;

stbi_loadf_from_file :: (f: *FILE, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *float #foreign stb_image;

stbi_loadf_from_memory :: (buffer: *u8, len: s32, x: *s32, y: *s32, channels_in_file: *s32, desired_channels: s32) -> *float #foreign stb_image;

stbi_set_flip_vertically_on_load :: (flag_true_if_should_flip: s32) -> void #foreign stb_image;

stbi_set_flip_vertically_on_load_thread :: (flag_true_if_should_flip: s32) -> void #foreign stb_image;

stbi_set_unpremultiply_on_load :: (flag_true_if_should_unpremultiply: s32) -> void #foreign stb_image;

stbi_set_unpremultiply_on_load_thread :: (flag_true_if_should_unpremultiply: s32) -> void #foreign stb_image;

stbi_zlib_decode_buffer :: (obuffer: *u8, olen: s32, ibuffer: *u8, ilen: s32) -> s32 #foreign stb_image;

stbi_zlib_decode_malloc :: (buffer: *u8, len: s32, outlen: *s32) -> *u8 #foreign stb_image;

stbi_zlib_decode_malloc_guesssize :: (buffer: *u8, len: s32, initial_size: s32, outlen: *s32) -> *u8 #foreign stb_image;

stbi_zlib_decode_malloc_guesssize_headerflag :: (buffer: *u8, len: s32, initial_size: s32, outlen: *s32, parse_header: s32) -> *u8 #foreign stb_image;

stbi_zlib_decode_noheader_buffer :: (obuffer: *u8, olen: s32, ibuffer: *u8, ilen: s32) -> s32 #foreign stb_image;

stbi_zlib_decode_noheader_malloc :: (buffer: *u8, len: s32, outlen: *s32) -> *u8 #foreign stb_image;
