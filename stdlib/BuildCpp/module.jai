#assert OS == .MACOS || OS == .LINUX, "BuildCpp currently supports Unix host toolchains";

Build_Type :: enum {
    STATIC_LIBRARY;
    DYNAMIC_LIBRARY;
    EXECUTABLE;
    OBJ_FILE;
}

build_cpp_static_lib :: #bake_arguments build_cpp(type = .STATIC_LIBRARY);
build_cpp_dynamic_lib :: #bake_arguments build_cpp(type = .DYNAMIC_LIBRARY);
build_cpp_executable :: #bake_arguments build_cpp(type = .EXECUTABLE);

#load "../internal/build/cpp.jai";
