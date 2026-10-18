# Objective_C


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `init_objective_c :: ()`
- `init_objective_c_selector_struct :: (selectors: *$T)`
- [OS == .MACOS] `BOOL :: s8`
- [else branch] `BOOL :: bool`
- `YES: BOOL : 1`
- `NO: BOOL : 0`
- `NSUInteger :: u64`
- `NSUIntegerMax :: 0xFFFFFFFFFFFFFFFF`
- `NSInteger :: s64`
- `Selector :: *void`
- `dispatch_queue_t :: *NSObject`
- `dispatch_data_t :: *NSObject`
- `to_string :: (str: *NSString) -> string`
- `objc_alloc :: (class: Class) -> id #no_context`
- `objc_copy :: (self: *$instancetype) -> *instancetype #no_context`
- `autorelease :: (self: *$instancetype) -> id #no_context`
- `release :: (self: *$instancetype) -> id #no_context @Releases`
- `retain :: (self: *$instancetype) -> *instancetype #no_context`
- `class :: (self: id) -> Class #no_context`
- `superclass :: (self: id) -> Class #no_context`
- `isEqual :: (self: id, object: id) -> bool #no_context`
- `objc_init :: (self: *$type) -> *type #no_context`
- `objc_alloc :: ($type: Type) -> *type #no_context`
- `objc_new :: ($type: Type) -> *type #no_context`
- `class :: ($type: Type) -> Class #no_context`
- `objc_lookUpClass :: (name: string) -> Class`
- `objc_getClass :: (name: string) -> Class`
- `objc_allocateClassPair :: (super: Class, name: string, extra_bytes: u64 = 0) -> Class`
- `objc_getProtocol :: (name: string) -> *Protocol`
- `sel_registerName :: (name: string) -> Selector`
- `sel_getUid :: (name: string) -> Selector`
- `selector :: (sel_str: string) -> Selector`
- `create_objc_class_member_stub :: (class_name: string, framework_name: string) -> string`
- `objc_create_class :: ($type: Type, $parent: Type) -> Class`
- `objc_add_instance_method :: (my_class: Class, proc: Any, selector_name: string)`
- `objc_add_class_method :: (my_class: Class, proc: Any, selector_name: string)`
- `objc_finalize_class :: (my_class: Class)`
- `objc_type_encode :: (builder: *String_Builder, info: *Type_Info, pointer_depth := 0)`
- `objc_type_encode :: (info: *Type_Info) -> string`
- `objc_func_encode :: (ty: *Type_Info_Procedure) -> string`
- `has_isa_member :: (ty: *Type_Info_Struct) -> bool`
- `type_check_objc_method_type :: (ty: *Type_Info_Procedure)`
- `Objective_C_Block_Basic_Descriptor :: struct`
- `Objective_C_Block_Basic :: struct`
- `_NSConcreteStackBlock :: () #foreign libSystem`
- `_NSConcreteGlobalBlock :: () #foreign libSystem`
- [else branch] `objc_msgSend_stret :: objc_msgSend`
- [else branch] `objc_msgSendSuper_stret :: objc_msgSendSuper`
- [else branch] `method_invoke_stret :: method_invoke`
- [else branch] `_objc_msgForward_stret :: _objc_msgForward`
- [else branch] `class_getMethodImplementation_stret :: class_getMethodImplementation`
- `objc_msgSend_typed :: inline (target: *void, selector: Selector) -> *void #no_context`

On non-x64 targets, the `*_stret` names above are aliases of their corresponding non-`stret` entry points and inherit those callable signatures. On x64, the dedicated `*_stret` declarations instead return `void` and use the platform structure-return calling convention.
- [CPU == .X64] `OBJC_BOOL_IS_BOOL :: 0`
- [CPU == .X64] `OBJC_BOOL_IS_CHAR :: 1`
- [CPU == .X64] `ARITH_SHIFT :: 32`
- [CPU == .X64] `CLS_CLASS :: 0x1`
- [CPU == .X64] `CLS_META :: 0x2`
- [CPU == .X64] `CLS_INITIALIZED :: 0x4`
- [CPU == .X64] `CLS_POSING :: 0x8`
- [CPU == .X64] `CLS_MAPPED :: 0x10`
- [CPU == .X64] `CLS_FLUSH_CACHE :: 0x20`
- [CPU == .X64] `CLS_GROW_CACHE :: 0x40`
- [CPU == .X64] `CLS_NEED_BIND :: 0x80`
- [CPU == .X64] `CLS_METHOD_ARRAY :: 0x100`
- [CPU == .X64] `CLS_JAVA_HYBRID :: 0x200`
- [CPU == .X64] `CLS_JAVA_CLASS :: 0x400`
- [CPU == .X64] `CLS_INITIALIZING :: 0x800`
- [CPU == .X64] `CLS_FROM_BUNDLE :: 0x1000`
- [CPU == .X64] `CLS_HAS_CXX_STRUCTORS :: 0x2000`
- [CPU == .X64] `CLS_NO_METHOD_ARRAY :: 0x4000`
- [CPU == .X64] `CLS_HAS_LOAD_METHOD :: 0x8000`
- [CPU == .X64] `CLS_CONSTRUCTING :: 0x10000`
- [CPU == .X64] `CLS_EXT :: 0x20000`
- [CPU == .X64] `OBSOLETE_OBJC_GETCLASSES :: 1`
- [CPU == .X64] `OBJC_NEXT_METHOD_LIST :: 1`
- [CPU == .X64] `Class :: *objc_class`
- [CPU == .X64] `NSObject :: struct`
- [CPU == .X64] `id :: *NSObject`
- [CPU == .X64] `objc_selector :: struct`
- [CPU == .X64] `SEL :: *objc_selector`
- [CPU == .X64] `IMP :: #type (unknown0: id, unknown1: SEL, __args: ..Any) -> id #c_call`
- [CPU == .X64] `sel_getName :: (sel: SEL) -> *u8 #foreign libobjc`
- [CPU == .X64] `sel_registerName :: (str: *u8) -> SEL #foreign libobjc`
- [CPU == .X64] `object_getClassName :: (obj: id) -> *u8 #foreign libobjc`
- [CPU == .X64] `object_getIndexedIvars :: (obj: id) -> *void #foreign libobjc`
- [CPU == .X64] `sel_isMapped :: (sel: SEL) -> BOOL #foreign libobjc`
- [CPU == .X64] `sel_getUid :: (str: *u8) -> SEL #foreign libobjc`
- [CPU == .X64] `objc_objectptr_t :: *void`
- [CPU == .X64] `objc_retainedObject :: (obj: objc_objectptr_t) -> id #foreign libobjc`
- [CPU == .X64] `objc_unretainedObject :: (obj: objc_objectptr_t) -> id #foreign libobjc`
- [CPU == .X64] `objc_unretainedPointer :: (obj: id) -> objc_objectptr_t #foreign libobjc`
- [CPU == .X64] `arith_t :: s64`
- [CPU == .X64] `uarith_t :: u64`
- [CPU == .X64] `STR :: *u8`
- [CPU == .X64] `Method :: *objc_method`
- [CPU == .X64] `Ivar :: *objc_ivar`
- [CPU == .X64] `Category :: *objc_category`
- [CPU == .X64] `objc_property :: struct`
- [CPU == .X64] `objc_property_t :: *objc_property`
- [CPU == .X64] `objc_class :: struct`
- [CPU == .X64] `Protocol :: NSObject`
- [CPU == .X64] `objc_method_description :: struct`
- [CPU == .X64] `objc_property_attribute_t :: struct`
- [CPU == .X64] `object_copy :: (obj: id, size: u64) -> id #foreign libobjc`
- [CPU == .X64] `object_dispose :: (obj: id) -> id #foreign libobjc`
- [CPU == .X64] `object_getClass :: (obj: id) -> Class #foreign libobjc`
- [CPU == .X64] `object_setClass :: (obj: id, cls: Class) -> Class #foreign libobjc`
- [CPU == .X64] `object_isClass :: (obj: id) -> BOOL #foreign libobjc`
- [CPU == .X64] `object_getIvar :: (obj: id, ivar: Ivar) -> id #foreign libobjc`
- [CPU == .X64] `object_setIvar :: (obj: id, ivar: Ivar, value: id) -> void #foreign libobjc`
- [CPU == .X64] `object_setIvarWithStrongDefault :: (obj: id, ivar: Ivar, value: id) -> void #foreign libobjc`
- [CPU == .X64] `object_setInstanceVariable :: (obj: id, name: *u8, value: *void) -> Ivar #foreign libobjc`
- [CPU == .X64] `object_setInstanceVariableWithStrongDefault :: (obj: id, name: *u8, value: *void) -> Ivar #foreign libobjc`
- [CPU == .X64] `object_getInstanceVariable :: (obj: id, name: *u8, outValue: **void) -> Ivar #foreign libobjc`
- [CPU == .X64] `objc_getClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .X64] `objc_getMetaClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .X64] `objc_lookUpClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .X64] `objc_getRequiredClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .X64] `objc_getClassList :: (buffer: *Class, bufferCount: s32) -> s32 #foreign libobjc`
- [CPU == .X64] `objc_copyClassList :: (outCount: *u32) -> *Class #foreign libobjc`
- [CPU == .X64] `class_getName :: (cls: Class) -> *u8 #foreign libobjc`
- [CPU == .X64] `class_isMetaClass :: (cls: Class) -> BOOL #foreign libobjc`
- [CPU == .X64] `class_getSuperclass :: (cls: Class) -> Class #foreign libobjc`
- [CPU == .X64] `class_setSuperclass :: (cls: Class, newSuper: Class) -> Class #foreign libobjc`
- [CPU == .X64] `class_getVersion :: (cls: Class) -> s32 #foreign libobjc`
- [CPU == .X64] `class_setVersion :: (cls: Class, version: s32) -> void #foreign libobjc`
- [CPU == .X64] `class_getInstanceSize :: (cls: Class) -> u64 #foreign libobjc`
- [CPU == .X64] `class_getInstanceVariable :: (cls: Class, name: *u8) -> Ivar #foreign libobjc`
- [CPU == .X64] `class_getClassVariable :: (cls: Class, name: *u8) -> Ivar #foreign libobjc`
- [CPU == .X64] `class_copyIvarList :: (cls: Class, outCount: *u32) -> *Ivar #foreign libobjc`
- [CPU == .X64] `class_getInstanceMethod :: (cls: Class, name: SEL) -> Method #foreign libobjc`
- [CPU == .X64] `class_getClassMethod :: (cls: Class, name: SEL) -> Method #foreign libobjc`
- [CPU == .X64] `class_getMethodImplementation :: (cls: Class, name: SEL) -> IMP #foreign libobjc`
- [CPU == .X64] `class_getMethodImplementation_stret :: (cls: Class, name: SEL) -> IMP #foreign libobjc`
- [CPU == .X64] `class_respondsToSelector :: (cls: Class, sel: SEL) -> BOOL #foreign libobjc`
- [CPU == .X64] `class_copyMethodList :: (cls: Class, outCount: *u32) -> *Method #foreign libobjc`
- [CPU == .X64] `class_conformsToProtocol :: (cls: Class, protocol: *Protocol) -> BOOL #foreign libobjc`
- [CPU == .X64] `class_copyProtocolList :: (cls: Class, outCount: *u32) -> **Protocol #foreign libobjc`
- [CPU == .X64] `class_getProperty :: (cls: Class, name: *u8) -> objc_property_t #foreign libobjc`
- [CPU == .X64] `class_copyPropertyList :: (cls: Class, outCount: *u32) -> *objc_property_t #foreign libobjc`
- [CPU == .X64] `class_getIvarLayout :: (cls: Class) -> *u8 #foreign libobjc`
- [CPU == .X64] `class_getWeakIvarLayout :: (cls: Class) -> *u8 #foreign libobjc`
- [CPU == .X64] `class_addMethod :: (cls: Class, name: SEL, imp: IMP, types: *u8) -> BOOL #foreign libobjc`
- [CPU == .X64] `class_replaceMethod :: (cls: Class, name: SEL, imp: IMP, types: *u8) -> IMP #foreign libobjc`
- [CPU == .X64] `class_addIvar :: (cls: Class, name: *u8, size: u64, alignment: u8, types: *u8) -> BOOL #foreign libobjc`
- [CPU == .X64] `class_addProtocol :: (cls: Class, protocol: *Protocol) -> BOOL #foreign libobjc`
- [CPU == .X64] `class_addProperty :: (cls: Class, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32) -> BOOL #foreign libobjc`
- [CPU == .X64] `class_replaceProperty :: (cls: Class, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32) -> void #foreign libobjc`
- [CPU == .X64] `class_setIvarLayout :: (cls: Class, layout: *u8) -> void #foreign libobjc`
- [CPU == .X64] `class_setWeakIvarLayout :: (cls: Class, layout: *u8) -> void #foreign libobjc`
- [CPU == .X64] `objc_getFutureClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .X64] `class_createInstance :: (cls: Class, extraBytes: u64) -> id #foreign libobjc`
- [CPU == .X64] `objc_constructInstance :: (cls: Class, bytes: *void) -> id #foreign libobjc`
- [CPU == .X64] `objc_destructInstance :: (obj: id) -> *void #foreign libobjc`
- [CPU == .X64] `objc_allocateClassPair :: (superclass: Class, name: *u8, extraBytes: u64) -> Class #foreign libobjc`
- [CPU == .X64] `objc_registerClassPair :: (cls: Class) -> void #foreign libobjc`
- [CPU == .X64] `objc_duplicateClass :: (original: Class, name: *u8, extraBytes: u64) -> Class #foreign libobjc`
- [CPU == .X64] `objc_disposeClassPair :: (cls: Class) -> void #foreign libobjc`
- [CPU == .X64] `method_getName :: (m: Method) -> SEL #foreign libobjc`
- [CPU == .X64] `method_getImplementation :: (m: Method) -> IMP #foreign libobjc`
- [CPU == .X64] `method_getTypeEncoding :: (m: Method) -> *u8 #foreign libobjc`
- [CPU == .X64] `method_getNumberOfArguments :: (m: Method) -> u32 #foreign libobjc`
- [CPU == .X64] `method_copyReturnType :: (m: Method) -> *u8 #foreign libobjc`
- [CPU == .X64] `method_copyArgumentType :: (m: Method, index: u32) -> *u8 #foreign libobjc`
- [CPU == .X64] `method_getReturnType :: (m: Method, dst: *u8, dst_len: u64) -> void #foreign libobjc`
- [CPU == .X64] `method_getArgumentType :: (m: Method, index: u32, dst: *u8, dst_len: u64) -> void #foreign libobjc`
- [CPU == .X64] `method_getDescription :: (m: Method) -> *objc_method_description #foreign libobjc`
- [CPU == .X64] `method_setImplementation :: (m: Method, imp: IMP) -> IMP #foreign libobjc`
- [CPU == .X64] `method_exchangeImplementations :: (m1: Method, m2: Method) -> void #foreign libobjc`
- [CPU == .X64] `ivar_getName :: (v: Ivar) -> *u8 #foreign libobjc`
- [CPU == .X64] `ivar_getTypeEncoding :: (v: Ivar) -> *u8 #foreign libobjc`
- [CPU == .X64] `ivar_getOffset :: (v: Ivar) -> ptrdiff_t #foreign libobjc`
- [CPU == .X64] `property_getName :: (property: objc_property_t) -> *u8 #foreign libobjc`
- [CPU == .X64] `property_getAttributes :: (property: objc_property_t) -> *u8 #foreign libobjc`
- [CPU == .X64] `property_copyAttributeList :: (property: objc_property_t, outCount: *u32) -> *objc_property_attribute_t #foreign libobjc`
- [CPU == .X64] `property_copyAttributeValue :: (property: objc_property_t, attributeName: *u8) -> *u8 #foreign libobjc`
- [CPU == .X64] `objc_getProtocol :: (name: *u8) -> *Protocol #foreign libobjc`
- [CPU == .X64] `objc_copyProtocolList :: (outCount: *u32) -> **Protocol #foreign libobjc`
- [CPU == .X64] `protocol_conformsToProtocol :: (proto: *Protocol, other: *Protocol) -> BOOL #foreign libobjc`
- [CPU == .X64] `protocol_isEqual :: (proto: *Protocol, other: *Protocol) -> BOOL #foreign libobjc`
- [CPU == .X64] `protocol_getName :: (proto: *Protocol) -> *u8 #foreign libobjc`
- [CPU == .X64] `protocol_getMethodDescription :: (proto: *Protocol, aSel: SEL, isRequiredMethod: BOOL, isInstanceMethod: BOOL) -> objc_method_description #foreign libobjc`
- [CPU == .X64] `protocol_copyMethodDescriptionList :: (proto: *Protocol, isRequiredMethod: BOOL, isInstanceMethod: BOOL, outCount: *u32) -> *objc_method_description #foreign libobjc`
- [CPU == .X64] `protocol_getProperty :: (proto: *Protocol, name: *u8, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> objc_property_t #foreign libobjc`
- [CPU == .X64] `protocol_copyPropertyList :: (proto: *Protocol, outCount: *u32) -> *objc_property_t #foreign libobjc`
- [CPU == .X64] `protocol_copyPropertyList2 :: (proto: *Protocol, outCount: *u32, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> *objc_property_t #foreign libobjc`
- [CPU == .X64] `protocol_copyProtocolList :: (proto: *Protocol, outCount: *u32) -> **Protocol #foreign libobjc`
- [CPU == .X64] `objc_allocateProtocol :: (name: *u8) -> *Protocol #foreign libobjc`
- [CPU == .X64] `objc_registerProtocol :: (proto: *Protocol) -> void #foreign libobjc`
- [CPU == .X64] `protocol_addMethodDescription :: (proto: *Protocol, name: SEL, types: *u8, isRequiredMethod: BOOL, isInstanceMethod: BOOL) -> void #foreign libobjc`
- [CPU == .X64] `protocol_addProtocol :: (proto: *Protocol, addition: *Protocol) -> void #foreign libobjc`
- [CPU == .X64] `protocol_addProperty :: (proto: *Protocol, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> void #foreign libobjc`
- [CPU == .X64] `objc_copyImageNames :: (outCount: *u32) -> **u8 #foreign libobjc`
- [CPU == .X64] `class_getImageName :: (cls: Class) -> *u8 #foreign libobjc`
- [CPU == .X64] `objc_copyClassNamesForImage :: (image: *u8, outCount: *u32) -> **u8 #foreign libobjc`
- [CPU == .X64] `sel_isEqual :: (lhs: SEL, rhs: SEL) -> BOOL #foreign libobjc`
- [CPU == .X64] `objc_enumerationMutation :: (obj: id) -> void #foreign libobjc`
- [CPU == .X64] `objc_setEnumerationMutationHandler :: (handler: #type (unknown0: id) -> void #c_call) -> void #foreign libobjc`
- [CPU == .X64] `objc_setForwardHandler :: (fwd: *void, fwd_stret: *void) -> void #foreign libobjc`
- [CPU == .X64] `imp_implementationWithBlock :: (block: id) -> IMP #foreign libobjc`
- [CPU == .X64] `imp_getBlock :: (anImp: IMP) -> id #foreign libobjc`
- [CPU == .X64] `imp_removeBlock :: (anImp: IMP) -> BOOL #foreign libobjc`
- [CPU == .X64] `objc_loadWeak :: (location: *id) -> id #foreign libobjc`
- [CPU == .X64] `objc_storeWeak :: (location: *id, obj: id) -> id #foreign libobjc`
- [CPU == .X64] `objc_AssociationPolicy :: u64`
- [CPU == .X64] `OBJC_ASSOCIATION :: enum u32`
- [CPU == .X64] `objc_setAssociatedObject :: (object: id, key: *void, value: id, policy: objc_AssociationPolicy) -> void #foreign libobjc`
- [CPU == .X64] `objc_getAssociatedObject :: (object: id, key: *void) -> id #foreign libobjc`
- [CPU == .X64] `objc_removeAssociatedObjects :: (object: id) -> void #foreign libobjc`
- [CPU == .X64] `objc_method_description_list :: struct`
- [CPU == .X64] `objc_protocol_list :: struct`
- [CPU == .X64] `objc_category :: struct`
- [CPU == .X64] `objc_ivar :: struct`
- [CPU == .X64] `objc_ivar_list :: struct`
- [CPU == .X64] `objc_method :: struct`
- [CPU == .X64] `objc_method_list :: struct`
- [CPU == .X64] `Symtab :: *objc_symtab`
- [CPU == .X64] `objc_symtab :: struct`
- [CPU == .X64] `Cache :: *objc_cache`
- [CPU == .X64] `objc_cache :: struct`
- [CPU == .X64] `Module :: *objc_module`
- [CPU == .X64] `objc_module :: struct`
- [CPU == .X64] `class_lookupMethod :: (cls: Class, sel: SEL) -> IMP #foreign libobjc`
- [CPU == .X64] `class_respondsToMethod :: (cls: Class, sel: SEL) -> BOOL #foreign libobjc`
- [CPU == .X64] `_objc_flush_caches :: (cls: Class) -> void #foreign libobjc`
- [CPU == .X64] `object_copyFromZone :: (anObject: id, nBytes: u64, z: *void) -> id #foreign libobjc`
- [CPU == .X64] `class_createInstanceFromZone :: (unknown0: Class, idxIvars: u64, z: *void) -> id #foreign libobjc`
- [CPU == .X64] `marg_prearg_size :: 0`
- [CPU == .X64] `objc_super :: struct`
- [CPU == .X64] `objc_msgSend :: (self: id, op: SEL, __args: ..Any) -> id #foreign libobjc`
- [CPU == .X64] `objc_msgSendSuper :: (super: *objc_super, op: SEL, __args: ..Any) -> id #foreign libobjc`
- [CPU == .X64] `objc_msgSend_stret :: (self: id, op: SEL, __args: ..Any) -> void #foreign libobjc`
- [CPU == .X64] `objc_msgSendSuper_stret :: (super: *objc_super, op: SEL, __args: ..Any) -> void #foreign libobjc`
- [CPU == .X64] `method_invoke :: (receiver: id, m: Method, __args: ..Any) -> id #foreign libobjc`
- [CPU == .X64] `method_invoke_stret :: (receiver: id, m: Method, __args: ..Any) -> void #foreign libobjc`
- [CPU == .X64] `_objc_msgForward :: (receiver: id, sel: SEL, __args: ..Any) -> id #foreign libobjc`
- [CPU == .X64] `_objc_msgForward_stret :: (receiver: id, sel: SEL, __args: ..Any) -> void #foreign libobjc`
- [CPU == .X64] `marg_list :: *void`
- [CPU == .ARM64] `OBJC_BOOL_IS_BOOL :: 1`
- [CPU == .ARM64] `ARITH_SHIFT :: 32`
- [CPU == .ARM64] `OBJC_GETCLASSHOOK_DEFINED :: 1`
- [CPU == .ARM64] `OBJC_ADDLOADIMAGEFUNC_DEFINED :: 1`
- [CPU == .ARM64] `OBJC_SETHOOK_LAZYCLASSNAMER_DEFINED :: 1`
- [CPU == .ARM64] `OBJC_REALIZECLASSFROMSWIFT_DEFINED :: 1`
- [CPU == .ARM64] `CLS_CLASS :: 0x1`
- [CPU == .ARM64] `CLS_META :: 0x2`
- [CPU == .ARM64] `CLS_INITIALIZED :: 0x4`
- [CPU == .ARM64] `CLS_POSING :: 0x8`
- [CPU == .ARM64] `CLS_MAPPED :: 0x10`
- [CPU == .ARM64] `CLS_FLUSH_CACHE :: 0x20`
- [CPU == .ARM64] `CLS_GROW_CACHE :: 0x40`
- [CPU == .ARM64] `CLS_NEED_BIND :: 0x80`
- [CPU == .ARM64] `CLS_METHOD_ARRAY :: 0x100`
- [CPU == .ARM64] `CLS_JAVA_HYBRID :: 0x200`
- [CPU == .ARM64] `CLS_JAVA_CLASS :: 0x400`
- [CPU == .ARM64] `CLS_INITIALIZING :: 0x800`
- [CPU == .ARM64] `CLS_FROM_BUNDLE :: 0x1000`
- [CPU == .ARM64] `CLS_HAS_CXX_STRUCTORS :: 0x2000`
- [CPU == .ARM64] `CLS_NO_METHOD_ARRAY :: 0x4000`
- [CPU == .ARM64] `CLS_HAS_LOAD_METHOD :: 0x8000`
- [CPU == .ARM64] `CLS_CONSTRUCTING :: 0x10000`
- [CPU == .ARM64] `CLS_EXT :: 0x20000`
- [CPU == .ARM64] `OBSOLETE_OBJC_GETCLASSES :: 1`
- [CPU == .ARM64] `OBJC_NEXT_METHOD_LIST :: 1`
- [CPU == .ARM64] `Class :: *objc_class`
- [CPU == .ARM64] `NSObject :: struct`
- [CPU == .ARM64] `id :: *NSObject`
- [CPU == .ARM64] `objc_selector :: struct`
- [CPU == .ARM64] `SEL :: *objc_selector`
- [CPU == .ARM64] `IMP :: #type () -> void #c_call`
- [CPU == .ARM64] `sel_getName :: (sel: SEL) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `sel_registerName :: (str: *u8) -> SEL #foreign libobjc`
- [CPU == .ARM64] `object_getClassName :: (obj: id) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `object_getIndexedIvars :: (obj: id) -> *void #foreign libobjc`
- [CPU == .ARM64] `sel_isMapped :: (sel: SEL) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `sel_getUid :: (str: *u8) -> SEL #foreign libobjc`
- [CPU == .ARM64] `objc_objectptr_t :: *void`
- [CPU == .ARM64] `objc_retainedObject :: (obj: objc_objectptr_t) -> id #foreign libobjc`
- [CPU == .ARM64] `objc_unretainedObject :: (obj: objc_objectptr_t) -> id #foreign libobjc`
- [CPU == .ARM64] `objc_unretainedPointer :: (obj: id) -> objc_objectptr_t #foreign libobjc`
- [CPU == .ARM64] `arith_t :: s64`
- [CPU == .ARM64] `uarith_t :: u64`
- [CPU == .ARM64] `STR :: *u8`
- [CPU == .ARM64] `Method :: *objc_method`
- [CPU == .ARM64] `Ivar :: *objc_ivar`
- [CPU == .ARM64] `Category :: *objc_category`
- [CPU == .ARM64] `objc_property :: struct`
- [CPU == .ARM64] `objc_property_t :: *objc_property`
- [CPU == .ARM64] `objc_class :: struct`
- [CPU == .ARM64] `Protocol :: NSObject`
- [CPU == .ARM64] `objc_method_description :: struct`
- [CPU == .ARM64] `objc_property_attribute_t :: struct`
- [CPU == .ARM64] `object_copy :: (obj: id, size: u64) -> id #foreign libobjc`
- [CPU == .ARM64] `object_dispose :: (obj: id) -> id #foreign libobjc`
- [CPU == .ARM64] `object_getClass :: (obj: id) -> Class #foreign libobjc`
- [CPU == .ARM64] `object_setClass :: (obj: id, cls: Class) -> Class #foreign libobjc`
- [CPU == .ARM64] `object_isClass :: (obj: id) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `object_getIvar :: (obj: id, ivar: Ivar) -> id #foreign libobjc`
- [CPU == .ARM64] `object_setIvar :: (obj: id, ivar: Ivar, value: id) -> void #foreign libobjc`
- [CPU == .ARM64] `object_setIvarWithStrongDefault :: (obj: id, ivar: Ivar, value: id) -> void #foreign libobjc`
- [CPU == .ARM64] `object_setInstanceVariable :: (obj: id, name: *u8, value: *void) -> Ivar #foreign libobjc`
- [CPU == .ARM64] `object_setInstanceVariableWithStrongDefault :: (obj: id, name: *u8, value: *void) -> Ivar #foreign libobjc`
- [CPU == .ARM64] `object_getInstanceVariable :: (obj: id, name: *u8, outValue: **void) -> Ivar #foreign libobjc`
- [CPU == .ARM64] `objc_getClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .ARM64] `objc_getMetaClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .ARM64] `objc_lookUpClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .ARM64] `objc_getRequiredClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .ARM64] `objc_getClassList :: (buffer: *Class, bufferCount: s32) -> s32 #foreign libobjc`
- [CPU == .ARM64] `objc_copyClassList :: (outCount: *u32) -> *Class #foreign libobjc`
- [CPU == .ARM64] `class_getName :: (cls: Class) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `class_isMetaClass :: (cls: Class) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `class_getSuperclass :: (cls: Class) -> Class #foreign libobjc`
- [CPU == .ARM64] `class_setSuperclass :: (cls: Class, newSuper: Class) -> Class #foreign libobjc`
- [CPU == .ARM64] `class_getVersion :: (cls: Class) -> s32 #foreign libobjc`
- [CPU == .ARM64] `class_setVersion :: (cls: Class, version: s32) -> void #foreign libobjc`
- [CPU == .ARM64] `class_getInstanceSize :: (cls: Class) -> u64 #foreign libobjc`
- [CPU == .ARM64] `class_getInstanceVariable :: (cls: Class, name: *u8) -> Ivar #foreign libobjc`
- [CPU == .ARM64] `class_getClassVariable :: (cls: Class, name: *u8) -> Ivar #foreign libobjc`
- [CPU == .ARM64] `class_copyIvarList :: (cls: Class, outCount: *u32) -> *Ivar #foreign libobjc`
- [CPU == .ARM64] `class_getInstanceMethod :: (cls: Class, name: SEL) -> Method #foreign libobjc`
- [CPU == .ARM64] `class_getClassMethod :: (cls: Class, name: SEL) -> Method #foreign libobjc`
- [CPU == .ARM64] `class_getMethodImplementation :: (cls: Class, name: SEL) -> IMP #foreign libobjc`
- [CPU == .ARM64] `class_respondsToSelector :: (cls: Class, sel: SEL) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `class_copyMethodList :: (cls: Class, outCount: *u32) -> *Method #foreign libobjc`
- [CPU == .ARM64] `class_conformsToProtocol :: (cls: Class, protocol: *Protocol) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `class_copyProtocolList :: (cls: Class, outCount: *u32) -> **Protocol #foreign libobjc`
- [CPU == .ARM64] `class_getProperty :: (cls: Class, name: *u8) -> objc_property_t #foreign libobjc`
- [CPU == .ARM64] `class_copyPropertyList :: (cls: Class, outCount: *u32) -> *objc_property_t #foreign libobjc`
- [CPU == .ARM64] `class_getIvarLayout :: (cls: Class) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `class_getWeakIvarLayout :: (cls: Class) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `class_addMethod :: (cls: Class, name: SEL, imp: IMP, types: *u8) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `class_replaceMethod :: (cls: Class, name: SEL, imp: IMP, types: *u8) -> IMP #foreign libobjc`
- [CPU == .ARM64] `class_addIvar :: (cls: Class, name: *u8, size: u64, alignment: u8, types: *u8) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `class_addProtocol :: (cls: Class, protocol: *Protocol) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `class_addProperty :: (cls: Class, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `class_replaceProperty :: (cls: Class, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32) -> void #foreign libobjc`
- [CPU == .ARM64] `class_setIvarLayout :: (cls: Class, layout: *u8) -> void #foreign libobjc`
- [CPU == .ARM64] `class_setWeakIvarLayout :: (cls: Class, layout: *u8) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_getFutureClass :: (name: *u8) -> Class #foreign libobjc`
- [CPU == .ARM64] `class_createInstance :: (cls: Class, extraBytes: u64) -> id #foreign libobjc`
- [CPU == .ARM64] `objc_constructInstance :: (cls: Class, bytes: *void) -> id #foreign libobjc`
- [CPU == .ARM64] `objc_destructInstance :: (obj: id) -> *void #foreign libobjc`
- [CPU == .ARM64] `objc_allocateClassPair :: (superclass: Class, name: *u8, extraBytes: u64) -> Class #foreign libobjc`
- [CPU == .ARM64] `objc_registerClassPair :: (cls: Class) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_duplicateClass :: (original: Class, name: *u8, extraBytes: u64) -> Class #foreign libobjc`
- [CPU == .ARM64] `objc_disposeClassPair :: (cls: Class) -> void #foreign libobjc`
- [CPU == .ARM64] `method_getName :: (m: Method) -> SEL #foreign libobjc`
- [CPU == .ARM64] `method_getImplementation :: (m: Method) -> IMP #foreign libobjc`
- [CPU == .ARM64] `method_getTypeEncoding :: (m: Method) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `method_getNumberOfArguments :: (m: Method) -> u32 #foreign libobjc`
- [CPU == .ARM64] `method_copyReturnType :: (m: Method) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `method_copyArgumentType :: (m: Method, index: u32) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `method_getReturnType :: (m: Method, dst: *u8, dst_len: u64) -> void #foreign libobjc`
- [CPU == .ARM64] `method_getArgumentType :: (m: Method, index: u32, dst: *u8, dst_len: u64) -> void #foreign libobjc`
- [CPU == .ARM64] `method_getDescription :: (m: Method) -> *objc_method_description #foreign libobjc`
- [CPU == .ARM64] `method_setImplementation :: (m: Method, imp: IMP) -> IMP #foreign libobjc`
- [CPU == .ARM64] `method_exchangeImplementations :: (m1: Method, m2: Method) -> void #foreign libobjc`
- [CPU == .ARM64] `ivar_getName :: (v: Ivar) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `ivar_getTypeEncoding :: (v: Ivar) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `ivar_getOffset :: (v: Ivar) -> ptrdiff_t #foreign libobjc`
- [CPU == .ARM64] `property_getName :: (property: objc_property_t) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `property_getAttributes :: (property: objc_property_t) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `property_copyAttributeList :: (property: objc_property_t, outCount: *u32) -> *objc_property_attribute_t #foreign libobjc`
- [CPU == .ARM64] `property_copyAttributeValue :: (property: objc_property_t, attributeName: *u8) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `objc_getProtocol :: (name: *u8) -> *Protocol #foreign libobjc`
- [CPU == .ARM64] `objc_copyProtocolList :: (outCount: *u32) -> **Protocol #foreign libobjc`
- [CPU == .ARM64] `protocol_conformsToProtocol :: (proto: *Protocol, other: *Protocol) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `protocol_isEqual :: (proto: *Protocol, other: *Protocol) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `protocol_getName :: (proto: *Protocol) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `protocol_getMethodDescription :: (proto: *Protocol, aSel: SEL, isRequiredMethod: BOOL, isInstanceMethod: BOOL) -> objc_method_description #foreign libobjc`
- [CPU == .ARM64] `protocol_copyMethodDescriptionList :: (proto: *Protocol, isRequiredMethod: BOOL, isInstanceMethod: BOOL, outCount: *u32) -> *objc_method_description #foreign libobjc`
- [CPU == .ARM64] `protocol_getProperty :: (proto: *Protocol, name: *u8, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> objc_property_t #foreign libobjc`
- [CPU == .ARM64] `protocol_copyPropertyList :: (proto: *Protocol, outCount: *u32) -> *objc_property_t #foreign libobjc`
- [CPU == .ARM64] `protocol_copyPropertyList2 :: (proto: *Protocol, outCount: *u32, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> *objc_property_t #foreign libobjc`
- [CPU == .ARM64] `protocol_copyProtocolList :: (proto: *Protocol, outCount: *u32) -> **Protocol #foreign libobjc`
- [CPU == .ARM64] `objc_allocateProtocol :: (name: *u8) -> *Protocol #foreign libobjc`
- [CPU == .ARM64] `objc_registerProtocol :: (proto: *Protocol) -> void #foreign libobjc`
- [CPU == .ARM64] `protocol_addMethodDescription :: (proto: *Protocol, name: SEL, types: *u8, isRequiredMethod: BOOL, isInstanceMethod: BOOL) -> void #foreign libobjc`
- [CPU == .ARM64] `protocol_addProtocol :: (proto: *Protocol, addition: *Protocol) -> void #foreign libobjc`
- [CPU == .ARM64] `protocol_addProperty :: (proto: *Protocol, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_copyImageNames :: (outCount: *u32) -> **u8 #foreign libobjc`
- [CPU == .ARM64] `class_getImageName :: (cls: Class) -> *u8 #foreign libobjc`
- [CPU == .ARM64] `objc_copyClassNamesForImage :: (image: *u8, outCount: *u32) -> **u8 #foreign libobjc`
- [CPU == .ARM64] `sel_isEqual :: (lhs: SEL, rhs: SEL) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `objc_enumerationMutation :: (obj: id) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_setEnumerationMutationHandler :: (handler: #type (unknown0: id) -> void #c_call) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_setForwardHandler :: (fwd: *void, fwd_stret: *void) -> void #foreign libobjc`
- [CPU == .ARM64] `imp_implementationWithBlock :: (block: id) -> IMP #foreign libobjc`
- [CPU == .ARM64] `imp_getBlock :: (anImp: IMP) -> id #foreign libobjc`
- [CPU == .ARM64] `imp_removeBlock :: (anImp: IMP) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `objc_loadWeak :: (location: *id) -> id #foreign libobjc`
- [CPU == .ARM64] `objc_storeWeak :: (location: *id, obj: id) -> id #foreign libobjc`
- [CPU == .ARM64] `objc_AssociationPolicy :: u64`
- [CPU == .ARM64] `OBJC_ASSOCIATION :: enum u32`
- [CPU == .ARM64] `objc_setAssociatedObject :: (object: id, key: *void, value: id, policy: objc_AssociationPolicy) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_getAssociatedObject :: (object: id, key: *void) -> id #foreign libobjc`
- [CPU == .ARM64] `objc_removeAssociatedObjects :: (object: id) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_hook_getImageName :: #type (cls: Class, outImageName: **u8) -> BOOL #c_call`
- [CPU == .ARM64] `objc_setHook_getImageName :: (newValue: objc_hook_getImageName, outOldValue: *objc_hook_getImageName) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_hook_getClass :: #type (name: *u8, outClass: *Class) -> BOOL #c_call`
- [CPU == .ARM64] `objc_setHook_getClass :: (newValue: objc_hook_getClass, outOldValue: *objc_hook_getClass) -> void #foreign libobjc`
- [CPU == .ARM64] `mach_header :: struct`
- [CPU == .ARM64] `objc_func_loadImage :: #type (header: *mach_header) -> void #c_call`
- [CPU == .ARM64] `objc_addLoadImageFunc :: (func: objc_func_loadImage) -> void #foreign libobjc`
- [CPU == .ARM64] `objc_hook_lazyClassNamer :: #type (cls: Class) -> *u8 #c_call`
- [CPU == .ARM64] `objc_setHook_lazyClassNamer :: (newValue: objc_hook_lazyClassNamer, oldOutValue: *objc_hook_lazyClassNamer) -> void #foreign libobjc`
- [CPU == .ARM64] `_objc_swiftMetadataInitializer :: #type (cls: Class, arg: *void) -> Class #c_call`
- [CPU == .ARM64] `_objc_realizeClassFromSwift :: (cls: Class, previously: *void) -> Class #foreign libobjc`
- [CPU == .ARM64] `objc_method_description_list :: struct`
- [CPU == .ARM64] `objc_protocol_list :: struct`
- [CPU == .ARM64] `objc_category :: struct`
- [CPU == .ARM64] `objc_ivar :: struct`
- [CPU == .ARM64] `objc_ivar_list :: struct`
- [CPU == .ARM64] `objc_method :: struct`
- [CPU == .ARM64] `objc_method_list :: struct`
- [CPU == .ARM64] `Symtab :: *objc_symtab`
- [CPU == .ARM64] `objc_symtab :: struct`
- [CPU == .ARM64] `Cache :: *objc_cache`
- [CPU == .ARM64] `objc_cache :: struct`
- [CPU == .ARM64] `Module :: *objc_module`
- [CPU == .ARM64] `objc_module :: struct`
- [CPU == .ARM64] `class_lookupMethod :: (cls: Class, sel: SEL) -> IMP #foreign libobjc`
- [CPU == .ARM64] `class_respondsToMethod :: (cls: Class, sel: SEL) -> BOOL #foreign libobjc`
- [CPU == .ARM64] `_objc_flush_caches :: (cls: Class) -> void #foreign libobjc`
- [CPU == .ARM64] `object_copyFromZone :: (anObject: id, nBytes: u64, z: *void) -> id #foreign libobjc`
- [CPU == .ARM64] `class_createInstanceFromZone :: (unknown0: Class, idxIvars: u64, z: *void) -> id #foreign libobjc`
- [CPU == .ARM64] `marg_prearg_size :: 0`
- [CPU == .ARM64] `objc_super :: struct`
- [CPU == .ARM64] `objc_msgSend :: () -> void #foreign libobjc`
- [CPU == .ARM64] `objc_msgSendSuper :: () -> void #foreign libobjc`
- [CPU == .ARM64] `method_invoke :: () -> void #foreign libobjc`
- [CPU == .ARM64] `_objc_msgForward :: () -> void #foreign libobjc`
- [CPU == .ARM64] `marg_list :: *void`
- `init_objective_c_foundation :: ()`
- `NSTimeInterval :: float64`
- `NSFullUserName :: () -> *NSString #foreign Foundation_Framework`
- `NSUserName :: () -> *NSString #foreign Foundation_Framework`
- `NSHomeDirectory :: () -> *NSString #foreign Foundation_Framework`
- `NSHomeDirectoryForUser :: (username: *NSString) -> *NSString #foreign Foundation_Framework`
- `NSAutoreleasePool :: struct`
- `NSRange :: struct`
- `NSBundle :: struct`
- `NSDate :: struct`
- `NSPoint :: struct`
- `CGFloat :: float64`
- `CGSize :: struct`
- `NSMakePoint :: (x: CGFloat, y: CGFloat) -> NSPoint`
- `NSSize :: CGSize`
- `NSMakeSize :: (w: CGFloat, h: CGFloat) -> NSSize`
- `NSRect :: struct`
- `NSMakeRect :: (x: CGFloat, y: CGFloat, w: CGFloat, h: CGFloat) -> NSRect`
- `NSStringEncoding :: NSUInteger`
- `NSUTF8StringEncoding :: 4`
- `NSUTF32StringEncoding :: 0x8c000100`
- `NSUTF32LittleEndianStringEncoding :: 0x9c000100`
- `NSString :: struct`
- `NSRunLoopMode :: *NSString`
- `NSPasteboardType :: *NSString`
- `NSColorSpaceName :: *NSString`
- `NSWindowsNTOperatingSystem :: 1`
- `NSWindows95OperatingSystem :: 2`
- `NSSolarisOperatingSystem :: 3`
- `NSHPUXOperatingSystem :: 4`
- `NSMACHOperatingSystem :: 5`
- `NSSunOSOperatingSystem :: 6`
- `NSOSF1OperatingSystem :: 7`
- `NSOperatingSystemVersion :: struct`
- `NSTimer :: struct`
- `NSRunLoop :: struct`
- `NSProcessInfo :: struct`
- `using NSActivityOptions :: enum u64`
- `using NSProcessInfoThermalState :: enum NSInteger`
- `NSThread :: struct`
- `NSNotification :: struct`
- `NSNotificationCenter :: struct`
- `NSNotificationName :: *NSString`
- `NSURL :: struct`
- `NSData :: struct`
- `NSDictionary :: struct (key: Type, value: Type)`
- `NSArray :: struct (Object_Type: Type)`
- `NSFileManager :: struct`
- `using NSSearchPathDirectory :: enum NSUInteger`
- `using NSSearchPathDomainMask :: enum_flags NSUInteger`
- `NSError :: struct`
- `NSHost :: struct`
- `CALayer :: struct`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `CALayer`

```jai
CALayer :: struct {
    using nsobject: NSObject;
}
```
### `CGSize`

```jai
CGSize :: struct {
    width : CGFloat;
    height: CGFloat;
}
```
### `NSActivityOptions`

```jai
using NSActivityOptions :: enum u64 {
    NSActivityIdleDisplaySleepDisabled :: 1099511627776;
    NSActivityIdleSystemSleepDisabled :: 1048576;
    NSActivitySuddenTerminationDisabled :: 16384;
    NSActivityAutomaticTerminationDisabled :: 32768;
    NSActivityUserInitiated :: 16777215;
    NSActivityUserInitiatedAllowingIdleSystemSleep :: 15728639;
    NSActivityBackground :: 255;
    NSActivityLatencyCritical :: 1095216660480;
}
```
### `NSArray`

```jai
NSArray :: struct (Object_Type: Type) {
    using #as super: NSObject;

arrayWithObject :: (obj: Object_Type) -> *NSArray(Object_Type);
arrayWithObjects :: (objs: *Object_Type, count: NSUInteger) -> *NSArray(Object_Type);

containsObject :: (self: *NSArray(Object_Type), anObject: Object_Type) -> BOOL;

objectAtIndex :: (self: *NSArray(Object_Type), index: NSUInteger) -> Object_Type;

count :: (self: *NSArray(Object_Type)) -> NSUInteger;
}
```
### `NSAutoreleasePool`

```jai
NSAutoreleasePool :: struct {
    using #as super: NSObject;

drain :: (self: *NSAutoreleasePool);

showPools :: ();
}
```
### `NSBundle`

```jai
NSBundle :: struct {
    using #as super: NSObject;

resourcePath :: (self: *NSBundle) -> *NSString;

mainBundle :: () -> *NSBundle;
}
```
### `NSData`

```jai
NSData :: struct {
    using #as super: NSObject;

bytes :: (self: *NSData) -> *void;

length :: (self: *NSData) -> NSUInteger;
}
```
### `NSDate`

```jai
NSDate :: struct {
    using #as super: NSObject;

distantPast :: () -> *NSDate;

distantFuture :: () -> *NSDate;
}
```
### `NSDictionary`

```jai
NSDictionary :: struct (key: Type, value: Type) {
    using #as super: NSObject;
}
```
### `NSError`

```jai
NSError :: struct {
    using #as super: NSObject;

localizedDescription :: (self: *NSError) -> *NSString;
}
```
### `NSFileManager`

```jai
NSFileManager :: struct {
    using #as super: NSObject;

defaultManager :: () -> *NSFileManager;

URLForDirectory :: (self: *NSFileManager, dir: NSSearchPathDirectory, inDomain: NSSearchPathDomainMask, appropriateForURL: *NSURL, create: BOOL, error: **NSError) -> *NSURL;
}
```
### `NSHost`

```jai
NSHost :: struct {
    using #as super: NSObject;

currentHost :: () -> *NSHost;

localizedName :: (self: *NSHost) -> *NSString;
}
```
### `NSNotification`

```jai
NSNotification :: struct {
    using #as super: NSObject;

object :: (self: *NSNotification) -> *NSObject;
}
```
### `NSNotificationCenter`

```jai
NSNotificationCenter :: struct {
    using #as super: NSObject;

defaultCenter :: () -> *NSNotificationCenter;

addObserver :: (self: *NSNotificationCenter, observer: id, selector: Selector, name: NSNotificationName, object: id);
}
```
### `NSObject`

```jai
NSObject :: struct {
    isa: Class;
}
```
### `NSOperatingSystemVersion`

```jai
NSOperatingSystemVersion :: struct {
    majorVersion: NSInteger;
    minorVersion: NSInteger;
    patchVersion: NSInteger;
}
```
### `NSPoint`

```jai
NSPoint :: struct {
    x: CGFloat;
    y: CGFloat;
}
```
### `NSProcessInfo`

```jai
NSProcessInfo :: struct {
    using nsobject: NSObject;
    _environment: *NSDictionary(*NSString, *NSString);
    _arguments: *NSArray(*NSString);
    _hostName: *NSString;
    _name: *NSString;
    automaticTerminationOptOutCounter: NSInteger;

operatingSystem :: (self: *NSProcessInfo) -> NSUInteger;

operatingSystemName :: (self: *NSProcessInfo) -> *NSString;

isOperatingSystemAtLeastVersion :: (self: *NSProcessInfo, version: NSOperatingSystemVersion) -> BOOL;

disableSuddenTermination :: (self: *NSProcessInfo) -> ();

enableSuddenTermination :: (self: *NSProcessInfo) -> ();

disableAutomaticTermination :: (self: *NSProcessInfo, reason: *NSString) -> ();

enableAutomaticTermination :: (self: *NSProcessInfo, reason: *NSString) -> ();

processInfo :: () -> *NSProcessInfo;

environment :: (self: *NSProcessInfo) -> *NSDictionary;

arguments :: (self: *NSProcessInfo) -> *NSArray(*NSString);

hostName :: (self: *NSProcessInfo) -> *NSString;

processName :: (self: *NSProcessInfo) -> *NSString;

setProcessName :: (self: *NSProcessInfo, processName: *NSString) -> ();

processIdentifier :: (self: *NSProcessInfo) -> s32;

globallyUniqueString :: (self: *NSProcessInfo) -> *NSString;

operatingSystemVersionString :: (self: *NSProcessInfo) -> *NSString;

operatingSystemVersion :: (self: *NSProcessInfo) -> NSOperatingSystemVersion;

processorCount :: (self: *NSProcessInfo) -> NSUInteger;

activeProcessorCount :: (self: *NSProcessInfo) -> NSUInteger;

physicalMemory :: (self: *NSProcessInfo) -> u64;

systemUptime :: (self: *NSProcessInfo) -> NSTimeInterval;

automaticTerminationSupportEnabled :: (self: *NSProcessInfo) -> BOOL;

setAutomaticTerminationSupportEnabled :: (self: *NSProcessInfo, automaticTerminationSupportEnabled: BOOL) -> ();

}
```
### `NSProcessInfoThermalState`

```jai
using NSProcessInfoThermalState :: enum NSInteger {
    NSProcessInfoThermalStateNominal :: 0;
    NSProcessInfoThermalStateFair :: 1;
    NSProcessInfoThermalStateSerious :: 2;
    NSProcessInfoThermalStateCritical :: 3;
}
```
### `NSRange`

```jai
NSRange :: struct {
    location: NSUInteger;
    length: NSUInteger;
}
```
### `NSRect`

```jai
NSRect :: struct {
    origin: NSPoint;
    size: NSSize;
}
```
### `NSRunLoop`

```jai
NSRunLoop :: struct {
    using #as super: NSObject;

currentRunLoop :: () -> *NSRunLoop;

run :: (self: *NSRunLoop);
}
```
### `NSSearchPathDirectory`

```jai
using NSSearchPathDirectory :: enum NSUInteger {
    NSApplicationDirectory :: 1;
    NSDemoApplicationDirectory;
    NSDeveloperApplicationDirectory;
    NSAdminApplicationDirectory;
    NSLibraryDirectory;
    NSDeveloperDirectory;
    NSUserDirectory;
    NSDocumentationDirectory;
    NSDocumentDirectory;
    NSCoreServiceDirectory;
    NSAutosavedInformationDirectory :: 11;
    NSDesktopDirectory :: 12;
    NSCachesDirectory :: 13;
    NSApplicationSupportDirectory :: 14;
    NSDownloadsDirectory :: 15;
    NSInputMethodsDirectory :: 16;
    NSMoviesDirectory :: 17;
    NSMusicDirectory :: 18;
    NSPicturesDirectory :: 19;
    NSPrinterDescriptionDirectory :: 20;
    NSSharedPublicDirectory :: 21;
    NSPreferencePanesDirectory :: 22;
    NSApplicationScriptsDirectory :: 23;
    NSItemReplacementDirectory :: 99;
    NSAllApplicationsDirectory :: 100;
    NSAllLibrariesDirectory :: 101;
    NSTrashDirectory :: 102;
}
```
### `NSSearchPathDomainMask`

```jai
using NSSearchPathDomainMask :: enum_flags NSUInteger {
    NSUserDomainMask :: 1;
    NSLocalDomainMask :: 2;
    NSNetworkDomainMask :: 4;
    NSSystemDomainMask :: 8;
    NSAllDomainsMask :: 0x0ffff;
}
```
### `NSString`

```jai
NSString :: struct {
    using #as super: NSObject;

initWithBytes :: (self: *NSString, bytes: *void, length: NSUInteger, encoding: NSStringEncoding) -> *NSString;

UTF8String :: (self: *NSString) -> *u8;

dataUsingEncoding :: (self: *NSString, encoding: NSStringEncoding, allowLossyConversion := NO) -> *NSData;

stringByExpandingTildeInPath :: (self: *NSString) -> *NSString;

length :: (self: *NSString) -> NSUInteger;

initWithString :: (self: *NSString, str: string) -> *NSString;

getTempString :: (str: string) -> *NSString;
}
```
### `NSThread`

```jai
NSThread :: struct {
    _nsobject: NSObject;

detachNewThreadSelectorToTargetWithObject :: (sel: Selector, toTarget: id, withObject: id);

isMultiThreaded :: () -> BOOL;
}
```
### `NSTimer`

```jai
NSTimer :: struct {
    using #as super: NSObject;

scheduledTimerWithTimeInterval :: (interval: NSTimeInterval, target: id, selector: Selector, userInfo: id, repeats: BOOL) -> *NSTimer;
}
```
### `NSURL`

```jai
NSURL :: struct {
    using #as super: NSObject;

URLWithString :: (str: *NSString) -> *NSURL;

fileURLWithPath :: (path: *NSString) -> *NSURL;

fileURLWithPath :: (path: *NSString, isDirectory: BOOL) -> *NSURL;

path :: (self: *NSURL) -> *NSString;

URLByDeletingLastPathComponent :: (self: *NSURL) -> *NSURL;

hasDirectoryPath :: (self: *NSURL) -> BOOL;

URLByAppendingPathComponent :: (self: *NSURL, comp: *NSString) -> *NSURL;

lastPathComponent :: (self: *NSURL) -> *NSString;
}
```
### `OBJC_ASSOCIATION`

```jai
OBJC_ASSOCIATION :: enum u32 {
    ASSIGN           :: 0;
    RETAIN_NONATOMIC :: 1;

    COPY_NONATOMIC   :: 3;

    RETAIN           :: 769;

    COPY             :: 771;

    OBJC_ASSOCIATION_ASSIGN           :: ASSIGN;
    OBJC_ASSOCIATION_RETAIN_NONATOMIC :: RETAIN_NONATOMIC;

    OBJC_ASSOCIATION_COPY_NONATOMIC   :: COPY_NONATOMIC;

    OBJC_ASSOCIATION_RETAIN           :: RETAIN;

    OBJC_ASSOCIATION_COPY             :: COPY;
}
```
### `Objective_C_Block_Basic`

```jai
Objective_C_Block_Basic :: struct {
    isa: *void;
    flags: s32;
    reserved: s32;
    invoke: (*void, args: .. Any) -> () #c_call;
    descriptor: *Objective_C_Block_Basic_Descriptor;

}
```
### `Objective_C_Block_Basic_Descriptor`

```jai
Objective_C_Block_Basic_Descriptor :: struct {
    reserved: u64;
    size: u64;

    copy_helper: (dst: *void, src: *void) #c_call;
    dispose_helper: (src: *void) #c_call;

    signature: *u8;
}
```
### `mach_header`

```jai
mach_header :: struct {}
```
### `objc_cache`

```jai
objc_cache :: struct {
    mask:     u32;
    occupied: u32;
    buckets:  [1] Method;
}
```
### `objc_category`

```jai
objc_category :: struct {
    category_name:    *u8;
    class_name:       *u8;
    instance_methods: *objc_method_list;
    class_methods:    *objc_method_list;
    protocols:        *objc_protocol_list;
}
```
### `objc_class`

```jai
objc_class :: struct {
    #as using super: NSObject;
    super_class:     Class;
    name:            *u8;
    version:         s64;
    info:            s64;
    instance_size:   s64;
    ivars:           *objc_ivar_list;
    methodLists:     **objc_method_list;
    cache:           *objc_cache;
    protocols:       *objc_protocol_list;
}
```
### `objc_ivar`

```jai
objc_ivar :: struct {
    ivar_name:   *u8;
    ivar_type:   *u8;
    ivar_offset: s32;

    space:       s32;
}
```
### `objc_ivar_list`

```jai
objc_ivar_list :: struct {
    ivar_count: s32;

    space:      s32;

    ivar_list:  [1] objc_ivar;
}
```
### `objc_method`

```jai
objc_method :: struct {
    method_name:  SEL;
    method_types: *u8;
    method_imp:   IMP;
}
```
### `objc_method_description`

```jai
objc_method_description :: struct {
    name:  SEL;
    types: *u8;
}
```
### `objc_method_description_list`

```jai
objc_method_description_list :: struct {
    count: s32;
    list:  [1] objc_method_description;
}
```
### `objc_method_list`

```jai
objc_method_list :: struct {
    obsolete:     *objc_method_list;

    method_count: s32;

    space:        s32;

    method_list:  [1] objc_method;
}
```
### `objc_module`

```jai
objc_module :: struct {
    version: u64;
    size:    u64;
    name:    *u8;
    symtab:  Symtab;
}
```
### `objc_property`

```jai
objc_property :: struct {}
```
### `objc_property_attribute_t`

```jai
objc_property_attribute_t :: struct {
    name:  *u8;
    value: *u8;
}
```
### `objc_protocol_list`

```jai
objc_protocol_list :: struct {
    next:  *objc_protocol_list;
    count: s64;
    list:  [1] *Protocol;
}
```
### `objc_selector`

```jai
objc_selector :: struct {}
```
### `objc_super`

```jai
objc_super :: struct {

    receiver: id;

    class:    Class;
}
```
### `objc_symtab`

```jai
objc_symtab :: struct {
    sel_ref_cnt: u64;
    refs:        *SEL;
    cls_def_cnt: u16;
    cat_def_cnt: u16;
    defs:        [1] *void;
}
```
## Contracts and constraints

### macOS AppKit window types (contextual dependency)

The dedicated [Objective_C/AppKit surface](appkit.md) documents the separately
imported AppKit namespace, including application and OpenGL context/pixel-format
types. It does not re-export those declarations from root `Objective_C`.

The macOS `Window_Type` alias is `*ObjC.NSWindow`, where `ObjC` is the
`Objective_C/AppKit` import used by `Window_Creation`. The following are
context needed to implement that public alias and its window operations; they
belong to the AppKit submodule and are not additional root `Objective_C`
exports. `NSResponder` declares `using #as super: NSObject`; `NSWindow`
declares `using #as responder: NSResponder` and has no other stored instance
fields in this binding. These exact directives provide the public conversion
path from `*NSWindow` through `*NSResponder` to the `*NSObject` receiver type;
no additional cast or conversion overload is needed. `NSWindowStyleMask`,
`NSWindowCollectionBehavior`, and
`NSBackingStoreType` are flags/enum types over `NSUInteger` (`u64`):

- `NSWindowStyleMask`: Borderless `0`, Titled `1`, Closable `2`, Miniaturizable
  `4`, Resizable `8`, UtilityWindow `16`, DocModalWindow `64`,
  NonactivatingPanel `128`, TexturedBackground `256`, HUDWindow `8192`,
  UnifiedTitleAndToolbar `4096`, FullScreen `16384`, FullSizeContentView `32768`.
- `NSWindowCollectionBehavior` is declared in the separately imported AppKit
  namespace as `using NSWindowCollectionBehavior :: enum_flags NSUInteger`.
  Its directly exposed prefixed members are `NSWindowCollectionBehaviorDefault
  :: 0`, `NSWindowCollectionBehaviorCanJoinAllSpaces :: 1 << 0`,
  `NSWindowCollectionBehaviorMoveToActiveSpace :: 1 << 1`,
  `NSWindowCollectionBehaviorManaged :: 1 << 2`,
  `NSWindowCollectionBehaviorTransient :: 1 << 3`,
  `NSWindowCollectionBehaviorStationary :: 1 << 4`,
  `NSWindowCollectionBehaviorParticipatesInCycle :: 1 << 5`,
  `NSWindowCollectionBehaviorIgnoresCycle :: 1 << 6`,
  `NSWindowCollectionBehaviorFullScreenPrimary :: 1 << 7`,
  `NSWindowCollectionBehaviorFullScreenAuxiliary :: 1 << 8`,
  `NSWindowCollectionBehaviorFullScreenNone :: 1 << 9`,
  `NSWindowCollectionBehaviorFullScreenAllowsTiling :: 1 << 11`, and
  `NSWindowCollectionBehaviorFullScreenDisallowsTiling :: 1 << 12`. No short
  enum-member aliases are declared. The SDK's newer `Primary`, `Auxiliary`,
  and `CanJoinAllApplications` values are absent from this Jai declaration.
- `NSBackingStoreType`: Retained `0`, Nonretained `1`, Buffered `2`.

Relevant `NSWindow` member signatures are `initWithContentRect(self:
*NSWindow, rect: NSRect, styleMask: NSWindowStyleMask, backing:
NSBackingStoreType, _defer: BOOL) -> *NSWindow` and its overload taking `self:
id` plus `screen: *NSScreen`; `setReleasedWhenClosed(self: *NSWindow, value:
BOOL)`, `setTitle(self: *NSWindow, title: string) -> id`,
`setCollectionBehavior(self: *NSWindow, behavior: NSWindowCollectionBehavior)`,
`makeKeyAndOrderFront(self: *NSWindow, sender: id)`,
`setDelegate(self: *NSWindow, dg: id)`, `setContentView(self: *NSWindow,
contentView: id)`, `makeFirstResponder(self: *NSWindow, view: id)`,
`contentView(self: *NSWindow) -> *NSView`, `center(self: *NSWindow)`,
`styleMask(self: *NSWindow) -> NSWindowStyleMask`,
`toggleFullScreen(self: *NSWindow, sender: id)`, and `frame(self: *NSView) ->
NSRect` (the latter is exposed on `NSWindow` as an alias). These are AppKit
Objective-C message members, not C ABI struct fields.

The AppKit binding's `NSWindow.setTitle(self: *NSWindow, title: string) -> id`
overload marshals the string to an Objective-C string and sends the title
selector, but the native selector returns void. The declared `id` result is
unspecified; callers must ignore it.

This module bridges Jai to the Objective-C runtime and Foundation/AppKit-style APIs. Selector fields are initialized from member names by converting underscores to colons; `objc_msgSend_typed` helpers use explicit function-pointer signatures because Objective-C message return conventions depend on the called method and target ABI. Runtime-created classes require an `isa` field and are documented as having fragile instance-variable layout; use a compiled Objective-C library for classes whose ivar ABI must survive OS/compiler updates. Objective-C object ownership follows retain/release/autorelease conventions and remains the caller’s responsibility.

The public `release` helper is declared to return `id`, but the Objective-C `release` selector has no result; the helper’s return value is unspecified and must not be used for chaining or ownership decisions. `autorelease` returns the autoreleased receiver under its `id` signature, and `retain` returns the retained receiver under its declared `*instancetype` signature. Object lifetime still follows Objective-C reference-count rules.

`create_objc_class_member_stub(class_name, framework_name)` returns a caller-released Jai declaration string for storage sized to the named class's instance bytes beyond its superclass. The member name uses the lowercased class name; for example, 24 additional bytes yield `_samplepanel_data: [24] u8;` plus a size comment. It attempts to load `/System/Library/Frameworks/<framework_name>.framework/<framework_name>` while querying the runtime, then closes the temporary library handle. The returned string describes a declaration; it does not allocate an Objective-C instance or establish a safe fragile-ivar layout. The class name bytes are passed to the runtime as a C string, so they must be NUL-terminated. Unknown classes or missing superclasses assert. The module does not promise that the runtime-measured size is stable across OS versions or safe for fragile-ivar ABI compatibility.

### Type encoding and method registration

`objc_type_encode(builder, info, pointer_depth=0)` appends one Objective-C type-encoding string to the builder's existing contents; it does not clear the builder. Its allocating overload returns the same encoding as a caller-released string. Supported encodings are:

| Reflected type | Encoding |
| --- | --- |
| exact `Class`, `id`, and `Selector` types | `#`, `@`, and `:` respectively |
| `void` | `v` |
| signed/unsigned integers of 1, 2, 4, or 8 bytes | `c`/`C`, `s`/`S`, `l`/`L`, or `q`/`Q` |
| 8-byte float; other supported float width | `d`; `f` |
| boolean; procedure | `B`; `?` |
| enum | encoding of its underlying integer type |
| pointer to a one-byte integer; other pointer | `*`; `^` followed recursively by its pointee encoding |
| fixed array | `[count element-encoding]` |
| array view; resizable array; Jai string | `{_static=q^c}`; `{_dynamic=qq^v?^v}`; `{string=q^c}` |

A struct with an `isa` member of exact `Class` type encodes as `@`. `has_isa_member` returns true for that member and asserts if a member named `isa` has another type. Other structs encode as `{ReflectedName=member-encodings}`; constant, imported, and `using` members are omitted. When `pointer_depth` is at least 2, the struct name remains but its member list is omitted. Pointer recursion increments this depth. Array element encoding starts at the default depth, independent of the caller-supplied depth. `Any` and other unhandled reflected categories, unsupported integer widths, or malformed `isa` members assert; there is no recovery result. `Selector` is a transparent alias of `*void`. `Selector == *void`, and both reflected types encode as `:`. Do not require these alias spellings to encode differently or invent nominal identity to do so. Ordinary pointer recursion applies only when the canonical type does not match an earlier special encoding case.

`objc_func_encode` produces a compact concatenation: the sole return type's encoding (or `v` for no return), followed by each declared argument encoding in order. It does not add argument-size/offset text or an implicit receiver/selector. Method procedures therefore must declare receiver and selector as their first two arguments when used with the registration helpers. The returned string is caller-released.

`type_check_objc_method_type` asserts unless the procedure uses the C calling convention, has at most one return value, and has at least two arguments. Argument 0 must be exact `id`, a pointer to a struct, or a struct with an `isa` field of exact `Class` type. The pointer-to-struct case does not additionally verify an `isa` field. Argument 1 must be exact `Selector`. The helper does not document additional validation for variadic procedures, context requirements, selector colon count, or argument/return categories beyond what `objc_type_encode` supports; do not rely on it to reject those cases.

`objc_create_class(type, parent)` requires both reflected types to be structs and the type to have at least one member. Its class name is the reflected type name; it looks up the parent runtime class by the parent's reflected name. A missing parent is logged and returns null. A runtime size mismatch for the parent or new class is logged as a warning and does not itself prevent creation. The first Jai member is reserved for the runtime `isa`/parent-subobject representation and is not added as a new ivar; later eligible nonzero instance members are registered as native ivars. The source's documented layout precondition is that this first member is `isa: Class` or a parent struct whose layout contains `isa`; the creation procedure does not fully validate that precondition. Failure to allocate the runtime class pair or add an ivar logs and returns null. The result is an allocated, not-yet-registered class; method installation and other class edits precede `objc_finalize_class`, which registers it. Class collisions surface as a null class-pair allocation result. The helper does not instantiate objects or copy default values from a Jai struct.

`objc_add_instance_method` and `objc_add_class_method` accept a procedure value and the exact selector string to register. Both validate the procedure with `type_check_objc_method_type`, derive its compact encoding, and assert if native method installation fails (including duplicate selector installation). The class-method form installs on the class's metaclass. `objc_finalize_class` registers the supplied class and returns no status; callers should invoke it only after completing class and method setup. Invalid inputs and native failures use assertions or null as described, not a recoverable error channel.
