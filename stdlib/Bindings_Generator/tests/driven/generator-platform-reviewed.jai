#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

errors: int;

report :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

release :: (names: [] string) {
    for name: names free(name);
    array_free(names);
}

main :: () {
    first, second: B.Library_Info;
    first.flags = .LINK_ALWAYS;
    first.symbol_names = string.["original"];
    second.symbol_names = string.["original", "second"];
    generator: B.Generator;

    array_add(*generator.libraries, *first);
    array_add(*generator.libraries, *second);
    defer array_free(generator.libraries);

    push_context {
        context.generator = *generator;
        assert(B.find_lib_for_foreign_name("original") == *first);
        assert(B.find_lib_for_foreign_name("second") == *second);
        assert(B.find_lib_for_foreign_name("absent") == null);
    }

    source := tprint("%../../../../target/generator-platform-macho.o", #filepath);
    bytes, read_ok := read_entire_file(source);

    assert(read_ok);
    defer free(bytes);
    path := sprint("%../../../../target/generator-platform-cache.o", #filepath);

    defer free(path);

    assert(write_entire_file(path, bytes));
    ok, owned := B.os_get_symbol_names_from_file_fixed_and_cached(path);

    assert(ok && owned.count == 3);
    assert(owned[0] == "gp_add");
    assert(B.library_symbol_cache.count == 1);
    owned[0].data[0] = #char "X";
    release(owned);
    assert(write_entire_file(path, "invalid replacement object"));
    hit_ok, borrowed := B.os_get_symbol_names_from_file_fixed_and_cached(path);

    assert(hit_ok && borrowed[0] == "gp_add");
    borrowed[0].data[0] = #char "Y";
    again_ok, again := B.os_get_symbol_names_from_file_fixed_and_cached(path);

    assert(again_ok && again.data == borrowed.data && again[0] == "Yp_add");
    B.library_symbol_cache[0].os = .WINDOWS;

    push_context {
        context.logger = report;
        mismatch_ok, mismatch := B.os_get_symbol_names_from_file_fixed_and_cached(path);
        assert(!mismatch_ok && mismatch.count == 0 && B.library_symbol_cache.count == 1);
    }

    B.library_symbol_cache[0].os = OS;
    B.library_symbol_cache[0].cpu = ifx CPU == .X64 then .ARM64 else .X64;

    push_context {
        context.logger = report;
        mismatch_ok, mismatch := B.os_get_symbol_names_from_file_fixed_and_cached(path);
        assert(!mismatch_ok && mismatch.count == 0 && B.library_symbol_cache.count == 1);
    }

    B.library_symbol_cache[0].cpu = CPU;
    assert(errors == 2);
    assert(B.library_symbol_cache[0].os == OS && B.library_symbol_cache[0].cpu == CPU);
    assert(B.library_symbol_cache[0].full_path == path);
    assert(write_entire_file(path, bytes));
    different_path := sprint("%../../../../target/../../../../target/generator-platform-cache.o", #filepath);

    defer free(different_path);

    {
        push_allocator(temp);
        different_ok, different := B.os_get_symbol_names_from_file_fixed_and_cached(different_path);
        assert(different_ok && B.library_symbol_cache.count == 2 && different[0] == "gp_add");
    }

    reset_temporary_storage();
    retained_ok, retained := B.os_get_symbol_names_from_file_fixed_and_cached(different_path);

    assert(retained_ok && retained[0] == "gp_add");

    generic, platform, paths_ok := B.get_libc_paths(.MACOS, CPU);

    assert(paths_ok && generic == platform && generic.count > 0);
    assert(generic.data != platform.data);
    defer free(generic);
    defer free(platform);
    unsupported, unsupported_platform, supported := B.get_libc_paths(.WASM, CPU);

    assert(!supported && !unsupported && !unsupported_platform);

    for entry: B.library_symbol_cache {
        free(entry.full_path);
        release(entry.symbol_names);
    }

    array_free(B.library_symbol_cache);
    print("ok\n");
}
