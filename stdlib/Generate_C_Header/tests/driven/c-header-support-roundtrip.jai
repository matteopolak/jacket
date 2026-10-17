#import "Basic";

Support_User :: struct {
    text: string;
    view: [] u32;
    dynamic: [..] u8;
    boxed: Any;
    type: Type;
    callback: (value: s32) -> s32 #c_call;
}

native :: #foreign_library "../../../../target/developer_tools_support";
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
