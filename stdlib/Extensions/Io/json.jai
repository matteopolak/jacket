// JSON from a stream: the glue between `Extensions/Json`, which knows nothing about IO, and a
// `Reader`.

// Reads a document to the end of `reader` and decodes it as a `T`: the same value or error as
// `from_json(T, text)` for the same bytes, however they arrive (a pipe written in small pieces,
// a socket). The pieces are gathered in one buffer that is freed before this returns. A reader
// that fails gives a `.READ` error naming the `Io_Error`; a document over `options.max_size`
// is a `.LIMIT` error and the rest of the stream is left unread. The reader is not closed.
//
// ```jai
// Io :: #import "Extensions/Io";
// #import "Extensions/Json";
//
// Point :: struct {
//     x: int;
//     y: int;
// }
//
// Io.io_run(() {
//     near, far := Io.make_pipe();
//     defer Io.close(near);
//     defer Io.close(far);
//
//     Io.write(Io.writer_of(far), "{\"x\": 1, \"y\": 2}");
//     Io.shutdown_write(far);
//
//     point, error := Io.read_json(Io.reader_of(near), Point);
//     assert(error.kind == .NONE && point.x == 1 && point.y == 2);
// });
// ```
read_json :: (reader: Reader, $T: Type, options := Json.Json_Options.{}) -> T, Json.Json_Error {
    source := reader;

    return Json.from_json(T, Json.Json_Source.{ *source, json_fill }, options);
}

#scope_module

json_fill :: (data: *void, buffer: [] u8) -> s64, string {
    reader := cast(*Reader) data;

    count, error := read(reader.*, buffer);

    if error != .NONE return 0, tprint("reading failed: %", error);

    return count, "";
}
