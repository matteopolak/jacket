#import "Basic";
#import "File";
B :: #import "Bindings_Generator";
P :: #import "POSIX";
S :: #import "String";

errors: s64;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

release :: (names: [] string) {
    for name: names free(name);
    array_free(names);
}

main :: () {
    path := tprint("%../../../../target/generator-platform-macho.o", #filepath);
    ok, names := B.os_get_symbol_names_from_file(path);

    assert(ok && names.count == 3);
    defer release(names);
    found := false;
    for name: names if name == "_gp_add" found = true;

    assert(found);
    bytes, read_ok := read_entire_file(path);

    assert(read_ok);
    defer free(bytes);
    content_ok, content_names := B.os_get_symbol_names_from_file_content(bytes, tprint("%../../../../target/diagnostic-only.o", #filepath));

    assert(content_ok && content_names.count == names.count);
    defer release(content_names);

    for name, i: names {
        assert(name == content_names[i]);
        assert(name.data != content_names[i].data);
    }

    for suffix: string.["elf.o", "coff.o", "a.a", "macho-o2.o"] {
        other_path := tprint("%../../../../target/generator-platform-%", #filepath, suffix);
        other_ok, others := B.os_get_symbol_names_from_file(other_path);

        assert(other_ok && others.count == 3);
        defer release(others);
        decorated := suffix == "a.a" || suffix == "macho-o2.o";
        expected := ifx decorated then "_gp_add" else "gp_add";
        found_other := false;

        for name: others if name == expected found_other = true;

        assert(found_other);
    }

    push_context {
        context.logger = capture_error;

        for size: int.[0, 1, 4, 16, 32] {
            bad_ok, bad_names := B.os_get_symbol_names_from_file_content(S.slice(bytes, 0, size), "truncated original object");
            assert(!bad_ok && bad_names.count == 0);
        }

        bad_ok, bad_names := B.os_get_symbol_names_from_file("bad\0path");

        assert(!bad_ok && bad_names.count == 0);
    }

    assert(errors == 6);

    push_context {
        context.logger = capture_error;
        invalid_ok, invalid_names := B.os_get_symbol_names_from_file_content(string.{ -1, null }, "invalid count");
        assert(!invalid_ok && invalid_names.count == 0);
    }

    root := tprint("%../../../../target", #filepath);
    paths := string.[root];
    static_lib := B.os_load_library(paths, "Fixture_Static", "generator-platform-a", true);

    assert(static_lib != null && !static_lib.is_shared_library && static_lib.is_system_library);
    assert(static_lib.identifier == "Fixture_Static");
    assert(B.os_symbol_exists_in_library(static_lib, "gp_add"));
    assert(!B.os_symbol_exists_in_library(static_lib, "gp_external"));
    assert(!B.os_symbol_exists_in_library(static_lib, "gp_private"));
    active: B.Generator;

    array_add(*active.libraries, static_lib);
    defer array_free(active.libraries);

    push_context {
        context.generator = *active;
        assert(B.find_lib_for_foreign_name("gp_add") == static_lib);
    }

    release(static_lib.symbol_names);
    free(static_lib.identifier);
    free(static_lib.full_path);
    free(static_lib);

    shared := B.os_load_library(paths, "Fixture_Shared", "libgenerator_platform_fixture", false);

    assert(shared != null && shared.is_shared_library && !shared.is_system_library);
    assert(B.os_symbol_exists_in_library(shared, "gp_add"));
    assert(!B.os_symbol_exists_in_library(shared, "gp_absent"));
    assert(!B.os_symbol_exists_in_library(shared, "gp_add\0suffix"));
    assert(!B.os_symbol_exists_in_library(null, "gp_add"));
    cname := to_c_string("gp_add");

    defer free(cname);
    add := cast(#type (x: s32) -> s32 #c_call) P.dlsym(shared.handle, cname);

    assert(add != null && add(22) == 42);
    function: B.Function;

    assert(B.function_is_missing_foreign_library(*function));
    function.library = shared;
    assert(!B.function_is_missing_foreign_library(*function));
    assert(B.function_is_missing_foreign_library(null));
    assert(P.dlclose(shared.handle) == 0);
    free(shared.identifier);
    free(shared.full_path);
    free(shared);

    foreign: string = "__gp_name";

    B.fix_macos_foreign_name(*foreign);
    assert(foreign == "_gp_name");
    empty: string;

    B.fix_macos_foreign_name(*empty);
    B.fix_macos_foreign_name(null);
    assert(!empty);

    push_context {
        context.logger = capture_error;
        missing := B.os_load_library(paths, "Missing", "does-not-exist", false);

        assert(missing == null);
    }

    assert(errors == 8);

    print("ok\n");
}
