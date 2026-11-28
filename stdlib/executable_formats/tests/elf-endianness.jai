#import "Basic";
E :: #import "executable_formats";

main :: () {
    // The identification of a 64-bit big-endian file, as the ELF format writes it: class 2, data 2.
    wire := cast(*u8) alloc(128);

    defer free(wire);
    for 0..127 wire[it] = 0;

    wire[0] = 0x7f;
    wire[1] = #char "E";
    wire[2] = #char "L";
    wire[3] = #char "F";
    wire[4] = 2;
    wire[5] = 2;
    wire[6] = 1;

    header := cast(*E.Elf_Header_64) wire;

    assert(header.ident.class == ._64_BIT);
    assert(header.ident.data == .MSB);
    assert(cast(u8) E.Elf_Endianness.MSB == 2 && cast(u8) E.Elf_Endianness.LSB == 1);

    // Only little-endian files are parsed; the big-endian one is rejected, not misread.
    wire[5] = 1;
    assert(header.ident.data == .LSB);
    wire[5] = 2;
    ok, _, _ := E.parse_elf_header_and_program_header_table(string.{ 128, wire });

    assert(!ok);
    print("ok\n");
}
