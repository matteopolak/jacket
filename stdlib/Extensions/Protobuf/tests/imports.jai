#scope_file
#import "Basic";
#import "String";
#import "File";
#import "Extensions/Testing";
Proto :: #import "Extensions/Protobuf";
Schema :: #import "Extensions/Protobuf/Schema";
Tree :: #import "Extensions/Protobuf/Schema"(DIR = #run Proto.beside("fixtures/tree"));
Again :: #import "Extensions/Protobuf/Schema"(DIR = #run Proto.beside("fixtures/tree"));
Single :: #import "Extensions/Protobuf/Schema"(PATH = #run Proto.beside("fixtures/tree/public.proto"));
Search :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; import 'external.proto'; message Local { external.Remote remote=1; }", IMPORT_PATHS = #run tprint("missing-root;%", Proto.beside("fixtures/import-roots")));
Known :: #import "Extensions/Protobuf/Schema"(TEXT = #string END
syntax='proto3';
import 'google/protobuf/any.proto';
import 'google/protobuf/duration.proto';
import 'google/protobuf/empty.proto';
import 'google/protobuf/field_mask.proto';
import 'google/protobuf/struct.proto';
import 'google/protobuf/timestamp.proto';
import 'google/protobuf/wrappers.proto';
message Event {
  google.protobuf.Timestamp time=1;
  google.protobuf.Struct data=2;
}
END);

#scope_export

imports_and_tree :: () {
    #assert Tree.demo.model.Item == Again.demo.model.Item;
    global: Tree.Global;

    expect(global.item == null);
    value: Tree.demo.api.Request;
    value.envelope = New(Tree.demo.model.Envelope);
    value.envelope.item = New(Tree.demo.model.Item);
    value.envelope.item.id = 17;
    defer Proto.free(*value);
    bytes, ee := Proto.encode(value);

    require(ee.kind == .NONE);
    defer free(bytes.data);
    copy, error := Proto.decode(Tree.demo.api.Request, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*copy);
    expect(Proto.equal(value, copy));
    public_user: Single.demo.api.PublicUser;
    public_user.item = New(Single.demo.model.Item);
    defer Proto.free(*public_user);
    expect(public_user.item.id == 0);
    search: Search.Local;

    expect(search.remote == null);
} @test

import_graph_errors :: () {
    cases := string.["a.proto", "missing.proto"];
    for cases {
        path := Proto.beside(tprint("fixtures/errors/%", it));

        defer free(path);
        text, ok := read_entire_file(path);

        require(ok);
        defer free(text);
        output, error := Schema.protobuf_generate(text, .{ path = path });

        defer free(error.message);
        expect(output.count == 0 && error.message.count > 0);

        if it_index == 0 {
            expect(contains(error.message, "import cycle"));
            expect(ends_with(error.path, "b.proto"));
            expect(error.line == 2);
        } else {
            expect(contains(error.message, "does-not-exist.proto"));
            expect(ends_with(error.path, "missing.proto"));
            expect(error.line == 3 && error.column == 1);
        }
    }
} @test

private_import_is_not_reexported :: () {
    path := Proto.beside("fixtures/errors/root.proto");

    defer free(path);
    _, error := Schema.protobuf_generate("syntax='proto3'; import 'private.proto'; message User { demo.model.Item hidden=1; }", .{ path = path });

    defer free(error.message);
    expect(contains(error.message, "unknown type"));
} @test

well_known_round_trip :: () {
    value: Known.Event;
    value.time = New(Known.google.protobuf.Timestamp);
    value.time.seconds = 123;
    value.time.nanos = 4;
    defer Proto.free(*value);
    bytes, ee := Proto.encode(value);

    require(ee.kind == .NONE);
    defer free(bytes.data);
    decoded, error := Proto.decode(Known.Event, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*decoded);
    expect(Proto.equal(value, decoded));
    any := Known.google.protobuf.Any_.{ type_url = "demo", value = u8.[1, 2] };
    encoded, ae := Proto.encode(any);

    require(ae.kind == .NONE);
    defer free(encoded.data);
    expect(encoded.count == 10);
    mask: Known.google.protobuf.FieldMask;

    expect(mask.paths.count == 0);
    duration: Known.google.protobuf.Duration;

    expect(duration.seconds == 0 && duration.nanos == 0);
    empty: Known.google.protobuf.Empty;

    expect(Proto.size(empty) == 0);
    wrapper := Known.google.protobuf.Int32Value.{ value = -1 };

    expect(Proto.size(wrapper) == 11);
    dynamic: Known.google.protobuf.Value;
    dynamic.kind.which = .STRING_VALUE;
    dynamic.kind.string_value = "hello";
    dbytes, de := Proto.encode(dynamic);

    require(de.kind == .NONE);
    defer free(dbytes.data);
    back, be := Proto.decode(Known.google.protobuf.Value, dbytes);

    require(be.kind == .NONE);
    defer Proto.free(*back);
    expect(Proto.equal(dynamic, back));
} @test

file_on_import_path_beats_built_in :: () {
    root := Proto.beside("fixtures/disk-wins");

    defer free(root);
    output, error := Schema.protobuf_generate("syntax='proto3'; import 'google/protobuf/empty.proto'; message M { google.protobuf.Empty e=1; }", .{ import_paths = root });

    defer free(error.message);
    require(error.message.count == 0);
    defer free(output);
    expect(contains(output, "OnlyOnDisk :: struct"));
} @test
