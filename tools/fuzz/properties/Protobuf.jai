// A decoder target spanning editions, closed enums, packed values, maps and groups.
Proto_Fuzz :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
edition="2023";
enum E { option features.enum_type=CLOSED; A=0; B=3; }
message M {
  int64 id=1; string text=2; bytes payload=3;
  repeated fixed32 words=4; repeated sint32 numbers=5; repeated E choices=6;
  map<string,E> states=7; M next=8;
  M group=9 [features.message_encoding=DELIMITED];
  oneof selected { string name=10; M child=11; E choice=12; }
}
END);

// Accepted arbitrary messages must survive re-encoding and compare equal.
fuzz_protobuf_decoder :: (r: *Rng) {
    input := gen(r, string);
    bytes: [] u8 = .{ input.count, input.data };
    value, error := Target.decode(Proto_Fuzz.M, bytes, .{ max_depth = 16, max_size = 65536 });

    defer Target.free(*value);
    defer Target.free(*error);
    if error.kind != .NONE return;

    encoded, ee := Target.encode(value);

    assert(ee.kind == .NONE);
    defer free(encoded.data);
    back, be := Target.decode(Proto_Fuzz.M, encoded, .{ max_depth = 16, max_size = 65536 });

    defer Target.free(*back);
    defer Target.free(*be);
    assert(be.kind == .NONE && Target.equal(value, back));
}

// Mutate valid encodings to reach paths that uniform arbitrary bytes rarely exercise.
fuzz_protobuf_damaged :: (r: *Rng) {
    value: Proto_Fuzz.M;
    value.id = .{ has = true, value = gen(r, s64) };
    value.text = .{ has = true, value = "valid UTF-8" };
    for 0..7 array_add(*value.words, gen(r, u32));

    defer array_free(value.words);
    encoded, ee := Target.encode(value);

    assert(ee.kind == .NONE);
    defer free(encoded.data);

    if encoded.count {
        index := cast(s64) (gen(r, u32) % cast(u32) encoded.count);
        encoded[index] ^= gen(r, u8);
        encoded.count = cast(s64) (gen(r, u32) % cast(u32) (encoded.count + 1));
    }

    back, be := Target.decode(Proto_Fuzz.M, encoded, .{ max_depth = 16, max_size = 65536 });

    Target.free(*back);
    Target.free(*be);
}

// Accepted JSON prints, and what it prints reads back to the same message and text.
fuzz_protobuf_json :: (r: *Rng) {
    seeds :: string.[
        "{\"id\":\"-5\",\"text\":\"a\",\"payload\":\"AQ==\",\"words\":[1,2],\"numbers\":[-3],\"choices\":[\"B\",0],\"states\":{\"k\":\"A\"},\"next\":{\"name\":\"n\"},\"group\":{},\"child\":{\"id\":1}}",
        "{\"choice\":3,\"next\":{\"next\":{\"states\":{\"\":\"B\"}}},\"id\":1e3}",
    ];

    input := gen(r, string);

    // Damaged valid documents reach deeper than arbitrary text.
    if gen(r, bool) {
        input = copy_string(seeds[gen(r, u8) % seeds.count],, temp);
        index := cast(s64) (gen(r, u32) % cast(u32) input.count);
        input[index] ^= gen(r, u8);
        input.count = cast(s64) (gen(r, u32) % cast(u32) (input.count + 1));
    }

    options := Target.Json_Options.{ max_depth = 16 };
    value, error := Target.from_json(Proto_Fuzz.M, input, options);

    defer Target.free(*value);
    defer Target.free(*error);
    if error.kind != .NONE return;

    text, te := Target.to_json(value);

    defer free(text);
    defer Target.free(*te);
    assert(te.kind == .NONE);
    back, be := Target.from_json(Proto_Fuzz.M, text, options);

    defer Target.free(*back);
    defer Target.free(*be);
    assert(be.kind == .NONE);
    again, ae := Target.to_json(back);

    defer free(again);
    defer Target.free(*ae);
    assert(again == text);
}

// Decoded binary messages print as JSON that reads back to the same text.
fuzz_protobuf_json_from_binary :: (r: *Rng) {
    input := gen(r, string);
    value, error := Target.decode(Proto_Fuzz.M, .{ input.count, input.data }, .{ max_depth = 16, max_size = 65536 });

    defer Target.free(*value);
    defer Target.free(*error);
    if error.kind != .NONE return;

    text, te := Target.to_json(value);

    defer free(text);
    defer Target.free(*te);
    assert(te.kind == .NONE);
    back, be := Target.from_json(Proto_Fuzz.M, text, .{ max_depth = 40 });

    defer Target.free(*back);
    defer Target.free(*be);
    assert(be.kind == .NONE);
    again, ae := Target.to_json(back);

    defer free(again);
    defer Target.free(*ae);
    assert(again == text);
}
