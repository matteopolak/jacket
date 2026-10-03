#module_parameters(Type_Indicator: Type = void);

#load "../internal/numerics/ui_types_right.jai";
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
