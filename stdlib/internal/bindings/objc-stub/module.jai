// Runtime-measured declaration text. This does not promise stable ivar layout.
create :: (class_name: string, framework_name: string) -> string {
    path := Basic.sprint("/System/Library/Frameworks/%.framework/%", framework_name, framework_name);

    defer release_string(path);
    terminated_path := Native.terminated_copy(path);

    defer Native.release(terminated_path);
    handle := open_library(terminated_path, 5); // Darwin RTLD_LAZY | RTLD_LOCAL.

    Encoding.require(handle != null, "Objective-C member stub framework could not be loaded");
    defer Encoding.require(close_library(handle) == 0, "Objective-C member stub framework close failed");
    name := Native.terminated_copy(class_name);

    defer Native.release(name);
    class := lookup_class(name);

    Encoding.require(class != null, "Objective-C member stub class is unavailable");
    parent := superclass(class);

    Encoding.require(parent != null, "Objective-C member stub superclass is unavailable");
    size := instance_size(class);
    base_size := instance_size(parent);

    Encoding.require(size >= base_size, "Objective-C class instance size is smaller than its superclass");
    lower := Strings.to_lower_copy(class_name);

    defer release_string(lower);

    return Basic.sprint("_%_data: [%] u8; // % bytes beyond superclass (% - %).", lower, size - base_size, size - base_size, size, base_size);
}

#scope_file
Basic :: #import "Basic";
Strings :: #import "String";
Encoding :: #import "internal/bindings/objc-encoding";
Native :: #import "internal/bindings/native";

system :: #library,system "System";
objc :: #library,system "objc";
open_library :: (path: *u8, mode: s32) -> *void #foreign system "dlopen";
close_library :: (handle: *void) -> s32 #foreign system "dlclose";
lookup_class :: (name: *u8) -> *void #foreign objc "objc_lookUpClass";
superclass :: (class: *void) -> *void #foreign objc "class_getSuperclass";
instance_size :: (class: *void) -> u64 #foreign objc "class_getInstanceSize";

release_string :: (value: string) {
    if value.data context.allocator.proc(.FREE, 0, value.count, value.data, context.allocator.data);
}
