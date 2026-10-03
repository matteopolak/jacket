// Generated from docs/specs/stdlib/modules/stb-image-write.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file

stb_image_write :: #library,system "stb_image_write";
#scope_export

// Callback and opaque types

stbi_write_func :: #type (_context: *void, data: *void, size: s32) -> void #c_call;

// Native storage

stbi_write_tga_with_rle: s32 #elsewhere stb_image_write;

stbi_write_png_compression_level: s32 #elsewhere stb_image_write;

stbi_write_force_png_filter: s32 #elsewhere stb_image_write;

// Native entry points

stbi_flip_vertically_on_write :: (flip_boolean: s32) -> void #foreign stb_image_write;

stbi_write_bmp :: (filename: *u8, w: s32, h: s32, comp: s32, data: *void) -> s32 #foreign stb_image_write;

stbi_write_bmp_to_func :: (func: stbi_write_func, _context: *void, w: s32, h: s32, comp: s32, data: *void) -> s32 #foreign stb_image_write;

stbi_write_hdr :: (filename: *u8, w: s32, h: s32, comp: s32, data: *float) -> s32 #foreign stb_image_write;

stbi_write_hdr_to_func :: (func: stbi_write_func, _context: *void, w: s32, h: s32, comp: s32, data: *float) -> s32 #foreign stb_image_write;

stbi_write_jpg :: (filename: *u8, x: s32, y: s32, comp: s32, data: *void, quality: s32) -> s32 #foreign stb_image_write;

stbi_write_jpg_to_func :: (func: stbi_write_func, _context: *void, x: s32, y: s32, comp: s32, data: *void, quality: s32) -> s32 #foreign stb_image_write;

stbi_write_png :: (filename: *u8, w: s32, h: s32, comp: s32, data: *void, stride_in_bytes: s32) -> s32 #foreign stb_image_write;

stbi_write_png_to_func :: (func: stbi_write_func, _context: *void, w: s32, h: s32, comp: s32, data: *void, stride_in_bytes: s32) -> s32 #foreign stb_image_write;

stbi_write_png_to_mem :: (pixels: *u8, stride_bytes: s32, x: s32, y: s32, n: s32, out_len: *s32) -> *u8 #foreign stb_image_write;

stbi_write_tga :: (filename: *u8, w: s32, h: s32, comp: s32, data: *void) -> s32 #foreign stb_image_write;

stbi_write_tga_to_func :: (func: stbi_write_func, _context: *void, w: s32, h: s32, comp: s32, data: *void) -> s32 #foreign stb_image_write;

stbi_zlib_compress :: (data: *u8, data_len: s32, out_len: *s32, quality: s32) -> *u8 #foreign stb_image_write;
