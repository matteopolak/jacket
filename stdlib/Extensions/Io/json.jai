// JSON from a stream. `Extensions/Json` knows nothing about IO (it builds for WebAssembly too),
// so it takes a plain callback for the pieces of a document (`Json_Source`); this is the glue that
// makes one from a `Reader`.
//
//     user, error := Io.read_json(Io.reader_of(*file), User);
//     defer free_json(*user);

// Reads a document to the end of `reader` and decodes it as a `T`: the same value or error as
// `from_json(T, text)` for the same bytes, however they arrive (a pipe written in small pieces,
// a socket). The pieces are gathered in one buffer that is freed before this returns. A reader
// that fails gives a `.READ` error naming the `Io_Error`; a document over `options.max_size`
// is a `.LIMIT` error and the rest of the stream is left unread. The reader is not closed.
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
