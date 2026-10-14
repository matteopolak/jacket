#import "Basic";
S :: #import "Simp"(render_api = .NONE);

main :: () {
    #assert size_of(S.Texture_Platform_Specific) == 0;
    #assert size_of(S.Shader_Platform) == 0;
    #assert size_of(S.Pixel_Read_Platform) == 0;
    size, stride := S.get_image_size(.RGBA8, 2, 3);

    assert(size == 24 && stride == 8);
    print("ok\n");
}
