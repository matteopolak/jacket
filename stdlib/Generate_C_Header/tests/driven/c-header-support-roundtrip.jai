Env_Output :: #import "Extensions/Environment";
Compiler_Output :: #import "Compiler";
#import "Basic";

Support_User :: struct {
    text: string;
    view: [] u32;
    dynamic: [..] u8;
    boxed: Any;
    type: Type;
    callback: (value: s32) -> s32 #c_call;
}

#run Compiler_Output.compiler_add_library_search_directory(Env_Output.output_directory());

native :: #library "developer_tools_support";
support_sum :: (value: *Support_User) -> s64 #foreign native;

main :: () {
    value: Support_User;
    value.text = "four";
    view: [2] u32;

    value.view = view;
    array_add(*value.dynamic, 7);
    defer array_free(value.dynamic);
    number: s32 = 10;

    value.boxed = cast(Any) number;
    value.type = s32;
    assert(support_sum(*value) == 18);
    print("ok\n");
}
