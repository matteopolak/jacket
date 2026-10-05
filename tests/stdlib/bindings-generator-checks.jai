// Bindings_Generator: compile-time struct layout checks (`generate_compile_time_struct_checks`),
// stripping of header-defined inline functions (`strip_flags = .INLINED_FUNCTIONS`) and the
// `#library` declaration at the end of the file. The C++ fixture is in tests/native/bindgen-checks.
// Skips (printing ok) when clang++ or libclang is unavailable.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

FIXTURE :: #filepath;

generate :: (strip: Strip_Flags, checks: bool) -> string {
    fixture := tprint("%/../native/bindgen-checks", FIXTURE);
    dir := "/tmp/jai_bindgen_checks";
    make_directory_if_it_does_not_exist(dir);
    ext := ifx OS == .MACOS then "dylib" else "so";
    library_path := tprint("%/libchecks.%", dir, ext);
    result, output := run_command("clang++", "-shared", "-fPIC", "-o", library_path, tprint("%/checks.cpp", fixture), capture_and_return_output = true);
    if result.exit_code != 0 return "";

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, tprint("%/checks.h", fixture));
    array_add(*opts.extra_clang_arguments, "-x", "c++", "-std=c++17");
    array_add(*opts.libpaths, dir);
    array_add(*opts.libnames, "libchecks");
    opts.strip_flags = strip;
    opts.generate_compile_time_struct_checks = checks;
    builder, ok := generate_bindings(opts);
    if !ok return "";
    return builder_to_string(*builder);
}

BINDINGS :: #run generate(.INLINED_FUNCTIONS, true);
WITH_INLINE :: #run generate(0, false);
HAVE_CPP :: BINDINGS.count > 0;

#if HAVE_CPP {
    #insert BINDINGS;
}

has :: (text: string, needle: string) -> bool { return contains(text, needle); }

main :: () {
    #if HAVE_CPP {
        // Struct checks, printed after each struct.
        assert(has(BINDINGS, "assert(size_of(Point) == 16, \"Point has size % instead of 16\", size_of(Point));"));
        assert(has(BINDINGS, "instance: Point;"));
        assert(has(BINDINGS, "assert(((cast(*void)(*instance.weight)) - cast(*void)(*instance)) == 8, \"Point.weight has unexpected offset % instead of 8\""));
        assert(has(BINDINGS, "assert(size_of(type_of(Point.x)) == 4, \"Point.x has unexpected size % instead of 4\""));
        assert(has(BINDINGS, "assert(size_of(type_of(Point3.point)) == 16"));
        assert(has(BINDINGS, "#import \"Basic\"; // The generated code uses assert."));
        assert(!has(WITH_INLINE, "#run {"));
        assert(!has(WITH_INLINE, "#import \"Basic\""));

        // The library is declared at the end, in the file scope, by the path it was found at.
        assert(has(BINDINGS, "#scope_file"));
        assert(has(BINDINGS, "libchecks :: #library \"/tmp/jai_bindgen_checks/libchecks\";") || has(BINDINGS, "checks :: #library"));

        // Inline functions defined in the header are stripped; the others stay.
        assert(!has(BINDINGS, "twice_x"));
        assert(!has(BINDINGS, "header_only_helper"));
        assert(!has(BINDINGS, "Constructor"));
        assert(has(BINDINGS, "sum :: "));
        assert(has(BINDINGS, "library_helper"));
        assert(has(WITH_INLINE, "twice_x"));

        // The bindings link and work.
        assert(library_helper(5) == 15);
        p: Point3;
        p.x = 1; p.y = 2; p.z = 3;
        assert(Point.sum(*p) == 3);
        assert(Point3.total(*p) == 6);
        print("ok\n");
    } else {
        print("ok\n");
    }
}
