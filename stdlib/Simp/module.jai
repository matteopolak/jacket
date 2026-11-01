// A small 2D renderer on OpenGL: coloured shapes, textures, text and render targets, drawn through
// immediate-mode calls.
//
// Make a window with `Window_Creation`, choose it with `set_render_target`, then each frame call
// `update_window`, `clear_render_target`, draw, and `swap_buffers`. Pick a shader first
// (`set_shader_for_color` or `set_shader_for_images`), then draw with `immediate_quad`, or send
// single corners with `immediate_vertex`. `load_font` and `draw_text` put text on screen. Textures come
// from `texture_load_from_file` or `texture_load_from_memory`.
//
// The default coordinate system has its origin at the bottom left with y growing up; pass
// `.LEFT_HANDED` to `set_render_target` for the top left with y growing down.
//
// ```jai,no_run
// #import "Window_Creation";
// #import "Input";
//
// window := create_window(800, 600, "simp");
// set_render_target(window);
// font := load_font("", "Roboto-Regular.ttf", 24);
//
// while true {
//     update_window_events();
//     for events_this_frame if it.type == .QUIT return;
//     update_window(window);
//
//     clear_render_target(0.1, 0.1, 0.15, 1);
//     set_shader_for_color();
//     immediate_quad(100, 100, 300, 200, .{0.9, 0.3, 0.2, 1});
//     draw_text(font, 100, 300, "Hello");
//     swap_buffers(window);
// }
// ```

// test: skip_on(.WASM, reason = "needs OpenGL, which has no wasm target")

#module_parameters (
    // The graphics API to draw with. `.NONE` draws nothing.
    render_api := Render_API.OPENGL
)() {
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
