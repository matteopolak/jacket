#import "Basic";
B :: #import "Bindings_Generator";
P :: #import "POSIX";

provider :: #foreign_library "../../../internal/developer-tools/generator_platform";
gp_library_open :: (path: *u8) -> *void #foreign provider;
gp_library_symbol :: (image: *void, name: *u8) -> s32 #foreign provider;
gp_library_close :: (image: *void) #foreign provider;

main :: () {
    suffix := ifx OS == .MACOS then ".dylib" else ".so";
    path := to_c_string(tprint("%../../../../target/libruntime_ci_probe%", #filepath, suffix));

    defer free(path);
    name := to_c_string("runtime_ci_value");

    defer free(name);

    for 0..1 {
        image := gp_library_open(path);

        assert(image != null && gp_library_symbol(image, name) != 0);
        value := cast(#type () -> s32 #c_call) P.dlsym(image, name);

        assert(value != null && value() == 73);
        gp_library_close(image);
    }

    paths := string.[tprint("%../../../../target", #filepath)];
    library := B.os_load_library(paths, "probe", "runtime_ci_probe", false);

    assert(library != null && library.is_shared_library);
    assert(B.os_symbol_exists_in_library(library, "runtime_ci_value"));
    assert(!B.os_symbol_exists_in_library(library, "absent_runtime_ci_value"));
    value := cast(#type () -> s32 #c_call) P.dlsym(library.handle, name);

    assert(value != null && value() == 73);
    gp_library_close(library.handle);
    free(library.identifier);
    free(library.full_path);
    free(library);
    print("ok\n");
}
