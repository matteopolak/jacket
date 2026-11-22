// Flash images from a linked ELF file: raw binary, Intel HEX and UF2. Runs on the host, with no
// `objcopy`.

#import "Basic";
#import "File";

// The formats `write_binary` writes; combine them with `|`.
Output_Formats :: enum_flags u32 {
    BIN; // the bytes, from the lowest load address
    HEX; // Intel HEX
    UF2; // USB Flashing Format, for drag-and-drop bootloaders
}

// Bytes the program places at an address.
Segment :: struct {
    address: u64;
    bytes: [] u8;
}

// Writes `base.bin`, `base.hex` and `base.uf2` as `formats` asks, from the loadable segments of the
// ELF file `elf_path`, at their load addresses (`p_paddr`). `uf2_family` is the board's UF2 family
// id, or 0. Returns false and a reason on failure.
write_binary :: (elf_path: string, base: string, formats: Output_Formats, uf2_family: u32 = 0) -> bool, string {
    elf, read := read_entire_file(elf_path);
    if !read return false, tprint("could not read %", elf_path);

    segments, entry, reason := elf_segments(xx elf);
    if reason return false, reason;

    if formats & .BIN {
        if !write_entire_file(tprint("%.bin", base), xx binary_image(segments)) return false, tprint("could not write %.bin", base);
    }

    if formats & .HEX {
        if !write_entire_file(tprint("%.hex", base), intel_hex(segments, entry)) return false, tprint("could not write %.hex", base);
    }

    if formats & .UF2 {
        if !write_entire_file(tprint("%.uf2", base), xx uf2_image(segments, uf2_family)) return false, tprint("could not write %.uf2", base);
    }

    return true, "";
}

// The `size`-byte little-endian integer at `offset` in `bytes`.
read_le :: (bytes: [] u8, offset: s64, size: s64) -> u64 {
    value: u64;
    for 0..size - 1 value |= (cast(u64) bytes[offset + it]) << cast(u64) (8 * it);
    return value;
}

// The loadable segments of a little-endian ELF32 or ELF64 file, and its entry address. The last
// result is a reason when the file is not one of those.
elf_segments :: (elf: [] u8) -> [] Segment, u64, string {
    none: [] Segment;
    if elf.count < 52 || elf[0] != 0x7f || elf[1] != #char "E" || elf[2] != #char "L" || elf[3] != #char "F" return none, 0, "not an ELF file";
    if elf[5] != 1 return none, 0, "only little-endian ELF files are supported";
    if elf[4] != 1 && elf[4] != 2 return none, 0, "not a 32-bit or 64-bit ELF file";

    wide := elf[4] == 2;
    word := ifx wide then 8 else 4;
    entry := read_le(elf, 24, word);
    table := cast(s64) read_le(elf, ifx wide then 32 else 28, word);
    entry_size := cast(s64) read_le(elf, ifx wide then 54 else 42, 2);
    count := cast(s64) read_le(elf, ifx wide then 56 else 44, 2);

    segments: [..] Segment;

    for index: 0..count - 1 {
        header := table + index * entry_size;
        if header + entry_size > elf.count return none, 0, "the ELF program header table is cut off";
        if read_le(elf, header, 4) != 1 continue; // PT_LOAD

        offset, address, size: u64;

        if wide {
            offset = read_le(elf, header + 8, 8);
            address = read_le(elf, header + 24, 8);
            size = read_le(elf, header + 32, 8);
        } else {
            offset = read_le(elf, header + 4, 4);
            address = read_le(elf, header + 12, 4);
            size = read_le(elf, header + 16, 4);
        }

        if size == 0 continue;
        if offset + size > cast(u64) elf.count return none, 0, "an ELF segment runs past the end of the file";

        array_add(*segments, .{ address, .{ cast(s64) size, elf.data + offset } });
    }

    return segments, entry, "";
}

// The bytes from the lowest address of `segments` to the end of the highest, gaps zero.
binary_image :: (segments: [] Segment) -> [] u8 {
    none: [] u8;
    if !segments.count return none;

    low := segments[0].address;
    high := low;

    for segments {
        if it.address < low low = it.address;
        if it.address + cast(u64) it.bytes.count > high high = it.address + cast(u64) it.bytes.count;
    }

    image := NewArray(cast(s64) (high - low), u8);

    for segment: segments {
        for 0..segment.bytes.count - 1 image[cast(s64) (segment.address - low) + it] = segment.bytes[it];
    }

    return image;
}

hex_digits :: "0123456789ABCDEF";

// Appends `value` as `digits` upper-case hex digits, leading zeros kept.
append_hex :: (builder: *String_Builder, value: u64, digits: s64) {
    shift := 4 * (digits - 1);
    while shift >= 0 {
        nibble := (value >> cast(u64) shift) & 0xf;
        append(builder, hex_digits[cast(s64) nibble]);
        shift -= 4;
    }
}

// One Intel HEX record: its length, address, type and data, then the checksum that makes the
// bytes sum to zero.
append_hex_record :: (builder: *String_Builder, address: u64, type: u8, data: [] u8) {
    sum := cast(u64) data.count + (address >> 8) + (address & 0xff) + cast(u64) type;
    for data sum += cast(u64) it;

    append(builder, ":");
    append_hex(builder, cast(u64) data.count, 2);
    append_hex(builder, address & 0xffff, 4);
    append_hex(builder, cast(u64) type, 2);
    for data append_hex(builder, cast(u64) it, 2);

    append_hex(builder, (0 - sum) & 0xff, 2);
    append(builder, "\n");
}

// Intel HEX for `segments`, with an extended linear address record whenever the upper 16 address
// bits change and a start address record for `entry`.
intel_hex :: (segments: [] Segment, entry: u64) -> string {
    builder: String_Builder;
    upper: u64 = 0xffff_ffff_ffff_ffff;

    for segment: segments {
        position := 0;
        while position < segment.bytes.count {
            address := segment.address + cast(u64) position;
            if address >> 16 != upper {
                upper = address >> 16;
                bytes: [2] u8 = .[cast,no_check(u8) (upper >> 8), cast,no_check(u8) upper];
                append_hex_record(*builder, 0, 4, bytes);
            }

            // A record stays within one 64 KiB window.
            take := min(min(16, segment.bytes.count - position), cast(s64) (0x10000 - (address & 0xffff)));

            append_hex_record(*builder, address & 0xffff, 0, .{ take, segment.bytes.data + position });
            position += take;
        }
    }

    start: [4] u8 = .[cast,no_check(u8) (entry >> 24), cast,no_check(u8) (entry >> 16), cast,no_check(u8) (entry >> 8), cast,no_check(u8) entry];

    append_hex_record(*builder, 0, 5, start);
    append_hex_record(*builder, 0, 1, .{});

    return builder_to_string(*builder);
}

UF2_PAYLOAD :: 256;
UF2_BLOCK :: 512;

// Writes `value` as 4 little-endian bytes at `offset` in `bytes`.
put_le :: (bytes: [] u8, offset: s64, value: u32) {
    for 0..3 bytes[offset + it] = cast,no_check(u8) (value >> cast(u32) (8 * it));
}

// UF2 blocks of 256 bytes each for `segments`, flagged with `family` when it is not 0.
uf2_image :: (segments: [] Segment, family: u32) -> [] u8 {
    blocks := 0;
    for segments blocks += (it.bytes.count + UF2_PAYLOAD - 1) / UF2_PAYLOAD;

    image := NewArray(blocks * UF2_BLOCK, u8);
    number := 0;

    for segment: segments {
        position := 0;
        while position < segment.bytes.count {
            block := image.data + number * UF2_BLOCK;
            view: [] u8 = .{ UF2_BLOCK, block };

            put_le(view, 0, 0x0a324655);
            put_le(view, 4, 0x9e5d5157);
            put_le(view, 8, ifx family != 0 then 0x2000 else 0);
            put_le(view, 12, cast,no_check(u32) (segment.address + cast(u64) position));
            put_le(view, 16, UF2_PAYLOAD);
            put_le(view, 20, cast(u32) number);
            put_le(view, 24, cast(u32) blocks);
            put_le(view, 28, family);

            take := min(UF2_PAYLOAD, segment.bytes.count - position);
            for 0..take - 1 view[32 + it] = segment.bytes[position + it];

            put_le(view, 508, 0x0ab16f30);

            position += take;
            number += 1;
        }
    }

    return image;
}
