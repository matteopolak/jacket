#scope_file
#import "Basic";
#import "String";
#import "Extensions/Testing";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
syntax = "proto3";
package demo;
import 'google/protobuf/any.proto';
import 'google/protobuf/duration.proto';
import 'google/protobuf/empty.proto';
import 'google/protobuf/field_mask.proto';
import 'google/protobuf/struct.proto';
import 'google/protobuf/timestamp.proto';
import 'google/protobuf/wrappers.proto';
enum Color { COLOR_UNSPECIFIED = 0; RED = 1; GREEN = 2; }
message Inner { int32 a = 1; repeated string tags = 2; }
message All {
  int32 i32 = 1;
  int64 i64 = 2;
  uint32 u32 = 3;
  uint64 u64 = 4;
  sint64 s64 = 5;
  fixed64 f64 = 6;
  float f = 7;
  double d = 8;
  bool flag = 9;
  string text_value = 10;
  bytes data = 11;
  Color color = 12;
  Inner inner = 13;
  repeated int32 nums = 14;
  repeated Inner inners = 15;
  repeated Color colors = 16;
  map<string, int32> by_name = 17;
  map<int64, Inner> by_id = 18;
  map<bool, Color> by_flag = 19;
  oneof choice { string word = 20; Inner nested = 21; Color shade = 22; }
  optional int32 maybe = 23;
  string type = 24;
  int32 renamed = 25 [json_name = "Other"];
  int32 after = 26;
}
message Known {
  google.protobuf.Timestamp time = 1;
  google.protobuf.Duration span = 2;
  google.protobuf.FieldMask mask = 3;
  google.protobuf.Struct data = 4;
  google.protobuf.Value value = 5;
  google.protobuf.ListValue list = 6;
  google.protobuf.Int64Value wide = 7;
  google.protobuf.StringValue name = 8;
  google.protobuf.BoolValue on = 9;
  google.protobuf.Empty nothing = 10;
  google.protobuf.Any any = 11;
  google.protobuf.NullValue null_value = 12;
  repeated google.protobuf.Value values = 13;
  repeated google.protobuf.Any anys = 14;
  google.protobuf.DoubleValue ratio = 15;
  google.protobuf.BytesValue blob = 16;
}
END);
Closed :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto2'; enum E { A = 1; B = 2; } message M { optional E e = 1; repeated E es = 2; optional int32 x = 3; }");

// Decodes text and encodes it again; the output must be `want`.
round_trip :: ($T: Type, text: string, want: string, options := Proto.Json_Options.{}, loc := #caller_location) {
    value, error := Proto.from_json(T, text, options);

    defer Proto.free(*value);
    defer Proto.free(*error);

    if !expect(error.kind == .NONE, tprint("% at %: %", error.kind, error.path, error.message), loc = loc) return;

    out, oe := Proto.to_json(value, options);

    defer free(out);
    defer Proto.free(*oe);
    expect(oe.kind == .NONE, loc = loc);
    expect_equal(out, want, loc = loc);
}

// Decoding text must fail with kind, and at path when one is given.
rejects :: ($T: Type, text: string, kind: Proto.Error_Kind, path := "", options := Proto.Json_Options.{}, loc := #caller_location) {
    value, error := Proto.from_json(T, text, options);

    defer Proto.free(*value);
    defer Proto.free(*error);

    expect(error.kind == kind, tprint("got % (%) for %", error.kind, error.message, text), loc = loc);
    if path.count expect_equal(error.path, path, loc = loc);
}

#scope_export

json_scalars :: () {
    round_trip(Api.demo.All, "{}", "{}");
    round_trip(Api.demo.All, "{\"i32\": -5, \"i64\": \"-9223372036854775808\", \"u32\": 4294967295, \"u64\": \"18446744073709551615\", \"s64\": 12, \"f64\": \"7\"}", "{\"i32\":-5,\"i64\":\"-9223372036854775808\",\"u32\":4294967295,\"u64\":\"18446744073709551615\",\"s64\":\"12\",\"f64\":\"7\"}");
    // Integers may be quoted or written as any JSON number with an integer value.
    round_trip(Api.demo.All, "{\"i32\": \"12\", \"i64\": 1e3, \"u32\": 100.000, \"u64\": \"2E2\"}", "{\"i32\":12,\"i64\":\"1000\",\"u32\":100,\"u64\":\"200\"}");
    round_trip(Api.demo.All, "{\"f\": \"NaN\", \"d\": \"-Infinity\"}", "{\"f\":\"NaN\",\"d\":\"-Infinity\"}");
    round_trip(Api.demo.All, "{\"f\": 0.1, \"d\": \"1.5e300\"}", "{\"f\":0.1,\"d\":1.5e300}");
    round_trip(Api.demo.All, "{\"flag\": true, \"textValue\": \"h\\u00e9\", \"data\": \"AQID\"}", "{\"flag\":true,\"textValue\":\"hé\",\"data\":\"AQID\"}");
    // Bytes accept URL-safe and unpadded base64.
    round_trip(Api.demo.All, "{\"data\": \"-_8\"}", "{\"data\":\"+/8=\"}");
    round_trip(Api.demo.All, "{\"data\": \"YQ\"}", "{\"data\":\"YQ==\"}");
    // null is absence.
    round_trip(Api.demo.All, "{\"i32\": null, \"inner\": null, \"nums\": null, \"byName\": null, \"color\": null}", "{}");

    rejects(Api.demo.All, "{\"i32\": 2147483648}", .BAD_VALUE, "i32");
    rejects(Api.demo.All, "{\"u32\": -1}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"i64\": \"9223372036854775808\"}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"i32\": 1.5}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"i32\": \" 1\"}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"f\": \"nan\"}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"f\": \"1 \"}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"data\": \"Y===\"}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"data\": \"Y!==\"}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"flag\": \"true\"}", .BAD_JSON);
    rejects(Api.demo.All, "{\"textValue\": 1}", .BAD_JSON);
    rejects(Api.demo.All, "{\"i32\": 1", .BAD_JSON);
    rejects(Api.demo.All, "{} x", .BAD_JSON);
    rejects(Api.demo.All, "[]", .BAD_JSON);
} @test

json_names :: () {
    // Both the JSON name and the .proto name are accepted; output uses the JSON name.
    round_trip(Api.demo.All, "{\"text_value\": \"a\", \"Other\": 1, \"type\": \"t\"}", "{\"textValue\":\"a\",\"type\":\"t\",\"Other\":1}");
    round_trip(Api.demo.All, "{\"renamed\": 2}", "{\"Other\":2}");
    round_trip(Api.demo.All, "{\"textValue\": \"a\", \"renamed\": 2}", "{\"text_value\":\"a\",\"renamed\":2}", .{ proto_names = true });

    rejects(Api.demo.All, "{\"text_value\": \"a\", \"textValue\": \"b\"}", .DUPLICATE_FIELD);
    rejects(Api.demo.All, "{\"i32\": 1, \"i32\": 2}", .DUPLICATE_FIELD);
    rejects(Api.demo.All, "{\"nope\": 1}", .UNKNOWN_FIELD);
    rejects(Api.demo.All, "{\"TextValue\": \"a\"}", .UNKNOWN_FIELD);
    round_trip(Api.demo.All, "{\"nope\": {\"deep\": [1, 2]}, \"i32\": 3}", "{\"i32\":3}", .{ ignore_unknown_fields = true });
} @test

json_enums :: () {
    round_trip(Api.demo.All, "{\"color\": \"GREEN\", \"colors\": [\"RED\", 2, 7]}", "{\"color\":\"GREEN\",\"colors\":[\"RED\",\"GREEN\",7]}");
    round_trip(Api.demo.All, "{\"color\": 2}", "{\"color\":2}", .{ enum_numbers = true });
    rejects(Api.demo.All, "{\"color\": \"BLUE\"}", .BAD_VALUE, "color");
    rejects(Api.demo.All, "{\"color\": true}", .BAD_JSON);
    // Ignoring unknown fields also drops unknown enum names, wherever they are.
    round_trip(Api.demo.All, "{\"color\": \"BLUE\", \"colors\": [\"BLUE\", \"RED\"], \"byFlag\": {\"true\": \"BLUE\", \"false\": \"RED\"}, \"shade\": \"BLUE\"}", "{\"colors\":[\"RED\"],\"byFlag\":{\"false\":\"RED\"}}", .{ ignore_unknown_fields = true });

    // A closed enum rejects unknown numbers too.
    round_trip(Closed.M, "{\"e\": \"B\", \"es\": [\"A\", 2]}", "{\"e\":\"B\",\"es\":[\"A\",\"B\"]}");
    rejects(Closed.M, "{\"e\": 5}", .BAD_VALUE);
    round_trip(Closed.M, "{\"e\": 5, \"es\": [5, 1], \"x\": 0}", "{\"es\":[\"A\"],\"x\":0}", .{ ignore_unknown_fields = true });
} @test

json_messages :: () {
    round_trip(Api.demo.All, "{\"inner\": {\"a\": 1, \"tags\": [\"x\", \"y\"]}, \"inners\": [{}, {\"a\": 2}]}", "{\"inner\":{\"a\":1,\"tags\":[\"x\",\"y\"]},\"inners\":[{},{\"a\":2}]}");
    round_trip(Api.demo.All, "{\"inner\": {}}", "{\"inner\":{}}");
    rejects(Api.demo.All, "{\"inners\": [{}, {\"a\": \"x\"}]}", .BAD_VALUE, "inners[1].a");
    rejects(Api.demo.All, "{\"inners\": [null]}", .BAD_JSON);
    rejects(Api.demo.All, "{\"nums\": [1, null]}", .BAD_JSON);
    rejects(Api.demo.All, "{\"inner\": [1]}", .BAD_JSON);

    deep: String_Builder;

    append(*deep, "{\"value\":");
    for 1..200 append(*deep, "[");

    rejects(Api.demo.Known, builder_to_string(*deep,, temp), .TOO_DEEP);
} @test

json_maps :: () {
    round_trip(Api.demo.All, "{\"byName\": {\"a\": 1, \"b\": \"2\"}, \"byId\": {\"-7\": {\"a\": 1}, \"8\": {}}, \"byFlag\": {\"true\": \"RED\"}}", "{\"byName\":{\"a\":1,\"b\":2},\"byId\":{\"-7\":{\"a\":1},\"8\":{}},\"byFlag\":{\"true\":\"RED\"}}");
    // A repeated key replaces the earlier value.
    round_trip(Api.demo.All, "{\"byName\": {\"a\": 1, \"a\": 3}}", "{\"byName\":{\"a\":3}}");
    rejects(Api.demo.All, "{\"byId\": {\"x\": {}}}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"byId\": {\"1.5\": {}}}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"byFlag\": {\"True\": \"RED\"}}", .BAD_VALUE);
    rejects(Api.demo.All, "{\"byName\": {\"a\": null}}", .BAD_JSON);
    rejects(Api.demo.All, "{\"byName\": [1]}", .BAD_JSON);
} @test

json_oneofs_and_presence :: () {
    round_trip(Api.demo.All, "{\"word\": \"hi\"}", "{\"word\":\"hi\"}");
    round_trip(Api.demo.All, "{\"nested\": {\"a\": 1}, \"after\": 2}", "{\"nested\":{\"a\":1},\"after\":2}");
    // A oneof member is printed even when it holds its default.
    round_trip(Api.demo.All, "{\"shade\": \"COLOR_UNSPECIFIED\"}", "{\"shade\":\"COLOR_UNSPECIFIED\"}");
    round_trip(Api.demo.All, "{\"word\": null, \"shade\": 1}", "{\"shade\":\"RED\"}");
    rejects(Api.demo.All, "{\"word\": \"a\", \"shade\": 1}", .DUPLICATE_FIELD);
    rejects(Api.demo.All, "{\"word\": \"a\", \"word\": \"b\"}", .DUPLICATE_FIELD);

    // Explicit presence prints a set default; implicit presence only with emit_defaults.
    round_trip(Api.demo.All, "{\"maybe\": 0, \"i32\": 0}", "{\"maybe\":0}");

    value: Api.demo.All;
    text, error := Proto.to_json(value, .{ emit_defaults = true });

    defer free(text);
    expect(error.kind == .NONE);
    expect(begins_with(text, "{\"i32\":0,\"i64\":\"0\",\"u32\":0,\"u64\":\"0\",\"s64\":\"0\",\"f64\":\"0\",\"f\":0,\"d\":0,\"flag\":false,\"textValue\":\"\",\"data\":\"\",\"color\":\"COLOR_UNSPECIFIED\",\"nums\":[],"), text);
    expect(!contains(text, "maybe") && !contains(text, "\"inner\"") && !contains(text, "word"), text);
} @test

json_pretty_and_merge :: () {
    value: Api.demo.All;
    value.i32 = 1;
    array_add(*value.nums, 2, 3);

    defer Proto.free(*value);
    text := Proto.to_json(value, .{ pretty = true });

    defer free(text);
    expect_equal(text, "{\n  \"i32\": 1,\n  \"nums\": [\n    2,\n    3\n  ]\n}");

    error := Proto.merge_json(*value, "{\"nums\": [4], \"u32\": 5}");

    expect(error.kind == .NONE);
    expect(value.i32 == 1 && value.u32_ == 5 && value.nums.count == 3);

    error = Proto.merge_json(*value, "{\"i32\": 9, \"u32\": true}");
    defer Proto.free(*error);

    expect(error.kind == .BAD_JSON && error.path == "u32");
    expect(value.i32 == 9);
} @test

json_well_known :: () {
    round_trip(Api.demo.Known, "{\"time\": \"1970-01-01T00:00:00Z\", \"span\": \"0s\"}", "{\"time\":\"1970-01-01T00:00:00Z\",\"span\":\"0s\"}");
    round_trip(Api.demo.Known, "{\"time\": \"2024-02-29T23:59:59.5+01:30\"}", "{\"time\":\"2024-02-29T22:29:59.500Z\"}");
    round_trip(Api.demo.Known, "{\"time\": \"0001-01-01T00:00:00.000001Z\"}", "{\"time\":\"0001-01-01T00:00:00.000001Z\"}");
    round_trip(Api.demo.Known, "{\"span\": \"-1.000000001s\"}", "{\"span\":\"-1.000000001s\"}");
    round_trip(Api.demo.Known, "{\"span\": \"-0.250s\"}", "{\"span\":\"-0.250s\"}");
    round_trip(Api.demo.Known, "{\"mask\": \"fooBar,baz.quxQuux\"}", "{\"mask\":\"fooBar,baz.quxQuux\"}");
    round_trip(Api.demo.Known, "{\"mask\": \"\"}", "{\"mask\":\"\"}");
    round_trip(Api.demo.Known, "{\"wide\": \"5\", \"name\": \"\", \"on\": false, \"ratio\": \"NaN\", \"blob\": \"AA==\"}", "{\"wide\":\"5\",\"name\":\"\",\"on\":false,\"ratio\":\"NaN\",\"blob\":\"AA==\"}");
    round_trip(Api.demo.Known, "{\"nothing\": {}}", "{\"nothing\":{}}");
    round_trip(Api.demo.Known, "{\"nullValue\": null}", "{}");
    round_trip(Api.demo.Known, "{\"nullValue\": \"NULL_VALUE\"}", "{}");

    rejects(Api.demo.Known, "{\"time\": \"1970-01-01T00:00:00z\"}", .BAD_VALUE, "time");
    rejects(Api.demo.Known, "{\"time\": \"1970-02-30T00:00:00Z\"}", .BAD_VALUE);
    rejects(Api.demo.Known, "{\"time\": \"10000-01-01T00:00:00Z\"}", .BAD_VALUE);
    rejects(Api.demo.Known, "{\"time\": \"0001-01-01T00:00:00+00:01\"}", .BAD_VALUE);
    rejects(Api.demo.Known, "{\"time\": \"1970-01-01T00:00:00.1234567890Z\"}", .BAD_VALUE);
    rejects(Api.demo.Known, "{\"span\": \"1\"}", .BAD_VALUE);
    rejects(Api.demo.Known, "{\"span\": \"315576000001s\"}", .BAD_VALUE);
    rejects(Api.demo.Known, "{\"mask\": \"foo_bar\"}", .BAD_VALUE);
    rejects(Api.demo.Known, "{\"time\": 0}", .BAD_JSON);

    // Times and masks JSON can't express fail to print.
    value: Api.demo.Known;
    value.time = New(type_of(value.time.*));
    value.time.seconds = 253402300800;

    defer Proto.free(*value);
    text, error := Proto.to_json(value);

    defer Proto.free(*error);
    expect(error.kind == .BAD_VALUE && error.path == "time" && text == "", error.message);

    mask: Api.demo.Known;
    mask.mask = New(type_of(mask.mask.*));
    array_add(*mask.mask.paths, copy_string("Upper"));

    defer Proto.free(*mask);
    _, me := Proto.to_json(mask);

    defer Proto.free(*me);
    expect(me.kind == .BAD_VALUE);
} @test

json_struct_values :: () {
    round_trip(Api.demo.Known, "{\"data\": {\"a\": [1, \"x\", null, true, {\"b\": {}}], \"c\": null}}", "{\"data\":{\"a\":[1,\"x\",null,true,{\"b\":{}}],\"c\":null}}");
    round_trip(Api.demo.Known, "{\"value\": null}", "{\"value\":null}");
    round_trip(Api.demo.Known, "{\"value\": 1.5, \"list\": [[], {}], \"values\": [null, 2]}", "{\"value\":1.5,\"list\":[[],{}],\"values\":[null,2]}");
    round_trip(Api.demo.Known, "{\"data\": {}}", "{\"data\":{}}");
    rejects(Api.demo.Known, "{\"data\": [1]}", .BAD_JSON);
    rejects(Api.demo.Known, "{\"list\": {}}", .BAD_JSON);

    // A Value with no kind, or holding NaN, has no JSON.
    value: Api.demo.Known;
    value.value = New(type_of(value.value.*));

    defer Proto.free(*value);
    _, error := Proto.to_json(value);

    defer Proto.free(*error);
    expect(error.kind == .BAD_VALUE && error.path == "value");
} @test

json_any :: () {
    options := Proto.Json_Options.{ registry = Api.proto_registry };

    round_trip(Api.demo.Known, "{\"any\": {\"@type\": \"type.googleapis.com/demo.Inner\", \"a\": 3, \"tags\": [\"t\"]}}", "{\"any\":{\"@type\":\"type.googleapis.com/demo.Inner\",\"a\":3,\"tags\":[\"t\"]}}", options);
    // "@type" may come anywhere in the object.
    round_trip(Api.demo.Known, "{\"any\": {\"a\": 3, \"@type\": \"x/demo.Inner\"}}", "{\"any\":{\"@type\":\"x/demo.Inner\",\"a\":3}}", options);
    round_trip(Api.demo.Known, "{\"any\": {\"@type\": \"type.googleapis.com/google.protobuf.Duration\", \"value\": \"1.5s\"}}", "{\"any\":{\"@type\":\"type.googleapis.com/google.protobuf.Duration\",\"value\":\"1.500s\"}}", options);
    round_trip(Api.demo.Known, "{\"anys\": [{\"@type\": \"t/google.protobuf.Any\", \"value\": {\"@type\": \"t/google.protobuf.Struct\", \"value\": {\"k\": 1}}}, {}]}", "{\"anys\":[{\"@type\":\"t/google.protobuf.Any\",\"value\":{\"@type\":\"t/google.protobuf.Struct\",\"value\":{\"k\":1}}},{}]}", options);
    round_trip(Api.demo.Known, "{\"any\": {\"@type\": \"t/demo.Inner\", \"nope\": 1}}", "{\"any\":{\"@type\":\"t/demo.Inner\"}}", .{ registry = Api.proto_registry, ignore_unknown_fields = true });

    rejects(Api.demo.Known, "{\"any\": {\"@type\": \"t/demo.Missing\"}}", .UNKNOWN_TYPE, options = options);
    rejects(Api.demo.Known, "{\"any\": {\"@type\": \"t/demo.Inner\"}}", .UNKNOWN_TYPE);
    rejects(Api.demo.Known, "{\"any\": {\"a\": 1}}", .BAD_VALUE, options = options);
    rejects(Api.demo.Known, "{\"any\": {\"@type\": \"t/demo.Inner\", \"a\": \"x\"}}", .BAD_VALUE, options = options);
    rejects(Api.demo.Known, "{\"any\": {\"@type\": \"t/demo.Inner\", \"@type\": \"t/demo.Inner\"}}", .DUPLICATE_FIELD, options = options);
    rejects(Api.demo.Known, "{\"any\": {\"@type\": \"t/google.protobuf.Duration\", \"value\": \"1s\", \"value\": \"2s\"}}", .DUPLICATE_FIELD, options = options);

    // Printing an Any needs its type in the registry, and bytes that decode as that type.
    value: Api.demo.Known;
    value.any = New(type_of(value.any.*));
    value.any.type_url = copy_string("t/demo.Inner");
    value.any.value = NewArray(1, u8);
    value.any.value[0] = 0xff;

    defer Proto.free(*value);
    _, unknown := Proto.to_json(value);

    defer Proto.free(*unknown);
    expect(unknown.kind == .UNKNOWN_TYPE);
    _, bad := Proto.to_json(value, options);

    defer Proto.free(*bad);
    expect(bad.kind == .BAD_VALUE && bad.path == "any");
} @test
