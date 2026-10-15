// Reviewed Objective_C signatures/isa requirement, canonical reflection and
// independent Apple native ABI. Underspecified Jai policies remain documented.
#import "Basic";
O :: #import "Objective_C";

libobjc :: #library,system "objc";
send_address :: () #foreign libobjc "objc_msgSend";

#if CPU == .X64 {
    send_stret_address :: () #foreign libobjc "objc_msgSend_stret";
}

probe :: #library,system "objc_runtime_probe";
native_encoding :: (text: *u8, length: u64, size: u64, alignment: u64) -> s32 #foreign probe "binding_objc_encoding_layout";
native_method :: (class: *void, selector: *u8) -> s32 #foreign probe "binding_objc_method_signature";

check_encoding :: (info: *Type_Info, expected: string, alignment: u64) {
    encoded := O.objc_type_encode(info);
    assert(encoded == expected);
    assert(native_encoding(encoded.data, cast(u64) encoded.count, cast(u64) info.runtime_size, alignment) != 0);
    context.allocator.proc(.FREE, 0, encoded.count, encoded.data, context.allocator.data);
}

JaiBindingRuntimeObject :: struct {
    using base: O.NSObject;
    tag: u8;
    value: s64;
}

Pair :: struct {
    x: s32;
    y: float64;
}

JaiBindingRuntimeOverlap :: struct {
    using base: O.NSObject;
    value: s64;
    #overlay (value) alias: s64;
}

answer :: (self: *JaiBindingRuntimeObject, selector: O.Selector, amount: s64) -> s64 #c_call {
    return self.value + amount;
}

class_answer :: (self: O.Class, selector: O.Selector) -> s64 #c_call {
    return 713;
}

bounds :: (self: *JaiBindingRuntimeObject, selector: O.Selector) -> O.NSRect #c_call {
    push_context, defer_pop;
    return O.NSMakeRect(1, 2, 3, 4);
}

main :: () {
    pool := O.objc_new(O.NSAutoreleasePool);

    assert(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    check_encoding(type_info(s64), "q", 8);
    check_encoding(type_info(u32), "I", 4);
    check_encoding(type_info(bool), "B", 1);
    check_encoding(type_info(*void), "^v", 8);
    check_encoding(type_info([3] float32), "[3f]", 4);
    check_encoding(type_info(Pair), "{Pair=id}", 8);
    builder: String_Builder;

    append(*builder, "prefix:");
    O.objc_type_encode(*builder, type_info(s16));
    appended := builder_to_string(*builder);

    assert(appended == "prefix:s");
    context.allocator.proc(.FREE, 0, appended.count, appended.data, context.allocator.data);
    free_buffers(*builder);
    O.type_check_objc_method_type(type_info(type_of(answer)));
    method_encoding := O.objc_func_encode(type_info(type_of(answer)));

    assert(method_encoding == "q@:q");
    context.allocator.proc(.FREE, 0, method_encoding.count, method_encoding.data, context.allocator.data);
    dynamic := O.objc_create_class(JaiBindingRuntimeObject, O.NSObject);

    assert(dynamic != null);
    assert(O.class_getInstanceSize(dynamic) == size_of(JaiBindingRuntimeObject));
    assert(O.ivar_getOffset(O.class_getInstanceVariable(dynamic, "tag")) == 8);
    assert(O.ivar_getOffset(O.class_getInstanceVariable(dynamic, "value")) == 16);
    O.objc_add_instance_method(dynamic, answer, "answer:");
    O.objc_add_class_method(dynamic, class_answer, "classAnswer");
    O.objc_add_instance_method(dynamic, bounds, "bounds");
    O.objc_finalize_class(dynamic);
    stub := O.create_objc_class_member_stub("JaiBindingRuntimeObject", "Foundation");

    assert(stub.count > "_jaibindingruntimeobject_data: [16] u8;".count);

    for i: 0.."_jaibindingruntimeobject_data: [16] u8;".count - 1 {
        assert(stub[i] == "_jaibindingruntimeobject_data: [16] u8;"[i]);
    }

    context.allocator.proc(.FREE, 0, stub.count, stub.data, context.allocator.data);
    assert(native_method(cast(*void) dynamic, "answer:") != 0);
    object := O.objc_new(JaiBindingRuntimeObject);

    assert(object != null && O.class(cast(O.id) object) == dynamic);
    object.tag = 91;
    object.value = 170;
    Send :: #type (self: *JaiBindingRuntimeObject, selector: O.Selector, amount: s64) -> s64 #c_call;
    send := cast(Send) send_address;

    assert(send(object, O.selector("answer:"), 9) == 179);
    assert(object.tag == 91 && object.value == 170);
    ClassSend :: #type (self: O.Class, selector: O.Selector) -> s64 #c_call;
    class_send := cast(ClassSend) send_address;

    assert(class_send(dynamic, O.selector("classAnswer")) == 713);
    BoundsSend :: #type (self: *JaiBindingRuntimeObject, selector: O.Selector) -> O.NSRect #c_call;
    bounds_send: BoundsSend;

    #if CPU == .X64 bounds_send = cast(BoundsSend) send_stret_address;
    else bounds_send = cast(BoundsSend) send_address;
    rectangle := bounds_send(object, O.selector("bounds"));

    assert(rectangle.origin.x == 1 && rectangle.origin.y == 2);
    assert(rectangle.size.width == 3 && rectangle.size.height == 4);
    O.release(object);
    // Independent failure safety: an unrepresentable overlay is rejected and
    // its partially allocated native class is disposed, releasing the name.
    assert(O.objc_create_class(JaiBindingRuntimeOverlap, O.NSObject) == null);
    recovered := O.objc_allocateClassPair(O.class(O.NSObject), type_info(JaiBindingRuntimeOverlap).name);

    assert(recovered != null);
    O.objc_disposeClassPair(recovered);
}
