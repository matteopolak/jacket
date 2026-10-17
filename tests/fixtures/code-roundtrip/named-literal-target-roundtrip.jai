#import "Basic";
C :: #import "Compiler";

Detail :: struct {
    count: s64;
    note: string;
}

Packet :: struct {
    tag: u8;
    detail: Detail;
    cells: [2] Detail;
}

edit_and_copy :: (code: Code) -> Code {
    root, nodes := C.compiler_get_nodes(code);
    for node: nodes {
        if node.kind == .LITERAL {
            literal := cast(*C.Code_Literal) node;
            if literal.value_type == .NUMBER && literal._s64 == 13 literal._s64 = 17;
        }
    }

    return C.compiler_get_code(root, code);
}

make_packet :: #insert #run edit_and_copy(#code () -> Packet {
    return .{ tag = 2, detail.count = 13, detail.note = "a\"b", cells[1].count = 29 };
});

main :: () {
    packet := make_packet();

    assert(type_of(packet) == Packet);
    assert(packet.tag == 2 && packet.detail.count == 17 && packet.detail.note == "a\"b");
    assert(packet.cells[0].count == 0 && packet.cells[1].count == 29);
    print("named literal targets ok\n");
}
