// The HTTP client against the notes API of crud.jai (start that first, or pass another base URL).
//
//     jaic examples/http/client.jai -interp
//     jaic examples/http/client.jai -interp -- http://localhost:3000
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
    // A zeroed Client works; the base URL lets requests name just the path.
    api: Client;

    api.base_url = run.base;

    // A request returns once the status and headers are in; the body is read from the response.
    response, error := post(*api, "/notes", New_Note.{ "write the docs", false });

    if error.kind != .NONE {
        print("could not create a note: %\n", error_text(error));
        return;
    }

    defer close(*response);

    created, decode_error := read_json(*response, Note);

    if decode_error.kind != .NONE {
        print("the answer was not a note: %\n", error_text(decode_error));
        return;
    }

    print("created #% (status %): %\n", created.id, response.status, created.title);

    note_response, _ := get(*api, tprint("/notes/%", created.id));
    note, _ := read_json(*note_response, Note);

    close(*note_response);

    print("read it back: % (done: %)\n", note.title, note.done);

    // A 404 is a response too. `error_for_status` turns a non-2xx one into an error that carries
    // the problem document's title and detail.
    missing, _ := get(*api, "/notes/99999");

    missing_error := error_for_status(*missing);

    close(*missing);

    print("asking for a missing note: % (%)\n", missing_error.status, missing_error.detail);

    // `send` for everything else: a method, headers and a raw body.
    updated, send_error := send(*api, .{ method = .PUT, url = tprint("/notes/%", created.id), headers = .[.{ "Content-Type", "application/json" }], body = to_bytes(tprint("{\"title\":\"%\",\"done\":true}", created.title)) });

    if send_error.kind == .NONE {
        text, _ := read_text(*updated);

        print("PUT answered % with %\n", updated.status, text);
    }

    close(*updated);

    deleted, delete_error := delete(*api, tprint("/notes/%", created.id));

    delete_status := deleted.status;

    close(*deleted);

    print("deleted: %\n", ifx delete_error.kind == .NONE && delete_status == 204 then "yes" else error_text(delete_error));
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
