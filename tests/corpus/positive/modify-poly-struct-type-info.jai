#import "Basic";
Static :: struct(pattern: string) { IS_STATIC_REGEX :: true; }
f :: ($R: Type, text: string) -> string #modify {
    info := cast(*Type_Info) R;
    if info.type != .STRUCT return false, "not a Static";
    s := cast(*Type_Info_Struct) info;
    return s.polymorph_source_struct == type_info(Static), "not a Static";
} { return R.pattern; }
D :: Static("ab");
main :: () {
    print("%\n", f(D, "x"));
    print("%\n", f(Static("cd"), "x"));
    s := cast(*Type_Info_Struct) type_info(D);
    print("%\n", s.polymorph_source_struct == type_info(Static));
}
