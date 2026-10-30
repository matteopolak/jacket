// A native user interface library for jaic programs. A jaic extension, not official Jai: other
// Jai compilers have no such module, and it may change between jaic releases. See
// docs/extensions/ui.md.
//
// Windows, rendering and text are our own: the window comes from `Window_Creation`, drawing from
// `Extensions/WebGPU` (Metal, Vulkan or D3D12 natively, the page's WebGPU in the playground) and
// text from FreeType into a glyph atlas. The widget tree is retained and keyed but declared with
// plain calls every frame, so the app owns its data and the library owns scroll offsets, focus and
// cursors. Containers are `for` blocks.
//
// ```jai,no_run,skip_on(wasm)
// #import "Basic";
// #import "Extensions/UI";
//
// App :: struct { count: int; }
//
// main :: () {
//     app: App;
//     run_app(.{title = "Counter", width = 320, height = 200}, *app, build);
// }
//
// build :: (app: *App) {
//     for column(.{gap = 8, padding = 16}) {
//         label(tprint("Clicked % times", app.count));
//         if button("Click me") app.count += 1;
//     }
// }
// ```
//
// Tests drive the same `build` procedures without a window or a GPU (`headless`, `frame`, `click`,
// `layout_dump`).
#module_parameters(DEBUG_WARNINGS := true);

#import "Basic";
#import "Math";
#import "Hash_Table";
#import "Flat_Pool";
#import "File";
#import "Input";
#import "freetype";
#import "Extensions/Environment";
#import "Extensions/Text_Buffer";
#import "Extensions/WebGPU";
#import "Window_Type";
#import "Window_Creation";
#import "Clipboard";
#import "Extensions/Image";

#assert OS != .WASM "Extensions/UI needs a native window and GPU, and has no WebAssembly backend yet";

#load "types.jai";
#load "theme.jai";
#load "text.jai";
#load "input.jai";
#load "tree.jai";
#load "layout.jai";
#load "draw.jai";
#load "widgets.jai";
#load "text_edit.jai";
#load "headless.jai";
#load "render.jai";
#load "app.jai";
