#import "Basic";
#import "Extensions/Testing";
#import,dir "../tools/asm";

forward_labels_and_round_trip :: (t: *Test) {
    bytes, error := assemble("jp end\nstart:\nop 6001\ncall start\nindex end\nend:\nop 00FD\nbyte F0\n");

    defer free(bytes);
    defer free(error);
    expect_equal(error, "");
    expect_equal(bytes, "\x12\x08\x60\x01\x22\x02\xA2\x08\x00\xFD\xF0");
    text := disassemble(bytes);

    defer free(text);
    again, problem := assemble(text);

    defer free(again);
    defer free(problem);
    expect_equal(problem, "");
    expect_equal(again, bytes);
} @test

bad_assembly_is_rejected :: (t: *Test) {
    for source: string.["", "wat 00", "op G000", "byte 100", "jp missing", "x:\nx:\nop 00FD", "byte 01\nop 00FD", "op 10000", "jp FFFF"] {
        bytes, error := assemble(source);

        defer free(bytes);
        defer free(error);
        expect(error.count > 0, source);
        expect_equal(bytes, "");
    }
} @test

comments_and_crlf :: (t: *Test) {
    bytes, error := assemble("; comment\r\n op 00FD ; done\r\n");

    defer free(bytes);
    defer free(error);
    expect_equal(error, "");
    expect_equal(bytes, "\x00\xFD");
} @test
