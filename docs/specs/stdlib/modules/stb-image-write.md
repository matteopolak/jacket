# stb_image_write


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `stbi_write_png_to_mem :: (pixels: *u8, stride_bytes: s32, x: s32, y: s32, n: s32, out_len: *s32) -> *u8 #foreign stb_image_write`
- `stbi_zlib_compress :: (data: *u8, data_len: s32, out_len: *s32, quality: s32) -> *u8 #foreign stb_image_write`
- `stbi_write_tga_with_rle: s32 #elsewhere stb_image_write`
- `stbi_write_png_compression_level: s32 #elsewhere stb_image_write`
- `stbi_write_force_png_filter: s32 #elsewhere stb_image_write`
- `stbi_write_png :: (filename: *u8, w: s32, h: s32, comp: s32, data: *void, stride_in_bytes: s32) -> s32 #foreign stb_image_write`
- `stbi_write_bmp :: (filename: *u8, w: s32, h: s32, comp: s32, data: *void) -> s32 #foreign stb_image_write`
- `stbi_write_tga :: (filename: *u8, w: s32, h: s32, comp: s32, data: *void) -> s32 #foreign stb_image_write`
- `stbi_write_hdr :: (filename: *u8, w: s32, h: s32, comp: s32, data: *float) -> s32 #foreign stb_image_write`
- `stbi_write_jpg :: (filename: *u8, x: s32, y: s32, comp: s32, data: *void, quality: s32) -> s32 #foreign stb_image_write`
- `stbi_write_func :: #type (_context: *void, data: *void, size: s32) -> void #c_call`
- `stbi_write_png_to_func :: (func: stbi_write_func, _context: *void, w: s32, h: s32, comp: s32, data: *void, stride_in_bytes: s32) -> s32 #foreign stb_image_write`
- `stbi_write_bmp_to_func :: (func: stbi_write_func, _context: *void, w: s32, h: s32, comp: s32, data: *void) -> s32 #foreign stb_image_write`
- `stbi_write_tga_to_func :: (func: stbi_write_func, _context: *void, w: s32, h: s32, comp: s32, data: *void) -> s32 #foreign stb_image_write`
- `stbi_write_hdr_to_func :: (func: stbi_write_func, _context: *void, w: s32, h: s32, comp: s32, data: *float) -> s32 #foreign stb_image_write`
- `stbi_write_jpg_to_func :: (func: stbi_write_func, _context: *void, x: s32, y: s32, comp: s32, data: *void, quality: s32) -> s32 #foreign stb_image_write`
- `stbi_flip_vertically_on_write :: (flip_boolean: s32) -> void #foreign stb_image_write`


## Contracts and constraints

This module exposes stb image-writing functions through generated C bindings. Encoders consume caller-provided pixel buffers and dimensions/stride/channel counts; the caller must provide valid storage for the full image. The PNG-to-memory and zlib helpers return allocated C buffers with lengths through output pointers, so release them using the matching stb free routine where exposed. Build generation and native library availability depend on target-specific Bindings_Generator/BuildCpp configuration.
