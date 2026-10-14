#import "Basic";
S :: #import "Simp";
GL :: #import "GL";

// Exercise the exact private staging/format code without substituting a GL driver.
// Actual GPU execution remains a separate platform integration requirement.
#load "../../stdlib/internal/rendering/gl-textures.jai";

main :: () {
    source: [48] u8;
    for i: 0..47 source[i] = cast(u8) i;

    bitmap: S.Bitmap;
    bitmap.width = 3;
    bitmap.height = 2;
    bitmap.stride = 16;
    bitmap.format = .RGBA8;
    bitmap.data = .{ 32, source.data };
    packed, ok := pack_bitmap(*bitmap);

    assert(ok && packed.count == 24);
    for i: 0..11 assert(packed[i] == source[i] && packed[12 + i] == source[16 + i]);

    free(packed.data);
    bitmap.data.count = 20;
    packed, ok = pack_bitmap(*bitmap);
    assert(!ok && packed.data == null);
    bitmap.width = 7;
    bitmap.height = 5;
    bitmap.stride = 24;
    bitmap.format = .BC1;
    bitmap.data.count = 48;
    packed, ok = pack_bitmap(*bitmap);
    assert(ok && packed.count == 32);
    for i: 0..15 assert(packed[i] == source[i] && packed[16 + i] == source[24 + i]);

    free(packed.data);
    description, known := describe_gl_format(.RGBA8, true);

    assert(known && description.internal == GL.GL_SRGB8_ALPHA8);
    description, known = describe_gl_format(.R8, true);
    assert(!known);
    description, known = describe_gl_format(.Unknown, false);
    assert(!known);
    empty: S.Texture;

    S.texture_destroy(*empty);
    assert(empty.gl_handle == 0 && empty.width == 1);
    print("ok\n");
}

#scope_module
Texture :: S.Texture;
Texture_Format :: S.Texture_Format;
Texture_Flags :: S.Texture_Flags;
Bitmap :: S.Bitmap;
GLuint :: u32;

Basic :: #import "Basic";

bitmap_load :: S.bitmap_load;
bitmap_load_from_memory :: S.bitmap_load_from_memory;
deinit :: S.deinit;
get_image_size :: S.get_image_size;
get_image_bytes_per_texel :: S.get_image_bytes_per_texel;
