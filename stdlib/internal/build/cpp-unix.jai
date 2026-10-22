#scope_module
default_compiler :: (cpp: bool) -> string {
    if cpp return "c++";
    return "cc";
}

default_archiver :: () -> string {
    return "ar";
}

environment_tool :: (name: string, fallback: string) -> string {
    key := to_c_string(name);

    defer free(key);
    found := Native.getenv(key);
    if !found || found[0] == 0 return fallback;

    return to_string(found);
}

create_intermediate_directory :: (basename: string) -> string, bool {
    template := sprint("%.build-XXXXXX", basename);

    defer free(template);
    native := to_c_string(template);

    defer free(native);
    if !Native.mkdtemp(native) return .{}, false;

    return copy_string(to_string(native)), true;
}

cleanup_file :: (path: string, failure: *string) -> bool {
    native := to_c_string(path);
    defer free(native);
    if Native.unlink(native) == 0 || cast(s32) Native.errno() == Native.ENOENT return true;
    return remember_failure("BuildCpp intermediate file cleanup failed", failure);
}

append_compile_arguments :: (args: *[..] string, compiler: string, file: string, object: string, type: Build_Type, debug: bool, extra: [] string) -> bool {
    if debug {
        array_add(args, "-O0");
        array_add(args, "-g");
    } else array_add(args, "-O2");

    if type == .DYNAMIC_LIBRARY array_add(args, "-fPIC");
    for arg: extra array_add(args, arg);

    array_add(args, "-c");
    array_add(args, file);
    array_add(args, "-o");
    array_add(args, object);

    return true;
}

append_link_arguments :: (args: *[..] string, compiler: string, staged: string, intermediates: string, type: Build_Type, debug: bool, extra: [] string, objects: [] string, libraries: [] string) -> bool {
    if type == .DYNAMIC_LIBRARY {
        #if OS == .MACOS array_add(args, "-dynamiclib");
        else array_add(args, "-shared");
    }

    for arg: extra array_add(args, arg);

    array_add(args, "-o");
    array_add(args, staged);
    for object: objects array_add(args, object);
    for library: libraries array_add(args, library);

    return true;
}

#scope_file
#import "Basic";
Native :: #import "POSIX";
