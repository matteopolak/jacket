// A JSON API for notes, kept in memory.
//
//     jaic run examples/http/crud.jai
//     curl -i -X POST localhost:8080/notes -H 'content-type: application/json' -d '{"title":"milk"}'
//     curl localhost:8080/notes/1
//     curl localhost:8080/notes?status=open
//
// Handlers declare what they need as parameters. A bad id, a body that is not JSON or a missing
// field is answered with a 400/415/422 problem document before the handler runs.
#import "Basic";
#import "Extensions/Http";

Note :: struct {
    id: int;
    title: string;
    done: bool;
}

New_Note :: struct {
    title: string;
    done: bool; @json(default)
}

// Query parameters are read into a struct; a field marked `default` may be left out.
Filter :: struct {
    status := "all"; @json(default) // all, open or done
}

// The shared state. One thread serves all requests (config.threads below), so no lock is needed.
Store :: struct {
    notes: [..] Note;
    next_id := 1;
}

list_notes :: (filter: Query(Filter), store: *State(Store)) -> [] Note {
    if filter.status == "all" return store.notes;

    matching: [..] Note;
    matching.allocator = temp;

    for store.notes if it.done == (filter.status == "done") array_add(*matching, it);

    return matching;
}

get_note :: (id: Path("id", int), store: *State(Store)) -> Note, Http_Error {
    for store.notes if it.id == id.value return it, .{};

    return .{}, not_found("No note with id %", id.value);
}

create_note :: (body: Json(New_Note), store: *State(Store)) -> Note, Http_Error {
    if !body.title return .{}, unprocessable("title must not be empty");

    note := Note.{ store.next_id, copy_string(body.title), body.done };

    store.next_id += 1;
    array_add(*store.notes, note);

    return note, .{}; // a value from POST is sent as 201 Created
}

replace_note :: (id: Path("id", int), body: Json(New_Note), store: *State(Store)) -> Note, Http_Error {
    for *store.notes {
        if it.id == id.value {
            it.title = copy_string(body.title);
            it.done = body.done;

            return it.*, .{};
        }
    }

    return .{}, not_found("No note with id %", id.value);
}

delete_note :: (id: Path("id", int), store: *State(Store)) -> Http_Error {
    for store.notes {
        if it.id == id.value {
            array_ordered_remove_by_index(*store.notes, it_index);

            return .{}; // nothing to return: 204 No Content
        }
    }

    return not_found("No note with id %", id.value);
}

main :: () {
    store: Store;
    app: App;

    app.state = *store;
    app.config.threads = 1;
    app.config.log_requests = true;

    get(*app, "/notes", list_notes);
    post(*app, "/notes", create_note);
    get(*app, "/notes/{id}", get_note);
    put(*app, "/notes/{id}", replace_note);
    delete(*app, "/notes/{id}", delete_note);

    print("listening on http://localhost:8080 (Ctrl-C to stop)\n");

    serve(*app);
}
