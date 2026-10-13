// The HTTP client against the notes API of crud.jai (start that first, or pass another base URL).
//
//     jaic run examples/http/client.jai
//     jaic run examples/http/client.jai -- http://localhost:3000
//
// The client runs on a fiber, so everything happens inside Io.io_run. Repeated requests to one
// host reuse the same connection.
#import "Basic";
#import "Extensions/Http";
Io :: #import "Extensions/Io";

Note :: struct {
    id: int;
    title: string;
    done: bool;
}

New_Note :: struct {
    title: string;
    done: bool;
}

Run :: struct {
    base: string;
}

demo :: (run: *Run) {
    base := run.base;

    created, error := post_json(Note, tprint("%/notes", base), New_Note.{ "write the docs", false });

    if error.kind != .NONE {
        print("could not create a note: %\n", error_text(error));
        return;
    }

    print("created #%: %\n", created.id, created.title);

    note, _ := get_json(Note, tprint("%/notes/%", base, created.id));

    print("read it back: % (done: %)\n", note.title, note.done);

    // A status that is not 2xx is an error that carries the problem document's title and detail.
    _, missing_error := get_json(Note, tprint("%/notes/%", base, 99999));

    print("asking for a missing note: % (%)\n", missing_error.status, missing_error.detail);

    // `send` for everything else: a method, headers, and the raw response.
    response, send_error := send(.{ method = .PUT, url = tprint("%/notes/%", base, created.id), headers = .[.{ "Content-Type", "application/json" }], body = to_bytes(tprint("{\"title\":\"%\",\"done\":true}", created.title)) });

    if send_error.kind == .NONE {
        print("PUT answered % with %\n", response.status, body_text(*response));
        free_response(*response);
    }

    delete_error := delete_request(tprint("%/notes/%", base, created.id));

    print("deleted: %\n", ifx delete_error.kind == .NONE then "yes" else error_text(delete_error));
}

to_bytes :: (text: string) -> [] u8 {
    return .{ text.count, text.data };
}

main :: () {
    run := Run.{ base = "http://localhost:8080" };

    args := get_command_line_arguments();

    if args.count > 1 run.base = args[1];

    Io.io_run(demo, *run);
}
