// An immediate-mode UI: widgets are procedure calls made every frame, each given the rectangle
// it draws in. Coordinates are in pixels with y growing upwards; GetRect_LeftHanded is the same
// module with y growing downwards.
//
// Each frame: pass input events to [getrect_handle_event], call [ui_per_frame_update], call widgets
// such as [button], [slider], [text_input] and [dropdown] with the [Rect] to draw in, then call
// [draw_popups] last. Lay rectangles out with [get_rect] and the `cut_` procedures. A widget keeps
// its state between frames under a hash of its call site and `identifier`, so give `identifier`
// a distinct value when one call site draws several widgets.
//
// [ui_init] sets the drawing procedures. Without [Draw_Procs] it draws with `Simp` when
// `Type_Indicator` is `void`; with any other `Type_Indicator`, pass them.

#module_parameters()(
    // `void` draws with `Simp`. To draw with something else, pass a type that has its own `Font` and texture procedures.
    Type_Indicator: Type = void
);

#load "../internal/numerics/ui_types.jai";
#load "../internal/numerics/ui_context.jai";
#load "../internal/numerics/ui_geometry.jai";
#load "../internal/numerics/ui_core.jai";
#load "../internal/numerics/ui_widgets.jai";
#load "../internal/numerics/ui_text.jai";
#load "../internal/numerics/ui_editor.jai";

#if OS == .WINDOWS {
    Windows :: #import "Windows";
}

#scope_module
LEFT_HANDED :: false;

#import "Basic";
#import "Math";
#import "Color_Interpolation";
#import "Input";
#import "Hash_Table";
