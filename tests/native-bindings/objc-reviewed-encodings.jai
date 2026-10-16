// Oracle: reviewed Objective_C.md Type encoding and method registration.
// Tests exact public text independently of native C spelling preferences.
#import "Basic";
O :: #import "Objective_C";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (condition: bool) {
    if !condition abort();
}

release_text :: (text: string) {
    if text.data context.allocator.proc(.FREE, 0, text.count, text.data, context.allocator.data);
}

encoding :: (info: *Type_Info, expected: string) {
    actual := O.objc_type_encode(info);
    defer release_text(actual);
    if actual != expected log_error("Expected encoding %, received %", expected, actual);
    check(actual == expected);
}

Simple :: struct {
    value: s32;
}

Header :: struct {
    isa: O.Class;
}

Node :: struct {
    value: s32;
    next: *Node;
}

UsingRecord :: struct {
    using inherited: Simple;
    tag: u16;
    constant :: 13;
}

plain :: (amount: s32) -> s64 {
    return amount;
}

method :: (self: O.id, selector: O.Selector, amount: s32) -> s64 #c_call {
    return amount;
}

pointer_receiver :: (self: *Simple, selector: O.Selector) #c_call {}
record_receiver :: (self: Header, selector: O.Selector) #c_call {}
empty :: () {}

main :: () {
    encoding(type_info(s32), "l");
    encoding(type_info(u32), "L");
    encoding(type_info(s8), "c");
    encoding(type_info(u8), "C");
    encoding(type_info(s16), "s");
    encoding(type_info(u16), "S");
    encoding(type_info(s64), "q");
    encoding(type_info(u64), "Q");
    encoding(type_info(float32), "f");
    encoding(type_info(float64), "d");
    encoding(type_info(bool), "B");
    encoding(type_info(O.Class), "#");
    encoding(type_info(O.id), "@");
    encoding(type_info(O.Selector), ":");
    check(type_info(O.Selector) == type_info(*void));
    encoding(type_info(*void), ":");
    encoding(type_info(*u8), "*");
    encoding(type_info(*s8), "*");
    encoding(type_info(*s32), "^l");
    encoding(type_info([3] s32), "[3l]");
    encoding(type_info([] float64), "{_static=q^c}");
    encoding(type_info([..] s16), "{_dynamic=qq^v?^v}");
    encoding(type_info(string), "{string=q^c}");
    encoding(type_info(type_of(plain)), "?");
    encoding(type_info(Simple), "{Simple=l}");
    encoding(type_info(Header), "@");
    encoding(type_info(*Header), "^@");
    encoding(type_info(UsingRecord), "{UsingRecord=S}");
    encoding(type_info(Node), "{Node=l^{Node=l^{Node}}}");
    check(O.has_isa_member(type_info(Header)));
    check(!O.has_isa_member(type_info(Simple)));

    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "prefix:");
    O.objc_type_encode(*builder, type_info(Node), 2);
    text := builder_to_string(*builder);

    check(text == "prefix:{Node}");
    release_text(text);
    O.objc_type_encode(*builder, type_info([1] Node), 2);
    text = builder_to_string(*builder);
    check(text == "[1{Node=l^{Node=l^{Node}}}]");
    release_text(text);

    O.type_check_objc_method_type(type_info(type_of(method)));
    O.type_check_objc_method_type(type_info(type_of(pointer_receiver)));
    O.type_check_objc_method_type(type_info(type_of(record_receiver)));
    text = O.objc_func_encode(type_info(type_of(plain)));
    check(text == "ql");
    release_text(text);
    text = O.objc_func_encode(type_info(type_of(method)));
    check(text == "q@:l");
    release_text(text);
    text = O.objc_func_encode(type_info(type_of(pointer_receiver)));
    check(text == "v^{Simple=l}:");
    release_text(text);
    text = O.objc_func_encode(type_info(type_of(record_receiver)));
    check(text == "v@:");
    release_text(text);
    text = O.objc_func_encode(type_info(type_of(empty)));
    check(text == "v");
    release_text(text);
}
