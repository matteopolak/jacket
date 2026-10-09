// executable_formats recognises ar archives, ELF, Mach-O, universal binaries, COFF objects and PE
// images from their first bytes (synthetic headers here, so it runs on every host); the two small
// helper modules generate_c_header and Project_Generator are checked with them.
#import "Basic";
#import "executable_formats";
#import "generate_c_header";
#import "Project_Generator";

blank :: (count: s64) -> string {
    bytes := NewArray(count, u8);
    return string.{ count, bytes.data };
}

put :: (text: string, offset: s64, bytes: ..u8) {
    for bytes text.data[offset + it_index] = it;
}

main :: () {
    // ar archives need the whole magic.
    assert(is_ar("!<arch>\nfile.o/"));
    assert(!is_ar("!<arch>") && !is_ar("") && !is_ar("!<arch> \nxx"));

    // ELF: magic plus a header's worth of bytes.
    elf := blank(64);
    put(elf, 0, 0x7f, #char "E", #char "L", #char "F");
    assert(is_elf(elf) && !is_macho(elf) && !is_coff(elf) && !is_apple_universal(elf));
    assert(!is_elf(blank(63)) && !is_elf(slice_to(elf, 63)), "a header shorter than 64 bytes is not an ELF");
    put(elf, 3, 0x47);
    assert(!is_elf(elf), "a wrong magic");

    // 64-bit Mach-O, little endian.
    macho := blank(32);
    put(macho, 0, 0xcf, 0xfa, 0xed, 0xfe);
    assert(is_macho(macho) && !is_elf(macho));
    assert(!is_macho(slice_to(macho, 31)));

    // Universal (fat) binaries have their magic big-endian on disk.
    fat := blank(8);
    put(fat, 0, 0xca, 0xfe, 0xba, 0xbe);
    assert(is_apple_universal(fat) && !is_macho(fat));
    assert(!is_apple_universal(slice_to(fat, 7)));

    // A COFF object starts with the machine type (AMD64 here); a PE image wraps one.
    coff := blank(20);
    put(coff, 0, 0x64, 0x86);
    assert(is_coff(coff) && !is_coff(slice_to(coff, 19)));
    pe := blank(64 + 4 + 20);
    put(pe, 0, #char "M", #char "Z");
    put(pe, 60, 64, 0, 0, 0);
    put(pe, 64, #char "P", #char "E", 0, 0);
    put(pe, 68, 0x64, 0x86);
    found, start := is_pe(pe);
    assert(found && start == 68, "pe %", start);
    assert(!is_coff(pe) && !is_elf(pe));
    put(pe, 64, #char "X");
    assert(!is_pe(pe), "a PE signature is required");
    put(pe, 64, #char "P");
    put(pe, 60, 200, 0, 0, 0);
    assert(!is_pe(pe), "an offset past the end is rejected");
    put(pe, 60, 64, 0, 0, 0);
    put(pe, 0, #char "N");
    assert(!is_pe(pe), "the MZ stub is required");
    assert(!is_pe(blank(10)));

    // Zero padded names lose their padding only.
    padded := u8.[#char "a", #char "b", 0, #char "c", 0, 0, 0];
    assert(string_from_zero_padded_array(padded) == "ab\0c");
    assert(string_from_zero_padded_array(u8.[0, 0]) == "");
    empty: [] u8;
    assert(string_from_zero_padded_array(empty) == "");

    // C names that clash with keywords get an underscore.
    assert(avoid_reserved_names("signed") == "_signed" && avoid_reserved_names("value") == "value" && avoid_reserved_names("") == "");

    // Project_Generator keeps a module-relative folder and refuses to climb out of the tree.
    assert(get_folder_for_path("src/game/main.jai") == "src/game");
    assert(get_folder_for_path("main.jai") == "");
    assert(get_folder_for_path("modules/Math/module.jai") == "modules/Math");
    assert(get_folder_for_path("/home/me/jai/modules/Basic/module.jai") == "modules/Basic");
    assert(get_folder_for_path("C:\\work\\modules\\Simp\\a.jai") == "modules\\Simp");
    assert(get_folder_for_path("../outside/file.jai") == "");
    assert(get_folder_for_path("a/b/") == "a/b");
    print("ok\n");
}

slice_to :: (text: string, count: s64) -> string {
    result := text;
    result.count = count;
    return result;
}
