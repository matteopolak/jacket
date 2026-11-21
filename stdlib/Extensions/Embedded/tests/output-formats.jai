// The linker script, and the BIN, HEX and UF2 images made from an ELF file.
//
//     jaic stdlib/Extensions/Embedded/tests/output-formats.jai -interp

#import "Basic";
#import "String";
#import "File";
Embedded :: #import "Extensions/Embedded";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

put :: (bytes: [] u8, offset: s64, value: u64, size: s64) {
    for 0..size - 1 bytes[offset + it] = cast,no_check(u8) (value >> cast(u64) (8 * it));
}

// An ELF64 file with a loadable segment at `address` holding `data`, and one with no contents that
// the loader skips.
make_elf :: (address: u64, data: [] u8) -> [] u8 {
    elf := NewArray(64 + 2 * 56 + data.count, u8);
    elf[0] = 0x7f;
    elf[1] = #char "E";
    elf[2] = #char "L";
    elf[3] = #char "F";
    elf[4] = 2; // 64-bit
    elf[5] = 1; // little-endian
    put(elf, 24, address + 4, 8); // entry
    put(elf, 32, 64, 8); // program headers
    put(elf, 54, 56, 2);
    put(elf, 56, 2, 2);

    first := 64;

    put(elf, first, 1, 4); // PT_LOAD
    put(elf, first + 8, cast(u64) (64 + 2 * 56), 8); // offset
    put(elf, first + 24, address, 8); // physical address
    put(elf, first + 32, cast(u64) data.count, 8); // file size

    second := first + 56;

    put(elf, second, 1, 4);
    put(elf, second + 24, 0x2000_0000, 8); // RAM: no bytes in the file
    put(elf, second + 40, 4096, 8);

    for 0..data.count - 1 elf[64 + 2 * 56 + it] = data[it];

    return elf;
}

linker_script_test :: () {
    ram_only := Embedded.linker_script(.{ ram = .{ 0x4000_0000, 0x100_0000 } });

    check(contains(ram_only, "ENTRY(_start)"), "entry");
    check(contains(ram_only, "RAM (rwx)  : ORIGIN = 0x40000000, LENGTH = 0x1000000"), "ram region");
    check(!contains(ram_only, "FLASH"), "no flash region");
    check(contains(ram_only, "__data_load = __data_start;"), "data is not copied");
    check(contains(ram_only, "__stack_top = ORIGIN(RAM) + LENGTH(RAM);"), "stack at the end of RAM");
    check(contains(ram_only, "__heap_end = __stack_top - 0x2000;"), "default stack of 8 KiB");
    check(contains(ram_only, "__heap_start = ALIGN(16);"), "the heap starts aligned, as its allocator hands out blocks");

    split := Embedded.linker_script(.{ flash = .{ 0x800_0000, 0x10_0000 }, ram = .{ 0x2000_0000, 0x2_0000 }, stack_size = 0x1000, entry = "reset" });

    check(contains(split, "ENTRY(reset)"), "named entry");
    check(contains(split, "FLASH (rx) : ORIGIN = 0x8000000, LENGTH = 0x100000"), "flash region");
    check(contains(split, "> RAM AT > FLASH"), "data loads from flash");
    check(contains(split, "__data_load = LOADADDR(.data);"), "data load address");
    check(contains(split, "KEEP(*(.vector_table))"), "vectors are kept");
    check(contains(split, "__heap_end = __stack_top - 0x1000;"), "stack size");
}

binary_test :: () {
    data: [6] u8 = .[0xde, 0xad, 0xbe, 0xef, 0x01, 0x02];
    elf := make_elf(0x800_0000, data);
    segments, entry, reason := Embedded.elf_segments(elf);

    check(reason == "", "an ELF file reads");
    check(segments.count == 1, "the empty segment is skipped");
    check(segments[0].address == 0x800_0000 && segments[0].bytes.count == 6, "segment");
    check(entry == 0x800_0004, "entry");

    image := Embedded.binary_image(segments);

    check(image.count == 6 && image[0] == 0xde && image[5] == 0x02, "raw image");

    // Two segments with a gap: zeros between them.
    two: [2] Embedded.Segment;
    two[0] = .{ 0x100, .{ 2, data.data } };
    two[1] = .{ 0x105, .{ 1, data.data + 4 } };
    gap := Embedded.binary_image(two);

    check(gap.count == 6 && gap[1] == 0xad && gap[2] == 0 && gap[4] == 0 && gap[5] == 0x01, "gap is zero");

    _, _, bad := Embedded.elf_segments(.{ 4, data.data });

    check(bad != "", "not an ELF file");
    big_endian := elf;
    big_endian[5] = 2;
    _, _, bad = Embedded.elf_segments(big_endian);
    check(contains(bad, "little-endian"), "big-endian is refused");
}

hex_test :: () {
    data: [6] u8 = .[0xde, 0xad, 0xbe, 0xef, 0x01, 0x02];
    segments := Embedded.elf_segments(make_elf(0x800_0000, data));
    hex := Embedded.intel_hex(segments, 0x800_0004);
    // Extended linear address 0x0800, a data record, the start address and the end.
    expected := ":020000040800F2\n:06000000DEADBEEF0102BF\n:0400000508000004EB\n:00000001FF\n";

    check(hex == expected, tprint("intel hex was\n%", hex));
}

uf2_test :: () {
    data: [6] u8 = .[0xde, 0xad, 0xbe, 0xef, 0x01, 0x02];
    segments := Embedded.elf_segments(make_elf(0x800_0000, data));
    image := Embedded.uf2_image(segments, 0xe48bff56);

    check(image.count == 512, "one block");
    check(read_u32(image, 0) == 0x0a324655 && read_u32(image, 4) == 0x9e5d5157, "start magic");
    check(read_u32(image, 8) == 0x2000, "family id flag");
    check(read_u32(image, 12) == 0x800_0000, "target address");
    check(read_u32(image, 16) == 256, "payload size");
    check(read_u32(image, 20) == 0 && read_u32(image, 24) == 1, "block numbers");
    check(read_u32(image, 28) == 0xe48bff56, "family id");
    check(image[32] == 0xde && image[37] == 0x02 && image[38] == 0, "payload, zero padded");
    check(read_u32(image, 508) == 0x0ab16f30, "end magic");

    plain := Embedded.uf2_image(segments, 0);

    check(read_u32(plain, 8) == 0, "no family flag");
}

read_u32 :: (bytes: [] u8, offset: s64) -> u32 {
    return cast(u32) Embedded.read_le(bytes, offset, 4);
}

write_binary_test :: () {
    data: [6] u8 = .[0xde, 0xad, 0xbe, 0xef, 0x01, 0x02];
    elf := make_elf(0x800_0000, data);
    working := get_working_directory();

    defer free(working);
    directory := tprint("%/embedded-binary-test", working);

    make_directory_if_it_does_not_exist(directory);
    elf_path := tprint("%/image.elf", directory);

    check(write_entire_file(elf_path, xx elf), "wrote the ELF file");

    base := tprint("%/image", directory);
    ok, reason := Embedded.write_binary(elf_path, base, .BIN | .HEX | .UF2);

    check(ok, reason);
    bin, found := read_entire_file(tprint("%.bin", base));

    check(found && bin.count == 6, "bin file");
    hex, hex_found := read_entire_file(tprint("%.hex", base));

    check(hex_found && starts_with(hex, ":020000040800F2"), "hex file");
    uf2, uf2_found := read_entire_file(tprint("%.uf2", base));

    check(uf2_found && uf2.count == 512, "uf2 file");
    free(bin);
    free(hex);
    free(uf2);

    ok, reason = Embedded.write_binary(tprint("%/missing.elf", directory), base, .BIN);
    check(!ok && contains(reason, "could not read"), "a missing file is reported");
}

main :: () {
    linker_script_test();
    binary_test();
    hex_test();
    uf2_test();
    write_binary_test();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }
}
