// Independently authored runtime helpers. Native storage stays runtime-owned.
create_objc_class_member_stub :: (class_name: string, framework_name: string) -> string {
    return ObjCStub.create(class_name, framework_name);
}

objc_type_encode :: (builder: *String_Builder, info: *Type_Info, pointer_depth := 0) {
    ObjCEncoding.encode(builder, info, type_info(Class), type_info(id), type_info(Selector), pointer_depth);
}

objc_type_encode :: (info: *Type_Info) -> string {
    builder: String_Builder;
    defer ObjCBasic.free_buffers(*builder);
    objc_type_encode(*builder, info);
    return ObjCBasic.builder_to_string(*builder);
}

objc_func_encode :: (ty: *Type_Info_Procedure) -> string {
    return ObjCRegistration.method_encoding(ty, type_info(Class), type_info(id), type_info(Selector));
}

type_check_objc_method_type :: (ty: *Type_Info_Procedure) {
    ObjCEncoding.validate_method(ty, type_info(Class), type_info(id), type_info(Selector));
}

has_isa_member :: (ty: *Type_Info_Struct) -> bool {
    return ObjCEncoding.has_isa_member(ty, type_info(Class));
}

objc_create_class :: ($type: Type, $parent: Type) -> Class {
    return cast(Class) ObjCRegistration.create(type_info(type), type_info(parent), cast(*void) class(parent), type_info(Class), type_info(id), type_info(Selector));
}

objc_add_instance_method :: (my_class: Class, proc: Any, selector_name: string) {
    ObjCRegistration.install(cast(*void) my_class, proc, selector_name, type_info(Class), type_info(id), type_info(Selector), false);
}

objc_add_class_method :: (my_class: Class, proc: Any, selector_name: string) {
    ObjCRegistration.install(cast(*void) my_class, proc, selector_name, type_info(Class), type_info(id), type_info(Selector), true);
}

#scope_file
String_Builder :: ObjCBasic.String_Builder;

ObjCBasic :: #import "Basic";
ObjCEncoding :: #import "internal/bindings/objc-encoding";
ObjCRegistration :: #import "internal/bindings/objc-registration";
ObjCStub :: #import "internal/bindings/objc-stub";
