// A baked struct parameter whose declared type names an earlier `$T` (`$help: Help(T)`), and a
// baked argument that is a member of a constant (`HELP.inner`).
#import "Basic";

Help :: struct(T: Type) {
    about: string;
    #insert -> string {
        info := cast(*Type_Info_Struct) type_info(T);
        builder: String_Builder;
        for info.members {
            if it.type.type == .STRUCT print(*builder, "%: Help(type_of(T.{}.%));\n", it.name, it.name);
        }
        return builder_to_string(*builder);
    };
}

Inner :: struct { x: int; }
Outer :: struct { inner: Inner; y: int; }

describe :: ($T: Type, $help: Help(T)) -> string {
    return help.about;
}

through :: ($T: Type, $help: Help(T)) -> string {
    return describe(type_of(T.{}.inner), help.inner);
}

HELP :: Help(Outer).{ about = "outer", inner = .{ about = "inner" } };

main :: () {
    assert(describe(Outer, HELP) == "outer");
    assert(through(Outer, HELP) == "inner");
    print("ok\n");
}
