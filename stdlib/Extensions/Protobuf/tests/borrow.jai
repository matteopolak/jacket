// Borrowed strings and bytes point into the input; free leaves them alone.
#scope_file
#import "Basic";
#import "Extensions/Testing";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { string text=1; bytes data=2; }");

#scope_export

borrowed_payloads :: () {
    bytes := u8.[10, 2, 104, 105, 18, 1, 255];

    value, error := Proto.decode(Api.M, bytes, .{ borrow = true });

    defer Proto.free(*value);
    require(error.kind == .NONE);
    expect(value.text.data == bytes.data + 2 && value.text == "hi");
    expect(value.data.data == bytes.data + 6 && value.data[0] == 255);

    // Replacing a borrowed field and failure cleanup must not free the input either.
    again := Proto.merge(*value, u8.[10, 1, 120]);

    expect(again.kind == .NONE && value.text == "x");

    broken := u8.[10, 2, 104, 105, 18, 2, 255];
    empty, failure := Proto.decode(Api.M, broken, .{ borrow = true });

    defer Proto.free(*failure);
    expect(failure.kind == .TRUNCATED && empty.text.count == 0);
} @test

Tree :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message Node { string name=1; Node child=2; repeated Node kids=3; }");

// Counts allocations and live ones, and catches frees of borrowed input bytes.
Borrow_Count :: struct {
    input: [] u8;
    allocations: s64;
    live: s64;
    input_frees: s64;
}

borrow_count :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    state := cast(*Borrow_Count) data;
    parent := context.default_allocator;

    if mode == .FREE {
        if !old return null;

        if old >= state.input.data && old < state.input.data + state.input.count {
            state.input_frees += 1;
            return null;
        }

        state.live -= 1;
    } else if mode == .ALLOCATE || (mode == .RESIZE && !old) {
        state.allocations += 1;
        state.live += 1;
    } else if mode == .RESIZE {
        state.allocations += 1;
    }

    return parent.proc(mode, size, old_size, old, parent.data);
}

borrowed_messages_share_the_arena :: () {
    root: Tree.Node;
    root.name = "root";
    kids: [100] Tree.Node;
    leaf := Tree.Node.{ name = "leaf" };

    for *kids {
        it.name = "kid";
        it.child = *leaf;
        array_add(*root.kids, it.*);
    }

    defer array_free(root.kids);
    bytes := Proto.encode(root);

    defer free(bytes.data);

    state := Borrow_Count.{ input = bytes };
    value: Tree.Node;
    error: Proto.Error;

    ctx := context;
    ctx.allocator = .{ borrow_count, *state };

    push_context ctx {
        value, error = Proto.decode(Tree.Node, bytes, .{ borrow = true });
        require(error.kind == .NONE);
        expect(value.kids.count == 100 && value.kids[99].child.name == "leaf");
        expect(value.kids[0].name.data >= bytes.data && value.kids[0].name.data < bytes.data + bytes.count);
        // 200 nested messages and a 100-element array come from a few arena blocks.
        expect(state.allocations <= 4, tprint("% allocations", state.allocations));
        Proto.free(*value);
    }

    expect(state.live == 0, tprint("% live allocations", state.live));
    expect(state.input_frees == 0, tprint("% frees of borrowed input", state.input_frees));
} @test

skip_utf8_accepts_any_bytes :: () {
    bytes := u8.[10, 2, 0xc0, 0x80];

    _, checked := Proto.decode(Api.M, bytes);

    defer Proto.free(*checked);
    expect(checked.kind == .BAD_UTF8);

    copied, error := Proto.decode(Api.M, bytes, .{ skip_utf8 = true });

    defer Proto.free(*copied);
    require(error.kind == .NONE);
    expect(copied.text.count == 2 && copied.text[0] == 0xc0 && copied.text.data != bytes.data + 2);

    borrowed, borrow_error := Proto.decode(Api.M, bytes, .{ borrow = true, skip_utf8 = true });

    defer Proto.free(*borrowed);
    require(borrow_error.kind == .NONE);
    expect(borrowed.text.data == bytes.data + 2);
} @test
