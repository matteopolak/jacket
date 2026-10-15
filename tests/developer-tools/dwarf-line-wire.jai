// Private parser dependency: actual DWARF4 line sections, independently decoded SDK rows.
#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "../../target/developer-tools-dwarf-line-wire.jai";
#load "../../target/developer-tools-dwarf-line-wire-oracle.jai";

main :: () {
    assert(size_of(D.Line_Program_Header) == 16);
    check_native_line_wire();
    bytes, bytes_ok := read_entire_file(tprint("%../../target/developer-tools-dwarf-line-x64-o0.bin", #filepath));

    assert(bytes_ok);
    defer free(bytes);
    original, original_ok := line_wire_decode(cast([] u8) bytes, 0, 8);

    assert(original_ok);
    defer line_wire_release(*original);
    assert(cast(*u8) *original.header.header_length - cast(*u8) *original.header == 6);
    assert(original.dirs.count > 0 && original.files.count > 0);
    for directory: original.dirs assert(directory.data >= bytes.data && directory.data < bytes.data + bytes.count);
    for file: original.files assert(file.path.data >= bytes.data && file.path.data < bytes.data + bytes.count);

    for index: s64.[10, 11, 14, 15] {
        saved := bytes[index];

        bytes[index] = 0;
        rejected, rejected_ok := line_wire_decode(cast([] u8) bytes, 0, 8);

        assert(!rejected_ok && rejected.rows.count == 0 && rejected.files.count == 0 && rejected.dirs.count == 0);
        bytes[index] = saved;
    }

    version := bytes[4];

    bytes[4] = 5;
    rejected, rejected_ok := line_wire_decode(cast([] u8) bytes, 0, 8);

    assert(!rejected_ok && rejected.rows.count == 0);
    bytes[4] = version;
    program := 10 + cast(s64) original.header.header_length;
    first, second := bytes[program], bytes[program + 1];

    bytes[program] = 0;
    bytes[program + 1] = 0;
    rejected, rejected_ok = line_wire_decode(cast([] u8) bytes, 0, 8);
    assert(!rejected_ok && rejected.rows.count == 0 && rejected.files.count == 0);
    bytes[program], bytes[program + 1] = first, second;
    for offset: s64.[-1, bytes.count + 1, 0x7fffffffffffffff] {
        rejected, rejected_ok = line_wire_decode(cast([] u8) bytes, offset, 8);
        assert(!rejected_ok && rejected.rows.count == 0);
    }

    for width: s64.[0, 1, 16] {
        rejected, rejected_ok = line_wire_decode(cast([] u8) bytes, 0, width);
        assert(!rejected_ok && rejected.rows.count == 0);
    }

    combined: String_Builder;

    defer free_buffers(*combined);
    append(*combined, bytes);
    append(*combined, bytes);
    double := builder_to_string(*combined);

    defer free(double);
    head, head_ok := line_wire_decode(cast([] u8) double, 0, 8);

    assert(head_ok && head.next_offset == bytes.count);
    defer line_wire_release(*head);
    tail, tail_ok := line_wire_decode(cast([] u8) double, head.next_offset, 8);

    assert(tail_ok && tail.next_offset == double.count && tail.rows.count == head.rows.count);
    defer line_wire_release(*tail);
    print("ok\n");
}
