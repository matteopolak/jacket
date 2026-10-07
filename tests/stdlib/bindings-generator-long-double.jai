// Bindings_Generator with `use_jaic_long_double` (the default): C functions and structs using
// `long double` are bound with jaic's Long_Double extension and work. The library and the
// bindings are made for the compile target, so this also runs as an x86-64 build (x87 long
// double) under Rosetta from crates/jaic-cli/tests/native.rs (c_long_double). The C fixture is
// in tests/native/c-long-double. Skips (printing ok) when clang or libclang is unavailable.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";
#import "Scratch_Directory";
#import "Extensions/Long_Double";

FIXTURE :: #filepath;

clang_target :: () -> string {
    arch := ifx CPU == .X64 then "x86_64" else "aarch64";
    if OS == .MACOS return tprint("%-apple-darwin", ifx CPU == .X64 then "x86_64" else "arm64");
    return tprint("%-linux-gnu", arch);
}

generate :: () -> string {
    #if OS != .MACOS && OS != .LINUX return "";
    fixture := tprint("%/../native/c-long-double", FIXTURE);
    target := clang_target();
    // Kept: the program loads the library from here when it runs.
    dir := scratch_directory(tprint("bindgen-long-double-%", target));
    ext := ifx OS == .MACOS then "dylib" else "so";
    library_path := tprint("%/liblongdouble.%", dir, ext);
    result := run_command("clang", "-target", target, "-shared", "-fPIC", "-o", library_path,
        tprint("%/longdouble.c", fixture), capture_and_return_output = true);
    if result.exit_code != 0 return "";

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, tprint("%/longdouble.h", fixture));
    array_add(*opts.extra_clang_arguments, "-target", target);
    array_add(*opts.libpaths, dir);
    array_add(*opts.libnames, "liblongdouble");
    builder, ok := generate_bindings(opts);
    if !ok return "";
    return builder_to_string(*builder);
}

BINDINGS :: #run generate();
HAVE_BINDINGS :: BINDINGS.count > 0;

#if HAVE_BINDINGS {
    #insert BINDINGS;
}

main :: () {
    #if HAVE_BINDINGS {
        // Wide: Long_Double everywhere, nothing stripped. Otherwise long double is double.
        assert(contains(BINDINGS, "ld_third :: () -> Long_Double") == LONG_DOUBLE_IS_WIDE);
        assert(contains(BINDINGS, "#import \"Extensions/Long_Double\"") == LONG_DOUBLE_IS_WIDE);
        assert(contains(BINDINGS, "ld_spill :: "));
        assert(contains(BINDINGS, "box_apply :: "));

        third := ld_third();
        assert(ld_beyond_double(third) == cast(s32) LONG_DOUBLE_IS_WIDE);
        assert(ld_add(third, third) == third + third);
        assert(ld_mix(2, 0.5, 1.25, 2.5, 3) == 10.5);
        assert(ld_to_double(third) == cast(float64) third);
        box := box_scale(.{ 1.5 }, 4);
        assert(box.v == 6);
        tagged := tagged_bump(.{ 5, third });
        assert(tagged.tag == 6 && tagged.v == third * 2);
        pair := pair_swap(.{ 1, third });
        assert(pair.a == third && pair.b == 1);
        assert(ld_spill(1, 2, 3, 4, 5, 6, 7, 8, 9, third, 3, 0.25) == 46.25);
    }
    print("ok\n");
}
