#module_parameters (render_api := Render_API.OPENGL)() {
    Render_API :: enum u32 {
        NONE;
        SOFTWARE;
        OPENGL;
        METAL;
        NVN;
    }
}

#load "image-types.jai";
#load "loaded-font-types.jai";
#load "types.jai";
#load "platform-types.jai";
#load "../internal/rendering/images.jai";
#load "../internal/rendering/mipmaps.jai";
#load "../internal/rendering/image-codecs.jai";
#load "../internal/rendering/loaded-fonts.jai";
#load "../internal/rendering/glyph-atlas.jai";
#load "../internal/rendering/dynamic-fonts.jai";
#load "../internal/rendering/font-features.jai";

#if render_api == .OPENGL {
    #load "../internal/rendering/gl-textures.jai";
    #load "../internal/rendering/gl-shaders.jai";
    #load "../internal/rendering/gl-renderer.jai";
    #load "../internal/rendering/immediate-geometry.jai";
    #load "../internal/rendering/gl-pixel-read.jai";
}

window_infos: [..] Window_Info;

#scope_module
#import "Math";
#import "Hash_Table";
#import "freetype";
Basic :: #import "Basic";
Unicode :: #import "Unicode";

assert :: Basic.assert;

Image :: #import "stb_image";
Image_Write :: #import "stb_image_write";
Resize :: #import "stb_image_resize";

#if render_api == .OPENGL {
    GL :: #import "GL";
}

File :: #import "File";

Window_Type :: WT.Window_Type;

WT :: #import "Window_Type";

GLuint :: u32;
GLint :: s32;
