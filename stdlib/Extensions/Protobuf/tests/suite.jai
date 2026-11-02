#import "Basic";
#import "String";
#import "Extensions/Testing";
Fuzz :: #import "Extensions/Fuzz";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("fixtures/all.proto"));
Flat :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("fixtures/all.proto"), MAP = .ARRAY, STRIP_ENUM_PREFIX = true);
Tables :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("fixtures/all.proto"), MAP = .TABLE);
#import "Extensions/Hash_Map";
#import "Hash_Table";

M :: Api.sample.api.All;
C :: M.Child;

hex :: (text: string) -> [] u8 {
    result := NewArray(text.count / 2, u8);
    for i: 0..result.count - 1 {
        a := text[2 * i];
        b := text[2 * i + 1];

        if a <= #char "9" a -= #char "0";
        else a -= #char "a" - 10;
        if b <= #char "9" b -= #char "0";
        else b -= #char "a" - 10;

        result[i] = a * 16 + b;
    }

    return result;
}

from_hex :: ($T: Type, text: string, options := Proto.Decode_Options.{}) -> T, Proto.Error {
    bytes := hex(text);
    defer free(bytes.data);
    return Proto.decode(T, bytes, options);
}

wire_primitives :: () {
    values := u64.[0, 127, 128, 4294967295, 9223372036854775808, 18446744073709551615];
    lengths := s64.[1, 1, 2, 5, 10, 10];
    storage: [10] u8;

    for values {
        w := Proto.Writer.{ bytes = storage };

        Proto.write_varint(*w, it);
        expect_equal(w.pos, lengths[it_index]);
        expect_equal(Proto.varint_size(it), w.pos);
        r := Proto.Reader.{ bytes = .{ w.pos, storage.data } };

        expect_equal(Proto.read_varint(*r), it);
        expect(r.error.kind == .NONE);

        for cut: 0..w.pos - 1 {
            truncated := Proto.Reader.{ bytes = .{ cut, storage.data } };
            Proto.read_varint(*truncated);
            expect(truncated.error.kind == .TRUNCATED);
        }
    }

    signed := s64.[0, -1, 1, -9223372036854775807 - 1, 9223372036854775807];
    for signed expect_equal(Proto.unzigzag(Proto.zigzag(it)), it);

    w := Proto.Writer.{ bytes = storage };

    Proto.write_fixed(*w, 0x0807060504030201, 8);
    for i: 0..7 expect_equal(storage[i], cast(u8) (i + 1));

    r := Proto.Reader.{ bytes = .{ 8, storage.data } };

    expect_equal(Proto.read_fixed(*r, 8), 0x0807060504030201);
} @test

scalar_fixtures :: () {
    // Every scalar kind, written directly from field tags and the wire format.
    value, error := from_hex(M, "08ffffffffffffffffff01100218032004280130023d040302014108070605040302014dfeffffff51feffffffffffffff5d0000c03f6100000000000002406801720268697a02ff00800102");

    require(error.kind == .NONE);
    defer Proto.free(*value);
    expect_equal(value.i32, -1);
    expect_equal(value.i64, 2);
    expect_equal(value.u32_, 3);
    expect_equal(value.u64_, 4);
    expect_equal(value.s32_, -1);
    expect_equal(value.s64_, 1);
    expect_equal(value.f32, 0x01020304);
    expect_equal(value.f64, 0x0102030405060708);
    expect_equal(value.sf32, -2);
    expect_equal(value.sf64, -2);
    expect_equal(value.f, 1.5);
    expect_equal(value.d, 2.25);
    expect(value.b);
    expect_equal(value.text, "hi");
    expect(value.data.count == 2 && value.data[0] == 255);
    expect_equal(cast(s32) value.status, 2);
    encoded := Proto.encode(value);

    defer free(encoded.data);
    again, e := Proto.decode(M, encoded);

    require(e.kind == .NONE);
    defer Proto.free(*again);
    expect(Proto.equal(value, again));
    expect_equal(Proto.size(value), encoded.count);
    negative := Proto.encode(M.{ i32 = -1 });

    defer free(negative.data);
    expected := hex("08ffffffffffffffffff01");

    defer free(expected.data);
    expect(Proto.bytes_equal(negative, expected));
    short, se := from_hex(M, "08ffffffff0f");

    require(se.kind == .NONE);
    defer Proto.free(*short);
    expect_equal(short.i32, -1);
    wide, we := from_hex(M, "2880808080109a01058080808010");

    require(we.kind == .NONE);
    defer Proto.free(*wide);
    expect(wide.s32_ == 0 && wide.packed.count == 1 && wide.packed[0] == 0);
} @test

presence_unknown_groups :: () {
    value, error := from_hex(M, "8801009201001007a80607");

    require(error.kind == .NONE);
    defer Proto.free(*value);
    expect(value.present.has && value.present.value == 0);
    expect(value.child != null);
    expect_equal(value.i64, 7);
    expect_equal(value._unknown.count, 3);
    bytes := Proto.encode(value);

    defer free(bytes.data);
    again, e := Proto.decode(M, bytes);

    require(e.kind == .NONE);
    defer Proto.free(*again);
    expect(Proto.equal(value, again));
    group, ge := from_hex(M, "0b08010c");

    require(ge.kind == .NONE);
    defer Proto.free(*group);
    expect_equal(group.i32, 0);
    expect_equal(group._unknown.count, 4);
    discarded, de := from_hex(M, "a80607", .{ discard_unknown = true });

    require(de.kind == .NONE);
    defer Proto.free(*discarded);
    expect_equal(discarded._unknown.count, 0);
} @test

repeated_and_merge :: () {
    value, error := from_hex(M, "9a01020102980103a00101a201020101aa010161aa010162b20102080192010208019201022002");

    require(error.kind == .NONE);
    defer Proto.free(*value);
    expect_equal(value.packed.count, 3);
    expect(value.packed[0] == -1 && value.packed[1] == 1 && value.packed[2] == -2);
    expect_equal(value.unpacked.count, 3);
    expect(value.labels.count == 2 && value.labels[0] == "a" && value.labels[1] == "b");
    expect(value.children.count == 1 && value.children[0].x == 1);
    expect(value.child.x == 1 && value.child.y == 2);
    patch := hex("08010802980104");

    defer free(patch.data);
    e := Proto.merge(*value, patch);

    require(e.kind == .NONE);
    expect_equal(value.i32, 2);
    expect_equal(value.packed.count, 4);
    Proto.free(*value);
    Proto.free(*value);
} @test

oneof_ownership :: () {
    value, error := from_hex(M, "ca0103616461d20103353535da01020801da01022002");

    require(error.kind == .NONE);
    defer Proto.free(*value);
    expect(value.via.which == .POST && value.via.post.x == 1 && value.via.post.y == 2);
    bytes := hex("e00100");

    defer free(bytes.data);
    e := Proto.merge(*value, bytes);

    require(e.kind == .NONE);
    expect(value.via.which == .CODE && value.via.code == 0);
    encoded := Proto.encode(value);

    defer free(encoded.data);
    expect(Proto.bytes_equal(encoded, bytes));
    expect_equal(M.proto_via_numbers[cast(s64) value.via.which], 28);
} @test

maps_all_modes :: () {
    text :: "ba01070a016112020801ba01070a016112022002c2010408011001c2010408011002c20100";
    value, error := from_hex(M, text);

    require(error.kind == .NONE);
    defer Proto.free(*value);
    expect(value.records.count == 1 && get(*value.records, "a").y == 2);
    expect_equal(cast(s32) get(*value.statuses, 1), 2);
    expect_equal(cast(s32) get(*value.statuses, 0), 0);
    flat, fe := from_hex(Flat.sample.api.All, text);

    require(fe.kind == .NONE);
    defer Proto.free(*flat);
    expect(flat.records.count == 2 && flat.records[0].value.x == 1 && flat.records[1].value.y == 2);
    expect_equal(flat.statuses.count, 3);
    table, te := from_hex(Tables.sample.api.All, text);

    require(te.kind == .NONE);
    defer Proto.free(*table);
    found, child := table_find(*table.records, "a");

    expect(found && child.y == 2);
    encoded := Proto.encode(value);

    defer free(encoded.data);
    again, ae := Proto.decode(M, encoded);

    require(ae.kind == .NONE);
    defer Proto.free(*again);
    expect(Proto.equal(value, again));
    fbytes := Proto.encode(flat);

    defer free(fbytes.data);
    f2, f2e := Proto.decode(Flat.sample.api.All, fbytes);

    require(f2e.kind == .NONE);
    defer Proto.free(*f2);
    expect(Proto.equal(flat, f2));
    tbytes := Proto.encode(table);

    defer free(tbytes.data);
    t2, t2e := Proto.decode(Tables.sample.api.All, tbytes);

    require(t2e.kind == .NONE);
    defer Proto.free(*t2);
    expect(Proto.equal(table, t2));
    expect_equal(Flat.sample.api.All.Status.ACTIVE, cast(Flat.sample.api.All.Status) 1);
} @test

wire_errors :: () {
    inputs := string.["0001", "808080801000", "0e", "0f", "0c", "0b", "0b14", "7201ff", "8080808080808080808000", "80808080808080808002", "720500", "ba0103080112", "88808080800000"];
    kinds := Proto.Error_Kind.[.FIELD_ZERO, .BAD_TAG, .BAD_WIRE_TYPE, .BAD_WIRE_TYPE, .BAD_GROUP, .TRUNCATED, .BAD_GROUP, .BAD_UTF8, .VARINT_TOO_LONG, .VARINT_TOO_LONG, .TRUNCATED, .TRUNCATED, .BAD_TAG];

    for inputs {
        value, error := from_hex(M, it);
        defer Proto.free(*error);
        expect_equal(error.kind, kinds[it_index]);
        expect(value.child == null && value.records.count == 0 && value.text.count == 0);
    }

    _, large := from_hex(M, "720500", .{ max_size = 3 });

    defer Proto.free(*large);
    expect(large.kind == .TOO_LARGE);
    _, depth := from_hex(M, "9201021a00", .{ max_depth = 1 });

    defer Proto.free(*depth);
    expect(depth.kind == .TOO_DEEP);
    expect_equal(depth.path, "child.next");
    bad, e := from_hex(M, "7a01ff");

    require(e.kind == .NONE);
    defer Proto.free(*bad);
    expect_equal(bad.data[0], 255);
} @test

truncation_and_random_bytes :: () {
    // One string nested inside one message means every proper prefix after the first tag truncates.
    bytes := hex("92010a12086162636465666768");

    defer free(bytes.data);

    for cut: 1..bytes.count - 1 {
        value, error := Proto.decode(M, .{ cut, bytes.data });
        defer Proto.free(*error);
        expect(error.kind == .TRUNCATED);
        expect(value.child == null);
    }

    rng: Fuzz.Rng;

    Fuzz.rng_seed(*rng, 123);

    for trial: 0..999 {
        data: [64] u8;
        n := Fuzz.below(*rng, 65);

        for i: 0..n - 1 data[i] = cast,trunc(u8) Fuzz.next(*rng);

        value, error := Proto.decode(M, .{ n, data.data });

        Proto.free(*value);
        Proto.free(*error);
    }
} @test

round_trip_properties :: () {
    rng: Fuzz.Rng;

    Fuzz.rng_seed(*rng, 47);

    for trial: 0..99 {
        original: M;
        original.i32 = cast,trunc(s32) Fuzz.next(*rng);
        original.s64_ = cast,trunc(s64) Fuzz.next(*rng);
        original.u64_ = Fuzz.next(*rng);
        original.text = copy_string("hello");
        original.present = .{ true, cast,trunc(s32) Fuzz.next(*rng) };
        original.child = New(C);
        original.child.label = copy_string("child");
        original.via.which = .PHONE;
        original.via.phone = copy_string("555");
        for i: 0..Fuzz.below(*rng, 20) - 1 array_add(*original.packed, cast,trunc(s32) Fuzz.next(*rng));

        bytes := Proto.encode(original);

        defer free(bytes.data);
        value, error := Proto.decode(M, bytes);

        require(error.kind == .NONE);
        expect(Proto.equal(original, value));
        Proto.free(*original);
        Proto.free(*value);
    }
} @test

append_and_optional_payloads :: () {
    Small :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { optional string name=1; optional bytes data=2; optional bool flag=3; repeated bytes blobs=4; }");
    bytes := hex("0a036f6c640a001202ff001800220161220162");

    defer free(bytes.data);
    value, error := Proto.decode(Small.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*value);
    expect(value.name.has && value.name.value.count == 0);
    expect(value.data.has && value.data.value.count == 2 && value.data.value[0] == 255);
    expect(value.flag.has && !value.flag.value);
    expect_equal(value.blobs.count, 2);
    out: [..] u8;

    defer array_free(out);
    array_add(*out, 123);
    Proto.encode_into(*out, value);
    expect_equal(out[0], 123);
    expect_equal(out.count - 1, Proto.size(value));
    builder: String_Builder;

    defer free_buffers(*builder);
    Proto.encode_into(*builder, value);
    text := builder_to_string(*builder);

    defer free(text);
    expect(Proto.bytes_equal(.{ text.count, text.data }, .{ out.count - 1, out.data + 1 }));
    again, ae := Proto.decode(Small.M, .{ text.count, text.data });

    require(ae.kind == .NONE);
    defer Proto.free(*again);
    expect(Proto.equal(value, again));
} @test

default_depth_limit :: () {
    Chain :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message Node { Node next=1; }");
    root: Chain.Node;
    next := *root;

    for i: 0..100 {
        next.next = New(Chain.Node);
        next = next.next;
    }

    defer Proto.free(*root);
    bytes := Proto.encode(root);

    defer free(bytes.data);
    value, error := Proto.decode(Chain.Node, bytes);

    defer Proto.free(*error);
    expect(error.kind == .TOO_DEEP && value.next == null);
    success, se := Proto.decode(Chain.Node, bytes, .{ max_depth = 101 });

    require(se.kind == .NONE);
    defer Proto.free(*success);
    expect(Proto.equal(root, success));
} @test

utf8_boundaries :: () {
    valid := string.["", "ascii", "é", "€", "😀"];
    for valid {
        value := M.{ text = it };
        bytes := Proto.encode(value);

        defer free(bytes.data);
        decoded, error := Proto.decode(M, bytes);

        require(error.kind == .NONE);
        defer Proto.free(*decoded);
        expect_equal(decoded.text, it);
    }

    // Prefixes put multibyte sequences and invalid tails on both sides of each word boundary.
    for prefix: 0..17 {
        for valid {
            bytes := NewArray(prefix + it.count, u8);

            defer free(bytes.data);
            memset(bytes.data, 97, prefix);
            if it.count memcpy(bytes.data + prefix, it.data, it.count);

            expect(Proto.valid_utf8(bytes));
        }

        bad := string.["c080", "eda080", "f4908080", "c2", "80", "f5808080", "e282", "f09f98"];
        for bad {
            tail := hex(it);

            defer free(tail.data);
            bytes := NewArray(prefix + tail.count, u8);

            defer free(bytes.data);
            memset(bytes.data, 97, prefix);
            memcpy(bytes.data + prefix, tail.data, tail.count);
            expect(!Proto.valid_utf8(bytes));
        }
    }

    invalid := string.["7202c080", "7203eda080", "7204f4908080", "7201c2", "720180", "7204f5808080"];
    for invalid {
        _, error := from_hex(M, it);
        defer Proto.free(*error);
        expect(error.kind == .BAD_UTF8);
    }
} @test

mutation_ownership :: () {
    bytes := hex("72036162639201051203616263aa010161b201051203616263ba010a0a016112051203616263ca0103616263a80607");

    defer free(bytes.data);

    for cut: 0..bytes.count {
        value, error := Proto.decode(M, .{ cut, bytes.data });
        Proto.free(*value);
        Proto.free(*error);
    }

    replacements := u8.[0, 128, 255];

    for *slot: bytes {
        original := slot.*;

        for replacement: replacements {
            slot.* = replacement;
            value, error := Proto.decode(M, bytes);
            Proto.free(*value);
            Proto.free(*error);
        }

        slot.* = original;
    }
} @test

other_schema_properties :: () {
    Scalars :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message Numbers { int32 signed=1; uint64 unsigned=2; optional bool present=3; }");
    Lists :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message Values { repeated sint64 values=1; repeated bool flags=2; }");
    rng: Fuzz.Rng;

    Fuzz.rng_seed(*rng, 56);

    for trial: 0..99 {
        x := Scalars.Numbers.{ signed = cast,trunc(s32) Fuzz.next(*rng), unsigned = Fuzz.next(*rng), present = .{ true, Fuzz.below(*rng, 2) != 0 } };
        bytes := Proto.encode(x);

        defer free(bytes.data);
        y, error := Proto.decode(Scalars.Numbers, bytes);

        require(error.kind == .NONE);
        defer Proto.free(*y);
        expect(Proto.equal(x, y));
        list: Lists.Values;

        defer Proto.free(*list);

        for i: 0..Fuzz.below(*rng, 10) - 1 {
            array_add(*list.values, cast,trunc(s64) Fuzz.next(*rng));
            array_add(*list.flags_, Fuzz.below(*rng, 2) != 0);
        }

        encoded := Proto.encode(list);

        defer free(encoded.data);
        decoded, e := Proto.decode(Lists.Values, encoded);

        require(e.kind == .NONE);
        defer Proto.free(*decoded);
        expect(Proto.equal(list, decoded));
    }
} @test

payload_decode_ownership :: () {
    bytes := hex("72036162637a020102aa0103646566b2010512036b6964f80101");
    value, error := Proto.decode(M, bytes);

    require(error.kind == .NONE);
    free(bytes.data);
    expect_equal(value.text, "abc");
    expect_equal(value.labels[0], "def");
    expect_equal(value.children[0].label, "kid");
    expect_equal(value.data.count, 2);
    expect_equal(value._unknown.count, 0); // Field 31 is a known, renamed field.
    Proto.free(*value);
    Proto.free(*value);
} @test

payload_merge_keeps_backing :: () {
    value, error := from_hex(M, "7203666f6f");

    require(error.kind == .NONE);
    old := value.text;
    bytes := hex("7203626172aa010178");

    defer free(bytes.data);
    merged := Proto.merge(*value, bytes);

    require(merged.kind == .NONE);
    expect_equal(value.text, "bar");
    expect_equal(old, "foo"); // Replaced block slices remain valid until owner free.
    expect_equal(value.labels[0], "x");
    Proto.free(*value);
} @test

payload_redecode_existing :: () {
    value, error := from_hex(M, "aa010178");

    require(error.kind == .NONE);
    bytes := hex("aa01017972017a");

    defer free(bytes.data);

    for 0..7 {
        merged := Proto.merge(*value, bytes);
        require(merged.kind == .NONE);
    }

    expect_equal(value.labels.count, 9);
    expect_equal(value.text, "z");
    Proto.free(*value);
} @test

payload_hand_replacements :: () {
    value, error := from_hex(M, "72036162637a020102");

    require(error.kind == .NONE);
    value.text = copy_string("assigned");
    value.data = NewArray(3, u8);
    Proto.free(*value);
} @test

payload_setters_copy_other_decode :: () {
    value, error := from_hex(M, "72036162637a020102");

    require(error.kind == .NONE);
    other, oe := from_hex(M, "72036465667a020304");

    require(oe.kind == .NONE);
    Proto.set_string(*value, *value.text, other.text);
    Proto.set_bytes(*value, *value.data, other.data);
    Proto.free(*other);
    expect_equal(value.text, "def");
    expect_equal(value.data[0], 3);
    Proto.set_string(*value, *value.text, value.text);
    Proto.set_bytes(*value, *value.data, value.data);
    Proto.free(*value);
} @test

payload_hand_built_free :: () {
    value: M;
    value.text = copy_string("hand");
    value.data = NewArray(3, u8);
    value.child = New(C);
    value.child.label = copy_string("nested");
    array_add(*value.labels, copy_string("repeated"));
    array_add(*value.children, .{ label = copy_string("child") });
    array_add(*value._unknown, 8);
    array_add(*value._unknown, 1);
    Proto.free(*value);
    Proto.free(*value);
} @test

payload_independent_child_merge :: () {
    child, ce := from_hex(C, "12036f6c64");

    require(ce.kind == .NONE);
    value: M;
    value.child = New(C);
    value.child.* = child; // Move the independently owning message, not a slice.
    child = .{};
    bytes := hex("92010512036e6577");

    defer free(bytes.data);
    merged := Proto.merge(*value, bytes);

    require(merged.kind == .NONE);
    expect_equal(value.child.label, "new");
    Proto.free(*value);
} @test

payload_unknown_storage :: () {
    // Unknown field 100, then a known string; repeat merge grows unknown storage.
    bytes := hex("a20603616263720178");

    defer free(bytes.data);
    value, error := Proto.decode(M, bytes);

    require(error.kind == .NONE);

    for 0..5 {
        merged := Proto.merge(*value, bytes);
        require(merged.kind == .NONE);
    }

    expect_equal(value._unknown.count, 42);
    encoded, ee := Proto.encode(value);

    require(ee.kind == .NONE);
    defer free(encoded.data);
    again, ae := Proto.decode(M, encoded);

    require(ae.kind == .NONE);
    expect(Proto.equal(value, again));
    Proto.free(*value);
    Proto.free(*again);
} @test

// Interleaved repeated fields force in-place growth, block resizing and scrap reuse.
arena_growth_roundtrip :: () {
    source: M;

    for 0..2999 {
        array_add(*source.labels, copy_string("label"));
        array_add(*source.children, .{ label = copy_string("child") });
    }

    encoded, ee := Proto.encode(source);

    require(ee.kind == .NONE);
    defer free(encoded.data);

    for 0..1 {
        value, error := Proto.decode(M, encoded);

        require(error.kind == .NONE);
        expect_equal(value.labels.count, 3000);
        expect_equal(value.children.count, 3000);
        expect_equal(value.children[2999].label, "child");
        expect(Proto.equal(source, value));

        // Merging appends more elements to arrays that live in earlier arena blocks.
        merged := Proto.merge(*value, encoded);

        require(merged.kind == .NONE);
        expect_equal(value.labels.count, 6000);
        expect_equal(value.labels[5999], "label");
        Proto.free(*value);
    }

    Proto.free(*source);
} @test
