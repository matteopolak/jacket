// Typed extensions: schema checks, get/set/has/clear/add, wire rules, memory and UTF-8.
#scope_file
#import "Basic";
#import "Extensions/Testing";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
syntax = "proto2";
package ext;
message Base {
    optional int32 id = 1;
    extensions 100 to 199;
}
message Inner {
    optional string s = 1;
    optional int32 n = 2;
    repeated int32 list = 3;
}
enum Kind { A = 0; B = 1; C = 2; }
extend Base {
    optional int32 count = 100 [default = 7];
    optional string label = 101;
    optional bytes blob = 102;
    optional Inner inner = 103;
    repeated int32 nums = 104 [packed = true];
    repeated string tags = 105;
    optional Kind kind = 106;
    optional group Grp = 107 {
        optional int32 g = 1;
    }
    optional sint64 zig = 108;
    repeated Inner inners = 109;
    optional double ratio = 110;
    repeated fixed32 fixeds = 111;
    optional bool flag = 112 [default = true];
    repeated int32 plain = 113;
}
message Holder {
    extend Base {
        optional int32 nested = 150;
    }
}
END);

Edition :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
edition = "2023";
package ed;
message Base { int32 id = 1; extensions 100 to max; }
message Text { string s = 1; }
extend Base {
    string label = 100;
    string raw = 101 [features.utf8_validation = NONE];
    Text text = 102;
    Text wrapped = 103 [features.message_encoding = DELIMITED];
}
END);

Open :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
syntax = "proto2";
message Base { extensions 100 to 199; }
message Text { optional string s = 1; }
extend Base {
    optional string raw = 100;
    optional Text text = 101;
}
END);

Base :: Api.ext.Base;

hex :: (text: string) -> [] u8 {
    digits: [..] u8;

    defer array_free(digits);

    for text if it != #char " " array_add(*digits, it);

    result := NewArray(digits.count / 2, u8);
    for i: 0..result.count - 1 {
        a := digits[2 * i];
        b := digits[2 * i + 1];

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

#scope_export

scalar_extensions :: () {
    m: Base;

    defer Proto.free(*m);

    // Absent: the default, or zero.
    value, found := Proto.get(*m, Api.ext.count);

    expect(!found && value == 7);
    expect(!Proto.has(*m, Api.ext.count));
    expect(Proto.get(*m, Api.ext.flag));

    expect(Proto.set(*m, Api.ext.count, -3).kind == .NONE);
    value, found = Proto.get(*m, Api.ext.count);
    expect(found && value == -3 && Proto.has(*m, Api.ext.count));

    // Setting again replaces rather than accumulating.
    Proto.set(*m, Api.ext.count, 99);
    size := m._unknown.count;

    Proto.set(*m, Api.ext.count, 98);
    expect_equal(m._unknown.count, size);
    expect_equal(Proto.get(*m, Api.ext.count), 98);

    Proto.set(*m, Api.ext.label, "hello");
    Proto.set(*m, Api.ext.blob, u8.[1, 2, 3]);
    Proto.set(*m, Api.ext.kind, Api.ext.Kind.B);
    Proto.set(*m, Api.ext.zig, -5);
    Proto.set(*m, Api.ext.ratio, 2.5);
    Proto.set(*m, Api.ext.flag, false);
    Proto.set(*m, Api.ext.Holder.nested, 4);

    expect(Proto.get(*m, Api.ext.label) == "hello");
    expect_equal(Proto.get(*m, Api.ext.blob).count, 3);
    expect(Proto.get(*m, Api.ext.kind) == .B);
    expect_equal(Proto.get(*m, Api.ext.zig), -5);
    expect_equal(Proto.get(*m, Api.ext.ratio), 2.5);
    expect(!Proto.get(*m, Api.ext.flag));
    expect_equal(Proto.get(*m, Api.ext.Holder.nested), 4);

    // Extensions survive a round trip through the wire.
    bytes, error := Proto.encode(m);

    require(error.kind == .NONE);
    defer free(bytes.data);

    back, failure := Proto.decode(Base, bytes);

    require(failure.kind == .NONE);
    defer Proto.free(*back);

    expect_equal(Proto.get(*back, Api.ext.count), 98);
    expect(Proto.get(*back, Api.ext.label) == "hello");
    expect(Proto.equal(m, back));

    Proto.clear(*back, Api.ext.count);
    expect(!Proto.has(*back, Api.ext.count));
    expect_equal(Proto.get(*back, Api.ext.count), 7);
    expect(Proto.get(*back, Api.ext.label) == "hello");
    expect(!Proto.equal(m, back));

    // Clearing an absent extension changes nothing.
    before := back._unknown.count;

    Proto.clear(*back, Api.ext.count);
    expect_equal(back._unknown.count, before);
} @test

// Last occurrence wins; other fields and wire types are not disturbed.
last_value_wins :: () {
    // count=1, id=5, count=2, then count as a length-delimited field (wrong wire type).
    bytes := hex("a006010805a00602a206020001");

    defer free(bytes.data);

    m, error := Proto.decode(Base, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);

    expect_equal(Proto.get(*m, Api.ext.count), 2);
    expect_equal(m.id.value, 5);

    // Clear drops matching occurrences and keeps the mistyped one.
    Proto.clear(*m, Api.ext.count);
    expect(!Proto.has(*m, Api.ext.count));
    expect_equal(m._unknown.count, 5);

    out, e := Proto.encode(m);

    require(e.kind == .NONE);
    defer free(out.data);
    expect_equal(out.count, 7);
} @test

repeated_extensions :: () {
    m: Base;

    defer Proto.free(*m);

    expect(!Proto.has(*m, Api.ext.nums));

    // Packed and expanded forms both decode, in order.
    packed := hex("c206 02 0102");

    defer free(packed.data);
    expect(Proto.merge(*m, packed).kind == .NONE);

    Proto.add(*m, Api.ext.nums, 10);
    Proto.add(*m, Api.ext.nums, 300);
    Proto.add(*m, Api.ext.plain, 5);
    Proto.add(*m, Api.ext.plain, 6);

    nums := Proto.get(*m, Api.ext.nums);

    defer Proto.free(*nums);
    expect_equal(nums.count, 4);
    expect(nums[0] == 1 && nums[1] == 2 && nums[2] == 10 && nums[3] == 300);

    plain := Proto.get(*m, Api.ext.plain);

    defer Proto.free(*plain);
    expect_equal(plain.count, 2);

    Proto.add(*m, Api.ext.tags, "a");
    Proto.add(*m, Api.ext.tags, "bc");
    tags := Proto.get(*m, Api.ext.tags);

    defer Proto.free(*tags);
    expect(tags.count == 2 && tags[0] == "a" && tags[1] == "bc");

    Proto.add(*m, Api.ext.fixeds, 7);
    fixeds := Proto.get(*m, Api.ext.fixeds);

    defer Proto.free(*fixeds);
    expect(fixeds.count == 1 && fixeds[0] == 7);

    // set replaces everything; an empty set removes the extension.
    Proto.set(*m, Api.ext.nums, s32.[9, 8]);
    again := Proto.get(*m, Api.ext.nums);

    defer Proto.free(*again);
    expect(again.count == 2 && again[0] == 9 && again[1] == 8);
    expect(Proto.has(*m, Api.ext.plain));

    Proto.set(*m, Api.ext.nums, s32.[]);
    expect(!Proto.has(*m, Api.ext.nums));
    Proto.clear(*m, Api.ext.plain);
    expect(!Proto.has(*m, Api.ext.plain));
} @test

message_extensions :: () {
    m: Base;

    defer Proto.free(*m);

    inner := Api.ext.Inner.{ n = .{ true, 5 } };

    Proto.set(*m, Api.ext.inner, inner);

    got, found := Proto.get(*m, Api.ext.inner);

    defer Proto.free(*got);
    expect(found && got.n.has && got.n.value == 5 && !got.s.has);

    // Occurrences merge: later scalars replace, repeated fields append.
    // inner { list: [1] } then inner { list: [2], n: 6 }
    other: Base;

    defer Proto.free(*other);
    merge_bytes := hex("ba06 02 1801 ba06 04 1802 1006");

    defer free(merge_bytes.data);
    require(Proto.merge(*other, merge_bytes).kind == .NONE);

    merged := Proto.get(*other, Api.ext.inner);

    defer Proto.free(*merged);
    expect(merged.list.count == 2 && merged.list[0] == 1 && merged.list[1] == 2);
    expect_equal(merged.n.value, 6);

    // Repeated messages and the group form.
    Proto.add(*m, Api.ext.inners, Api.ext.Inner.{ n = .{ true, 1 } });
    Proto.add(*m, Api.ext.inners, Api.ext.Inner.{ n = .{ true, 2 } });
    items := Proto.get(*m, Api.ext.inners);

    defer Proto.free(*items);
    expect(items.count == 2 && items[0].n.value == 1 && items[1].n.value == 2);

    Proto.set(*m, Api.ext.grp, Api.ext.Grp.{ g = .{ true, 3 } });
    group, ok := Proto.get(*m, Api.ext.grp);

    defer Proto.free(*group);
    expect(ok && group.g.value == 3);

    // The group is written as start and end tags (107 << 3 | 3, 107 << 3 | 4).
    other2: Base;

    defer Proto.free(*other2);
    Proto.set(*other2, Api.ext.grp, Api.ext.Grp.{ g = .{ true, 3 } });
    expect(other2._unknown.count == 6 && other2._unknown[0] == 0xdb && other2._unknown[1] == 0x06 && other2._unknown[4] == 0xdc);
} @test

// A message value that misses a required field is rejected and leaves the extendee alone.
required_message :: () {
    Needs :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; message B { extensions 10 to 20; } message N { required int32 a = 1; } extend B { optional N n = 10; }");
    b: Needs.B;

    defer Proto.free(*b);

    error := Proto.set(*b, Needs.n, Needs.N.{});

    defer Proto.free(*error);

    expect(error.kind == .MISSING_REQUIRED);
    expect_equal(b._unknown.count, 0);

    expect(Proto.set(*b, Needs.n, Needs.N.{ a = .{ true, 1 } }).kind == .NONE);
    expect(Proto.has(*b, Needs.n));
} @test

// Decoded unknown fields live in decode storage; rewriting them must neither leak nor free early.
rewrite_after_decode :: () {
    bytes := hex("0805 a00601 aa06026869");

    defer free(bytes.data);

    m, error := Proto.decode(Base, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);

    for 1..20 {
        Proto.set(*m, Api.ext.count, cast(s32) it);
        Proto.set(*m, Api.ext.label, "a longer label than before to force growth");
        Proto.add(*m, Api.ext.tags, "tag");
    }

    expect_equal(Proto.get(*m, Api.ext.count), 20);
    tags := Proto.get(*m, Api.ext.tags);

    defer Proto.free(*tags);
    expect_equal(tags.count, 20);

    Proto.clear(*m, Api.ext.label);
    Proto.clear(*m, Api.ext.tags);
    Proto.clear(*m, Api.ext.count);
    expect_equal(m._unknown.count, 0);
    expect_equal(m.id.value, 5);

    // Decoding again on top of the value keeps working.
    require(Proto.merge(*m, bytes).kind == .NONE);
    expect_equal(Proto.get(*m, Api.ext.count), 1);
} @test

// Hand-built messages free their own unknown bytes, and a message get releases its blocks.
build_and_free :: () {
    m: Base;

    Proto.set(*m, Api.ext.label, "x");
    Proto.set(*m, Api.ext.inner, Api.ext.Inner.{});
    value, found := Proto.get(*m, Api.ext.inner);

    expect(found);
    Proto.free(*value);
    Proto.free(*m);
    Proto.free(*m);
} @test

// get tolerates a malformed _unknown instead of reading past it.
malformed_unknown :: () {
    m: Base;

    defer Proto.free(*m);

    bad := u8.[0xa0, 0x06, 0x01, 0xa2, 0x06, 0x7f];

    array_add(*m._unknown, ..bad);

    expect_equal(Proto.get(*m, Api.ext.count), 1);
    expect(!Proto.has(*m, Api.ext.label));
    Proto.clear(*m, Api.ext.count);
    expect_equal(m._unknown.count, 3);
} @test

// A string extension is UTF-8 checked when the schema verifies strings, like a plain field.
utf8_extensions :: () {
    // Editions verify by default.
    m, error := from_hex(Edition.ed.Base, "a206 02 c328");

    defer Proto.free(*error);
    expect(error.kind == .BAD_UTF8);
    expect(error.offset == 3);
    expect(error.path == "[ed.label]");
    Proto.free(*m);

    ok, e2 := from_hex(Edition.ed.Base, "a20603416263");

    defer Proto.free(*ok);
    expect(e2.kind == .NONE);
    expect(Proto.get(*ok, Edition.ed.label) == "Abc");

    // NONE and proto2 do not verify.
    raw, e3 := from_hex(Edition.ed.Base, "aa06 02 c328");

    defer Proto.free(*raw);
    expect(e3.kind == .NONE);

    open, e4 := from_hex(Open.Base, "a206 02 c328");

    defer Proto.free(*open);
    expect(e4.kind == .NONE);

    // skip_utf8 turns the check off.
    lax, e5 := from_hex(Edition.ed.Base, "a206 02 c328", .{ skip_utf8 = true });

    defer Proto.free(*lax);
    expect(e5.kind == .NONE);

    // A wrong wire type is not the extension, so nothing is checked.
    other, e6 := from_hex(Edition.ed.Base, "a0060 1");

    defer Proto.free(*other);
    expect(e6.kind == .NONE);

    // Repeated occurrences and nested messages are checked too.
    nested, e7 := from_hex(Edition.ed.Base, "b206 04 0a02c328");

    defer Proto.free(*e7);
    defer Proto.free(*nested);
    expect(e7.kind == .BAD_UTF8);
    expect(e7.path == "[ed.text].s");

    grouped, e8 := from_hex(Edition.ed.Base, "bb06 0a02c328 bc06");

    defer Proto.free(*e8);
    defer Proto.free(*grouped);
    expect(e8.kind == .BAD_UTF8);

    fine, e9 := from_hex(Edition.ed.Base, "b206 04 0a024142");

    defer Proto.free(*fine);
    expect(e9.kind == .NONE);
    text, found := Proto.get(*fine, Edition.ed.text);

    defer Proto.free(*text);
    expect(found && text.s.value == "AB");

    // Proto2 messages carry unverified strings, so their extensions need no check.
    plain, e10 := from_hex(Open.Base, "aa06 04 0a02c328");

    defer Proto.free(*plain);
    expect(e10.kind == .NONE);
} @test
