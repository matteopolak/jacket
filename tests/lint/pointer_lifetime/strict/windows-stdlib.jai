#import "Basic";
W :: #import "Windows";
R :: #import "Windows_Resources";
P :: #import "Simple_Package";

#assert OS == .WINDOWS;

saved_library: string;
saved_linker: string;

windows_discovery :: () {
    library, linker := R.find_visual_studio_in_a_ridiculous_garbage_way();
    values: [2] string = .[library, linker];

    assert(values.count == 2);
    saved_library = copy_string(library);
    saved_linker = copy_string(linker);
}

Procs :: struct {
    values: [..] int;
}

keymap_copy :: (source: *Procs, dest: *Procs) {
    array_reset(*dest.values);
    for value: source.values array_add(*dest.values, value);
}

package_header :: (package: *P.Create_Package) {
    reset(*package.data);
    header: [64] u8;
    append(*package.data, header.data, header.count);
}

Capture :: struct {
    count: int;
}

capture :: (text: string, data: *void, info: Log_Info) {
    counts := cast(*Capture) data;
    counts.count += 1;
}

icon_context :: () {
    counts: Capture;

    push_context {
        context.logger = capture;
        context.logger_data = *counts;
        log("captured");
    }
}

module_path :: () -> string {
    module_name := u16.[111, 112, 101, 110, 103, 108, 51, 50, 46, 100, 108, 108, 0];
    library := W.GetModuleHandleW(module_name.data);
    path: [32768] u16;
    count := W.GetModuleFileNameW(library, path.data, cast(s32) path.count);
    utf8 := NewArray(131072, u8);
    bytes := W.WideCharToMultiByte(65001, 0, path.data, count, utf8.data, cast(s32) utf8.count, null, null);

    return string.{ bytes, utf8.data };
}

heap_copy_of_temporary :: () -> string {
    return copy_string(copy_temporary_string("copied"));
} @TempAllocates

native_lookup :: () -> *void {
    return W.wglGetProcAddress("glCreateShader\0".data);
}
