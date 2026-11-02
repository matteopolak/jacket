#scope_file
#import "Basic";
#import "Extensions/Testing";
Proto :: #import "Extensions/Protobuf";
Fuzz :: #import "Extensions/Fuzz";
Schema :: #import "Extensions/Protobuf/Schema";
E :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
edition = "2023";
option features.enum_type = CLOSED;
option features.utf8_validation = NONE;
enum Choice { FIRST=5; SECOND=9; }
message M {
  int32 required_value=1 [features.field_presence=LEGACY_REQUIRED];
  Choice choice=2 [default=SECOND];
  repeated Choice choices=3;
  map<int32,Choice> mapped=4;
  string text=5 [default='hello'];
  bytes data=6 [default='\001\377'];
  M child=7 [features.message_encoding=DELIMITED];
  oneof selection { Choice chosen=8; int32 number=9; }
}
END);
P2 :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
syntax="proto2";
message M {
  required int32 id=1;
  optional group Child=2 { required int32 a=3; optional int32 b=4 [default=-12]; }
  repeated group Items=5 { optional string text=1; }
  extensions 100 to max;
}
extend M { optional string extra=100; }
END);
P3 :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; enum E { ZERO=0; } message M { E e=1; string s=2; }");
Scopes :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
edition="2024";
option features.field_presence=IMPLICIT;
option features.json_format=ALLOW;
message M {
  option features.field_presence=EXPLICIT;
  option features.utf8_validation=NONE;
  repeated int32 expanded=1 [features.repeated_field_encoding=EXPANDED];
  int32 implicit=2 [features.field_presence=IMPLICIT];
  string checked=3 [features.utf8_validation=VERIFY];
  oneof via { option features.utf8_validation=NONE; string raw=4; }
  enum E { option features.enum_type=CLOSED; A=2; B=4; }
  E e=5;
}
END);

#scope_export

edition_defaults_and_required :: () {
    m: E.M;

    expect(!m.choice.has && m.choice.value == .SECOND);
    expect(!m.text.has && m.text.value == "hello");
    expect(!m.data.has && m.data.value.count == 2 && m.data.value[1] == 255);
    _, ee := Proto.encode(m);

    defer Proto.free(*ee);
    expect(ee.kind == .MISSING_REQUIRED && ee.path == "required_value");
    bytes := u8.[8, 0, 40, 255]; // wrong wire for text is unknown
    value, error := Proto.decode(E.M, bytes);

    defer Proto.free(*value);
    defer Proto.free(*error);
    expect(error.kind == .TRUNCATED);
    _, de := Proto.decode(E.M, .{});

    defer Proto.free(*de);
    expect(de.kind == .MISSING_REQUIRED && de.path == "required_value");
} @test

closed_enums_everywhere :: () {
    bytes := u8.[8, 1, 16, 7, 26, 3, 5, 7, 9, 34, 4, 8, 1, 16, 7, 64, 7];
    m, error := Proto.decode(E.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);
    expect(!m.choice.has && m.choice.value == .SECOND);
    expect(m.choices.count == 2 && m.choices[0] == .FIRST && m.choices[1] == .SECOND);
    expect(m.mapped.count == 0 && m.selection.which == .NONE);
    unknown := u8.[16, 7, 24, 7, 34, 4, 8, 1, 16, 7, 64, 7];

    expect(Proto.bytes_equal(m._unknown, unknown));
    encoded, ee := Proto.encode(m);

    require(ee.kind == .NONE);
    defer free(encoded.data);
    back, be := Proto.decode(E.M, encoded);

    require(be.kind == .NONE);
    defer Proto.free(*back);
    expect(Proto.equal(m, back));
    discarded, disc := Proto.decode(E.M, bytes, .{ discard_unknown = true });

    require(disc.kind == .NONE);
    defer Proto.free(*discarded);
    expect(discarded._unknown.count == 0);
} @test

group_and_required_merge :: () {
    // Two child occurrences jointly supply required a and optional b.
    bytes := u8.[8, 0, 19, 32, 2, 20, 19, 24, 3, 20, 43, 10, 1, 255, 44, 160, 6, 7];
    m, error := Proto.decode(P2.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);
    expect(m.child && m.child.a.has && m.child.a.value == 3 && m.child.b.value == 2);
    expect(m.items.count == 1 && m.items[0].text.value[0] == 255);
    encoded, ee := Proto.encode(m);

    require(ee.kind == .NONE);
    defer free(encoded.data);
    back, be := Proto.decode(P2.M, encoded);

    require(be.kind == .NONE);
    defer Proto.free(*back);
    expect(Proto.equal(m, back));
    bad := u8.[8, 0, 19, 20];
    _, missing := Proto.decode(P2.M, bad);

    defer Proto.free(*missing);
    expect(missing.kind == .MISSING_REQUIRED && missing.path == "child.a");
    truncated := u8.[8, 0, 19];
    _, te := Proto.decode(P2.M, truncated);

    defer Proto.free(*te);
    expect(te.kind == .TRUNCATED);
    wrong := u8.[8, 0, 19, 28];
    _, we := Proto.decode(P2.M, wrong);

    defer Proto.free(*we);
    expect(we.kind == .BAD_GROUP);
} @test

feature_scopes_and_utf8 :: () {
    m: Scopes.M;

    array_add(*m.expanded, 1);
    defer Proto.free(*m);
    bytes, error := Proto.encode(m);

    require(error.kind == .NONE);
    defer free(bytes.data);
    expected := u8.[8, 1];

    expect(Proto.bytes_equal(bytes, expected));
    invalid := u8.[26, 1, 255];
    _, bad := Proto.decode(Scopes.M, invalid);

    defer Proto.free(*bad);
    expect(bad.kind == .BAD_UTF8);
    raw := u8.[34, 1, 255];
    good, ge := Proto.decode(Scopes.M, raw);

    require(ge.kind == .NONE);
    defer Proto.free(*good);
    expect(good.via.which == .RAW);
    open := u8.[8, 123];
    value, ve := Proto.decode(P3.M, open);

    require(ve.kind == .NONE);
    defer Proto.free(*value);
    expect(cast(s32) value.e == 123);
} @test

default_payload_replacement :: () {
    bytes := u8.[8, 1, 42, 1, 120, 50, 1, 121];
    m, error := Proto.decode(E.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);
    expect(m.text.has && m.text.value == "x" && m.data.has && m.data.value[0] == 121);
} @test

schema_feature_diagnostics :: () {
    inputs := string.[
        "edition='2023'; option features.unknown=VALUE;",
        "edition='2024'; option features.enum_type=INVALID;",
        "edition='2023'; message M { repeated int32 n=1 [default=3]; }",
        "edition='2023'; message M { int32 n=1 [features.field_presence=IMPLICIT, default=3]; }"
    ];
    for inputs {
        output, error := Schema.protobuf_generate(it);
        expect(!output.count && error.message.count > 0);
        free(error.message);
    }

    output, error := Schema.protobuf_generate("edition='2024'; option features.(pb.cpp).legacy_closed_enum=true; message M { extensions 1 to max; } extend M { int32 x=1; }");

    require(!error.message.count);
    free(output);
} @test

nested_unknown_enum_is_not_dropped :: () {
    A :: #import "Extensions/Protobuf/Schema"(TEXT = "edition='2023'; enum E { option features.enum_type=CLOSED; A=3; } message N { E e=1; } message M { repeated N n=1; map<int32,E> m=2; }", MAP = .ARRAY);
    bytes := u8.[10, 2, 8, 7, 18, 2, 8, 1];
    m, error := Proto.decode(A.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);
    expect(m.n.count == 1 && !m.n[0].e.has && m.n[0]._unknown.count == 2);
    expect(m.m.count == 1 && m.m[0].value == .A);
} @test

special_and_concatenated_defaults :: () {
    A :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; message M { optional double a=1 [default=inf]; optional float b=2 [default=-inf]; optional double c=3 [default=nan]; optional string s=4 [default='ab' 'cd']; }");
    m: A.M;
    expect(m.a.value > 1e300 && m.b.value < -1e30 && m.c.value != m.c.value);
    expect(m.s.value == "abcd" && !m.s.has);
} @test

// This test also runs under Testing's allocator, which rejects leaks and bad frees.
damaged_editions_cleanup :: () {
    seed := u8.[8, 1, 26, 2, 5, 9, 34, 4, 8, 1, 16, 5, 42, 1, 120, 50, 1, 121, 59, 8, 1, 60, 64, 5];
    rng: Fuzz.Rng;

    Fuzz.rng_seed(*rng, 2024);

    for 0..999 {
        data := seed;
        index := Fuzz.below(*rng, data.count);
        data[index] ^= cast,trunc(u8) Fuzz.next(*rng);
        count := Fuzz.below(*rng, data.count + 1);
        value, error := Proto.decode(E.M, .{ count, data.data }, .{ max_depth = 8, max_size = 4096 });

        if error.kind == .NONE {
            encoded, ee := Proto.encode(value);

            require(ee.kind == .NONE);
            back, be := Proto.decode(E.M, encoded, .{ max_depth = 8, max_size = 4096 });

            expect(be.kind == .NONE && Proto.equal(value, back));
            Proto.free(*back);
            Proto.free(*be);
            free(encoded.data);
        }

        Proto.free(*value);
        Proto.free(*error);
    }
} @test

closed_enum_unknowns_inside_map_messages :: () {
    A :: #import "Extensions/Protobuf/Schema"(TEXT = "edition='2023'; enum E { option features.enum_type=CLOSED; A=0; } message N { E e=1; } message M { map<int32,N> items=1; }", MAP = .ARRAY);
    bytes := u8.[10, 6, 8, 1, 18, 2, 8, 7];
    value, error := Proto.decode(A.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*value);
    require(value.items.count == 1 && value.items[0].value != null);
    expect(!value.items[0].value.e.has && value.items[0].value._unknown.count == 2);
    encoded, ee := Proto.encode(value);

    require(ee.kind == .NONE);
    defer free(encoded.data);
    expect(Proto.bytes_equal(encoded, bytes));
} @test
