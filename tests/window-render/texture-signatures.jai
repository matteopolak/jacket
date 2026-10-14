#import "Basic";
S :: #import "Simp";

main :: () {
    upload: #type (*S.Texture, *S.Bitmap) -> bool = S.texture_load_from_bitmap;
    release: #type (*S.Texture) = S.texture_destroy;
    allocate: #type (s32, s32, S.Texture_Format, S.Texture_Flags) -> S.Texture = S.texture_create_render_target;
    update: #type (*S.Texture, *S.Bitmap) = S.backend_update_texture;
    supports: #type (S.Texture_Format) -> bool = S.is_supported;
    file: #type (*S.Texture, string, bool, bool) -> bool = S.texture_load_from_file;
    memory: #type (*S.Texture, [] u8, bool, bool) -> bool = S.texture_load_from_memory;

    assert(upload != null && release != null && allocate != null && update != null && supports != null && file != null && memory != null);
    print("ok\n");
}
