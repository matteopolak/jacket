#import "Basic";
#import "Compiler";
Static :: struct(pattern: string) { IS_STATIC_REGEX :: true; }
f :: ($R: Type, text: string) -> string #modify {
    info := cast(*Type_Info) R;
    if info.type != .STRUCT return false, "not a Static";
    s := cast(*Type_Info_Struct) info;
    return s.polymorph_source_struct == type_info(Static), "not a Static";
} { return "ok"; }
main :: () { print("%\n", f(int, "x")); }
