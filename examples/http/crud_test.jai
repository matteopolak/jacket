// End-to-end tests for the notes API (notes.jai), with no socket and no setup.
//
//     jaic run examples/http/crud_test.jai
//
// `test_client` is the ordinary HTTP client wired to the app in memory: every request goes
// through the real connection loop (parsing, routing, extractors, error mapping, the response
// writer), only the network is missing. Use it with the calls you know, `get`, `post`,
// `send`, and check the answers with the `expect_*` procedures, which print the
// request and the response when something is not as expected. The store is an ordinary variable
// of the test, so state can be prepared and inspected directly.
#import "Basic";
#import "Extensions/Http";
#load "notes.jai";

main :: () {
    run_tests(.[
        .{ "starts empty", starts_empty },
        .{ "creates a note and reads it back", creates_and_reads },
        .{ "filters by status", filters_by_status },
        .{ "replaces and deletes", replaces_and_deletes },
        .{ "answers bad requests with problem documents", rejects_bad_requests },
        .{ "reads bodies from responses", reads_bodies },
        .{ "works over a real socket too", over_a_socket },
    ]);
}

starts_empty :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    api := test_client(*app);

    notes := expect_json(get(api, "/notes"), [] Note);

    expect_equal(notes.count, 0);
}

creates_and_reads :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    api := test_client(*app);

    created := expect_json(post(api, "/notes", New_Note.{ title = "milk" }), Note, status = 201);

    expect_equal(created.id, 1);
    expect_equal(created.title, "milk");
    expect_equal(store.notes.count, 1);

    fetched := expect_json(get(api, "/notes/1"), Note);

    expect_equal(fetched.title, "milk");
    expect_equal(fetched.done, false);
}

filters_by_status :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    // Prepare the state directly instead of going through the API.
    array_add(*store.notes, Note.{ 1, "milk", false });
    array_add(*store.notes, Note.{ 2, "eggs", true });
    array_add(*store.notes, Note.{ 3, "flour", false });

    store.next_id = 4;

    api := test_client(*app);

    open := expect_json(get(api, "/notes?status=open"), [] Note);
    done := expect_json(get(api, "/notes?status=done"), [] Note);

    expect_equal(open.count, 2);
    expect_equal(done.count, 1);
    expect_equal(done[0].title, "eggs");
}

replaces_and_deletes :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    api := test_client(*app);

    post(api, "/notes", New_Note.{ title = "milk" });

    replaced := expect_json(put(api, "/notes/1", New_Note.{ title = "oat milk", done = true }), Note);

    expect_equal(replaced.title, "oat milk");
    expect_equal(store.notes[0].done, true);

    deleted := delete(api, "/notes/1");

    expect_status(deleted, 204);
    expect_equal(store.notes.count, 0);

    expect_problem(get(api, "/notes/1"), 404, "No note with id 1");
}

rejects_bad_requests :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    api := test_client(*app);

    // The handler refuses.
    expect_problem(post(api, "/notes", New_Note.{ title = "" }), 422, "title must not be empty");

    // The extractors refuse before the handler runs.
    expect_problem(get(api, "/notes/abc"), 400, "invalid path parameter `id`");
    expect_problem(send(api, .{ method = .POST, url = "/notes", body = bytes_of("{\"title\":") }), 415);

    json := Header_Field.[.{ "Content-Type", "application/json" }];

    broken := send(api, .{ method = .POST, url = "/notes", headers = json, body = bytes_of("{\"title\":") });

    expect_problem(broken, 400);

    missing := send(api, .{ method = .POST, url = "/notes", headers = json, body = bytes_of("{}") });

    problem := expect_problem(missing, 422);

    expect_equal(problem.location, "body");

    // The router answers what no handler does.
    expect_problem(get(api, "/nothing"), 404);

    not_allowed := delete(api, "/notes");

    expect_problem(not_allowed, 405);
    expect_header(not_allowed, "Allow", "GET, HEAD, POST, OPTIONS");

    expect_equal(store.notes.count, 0, "none of the bad requests stored anything");
}

reads_bodies :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    // Anything the real client can do works on a test client: the response holds the status and
    // headers, and the body is read from it afterwards.
    api := test_client(*app);

    response := post(api, "/notes", New_Note.{ title = "milk" });
    note, error := read_json(*response, Note);

    expect(error.kind == .NONE, "creating a note works");
    expect_equal(note.id, 1);

    missing_response := get(api, "/notes/42");
    missing := error_for_status(*missing_response);

    expect_equal(missing.kind, Client_Error_Kind.STATUS);
    expect_equal(missing.status, 404);
    expect_equal(missing.detail, "No note with id 42");

    list := get(api, "/notes");

    expect_status(list, 200);
    expect_header(list, "Content-Type", "application/json");
}

over_a_socket :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    // The same test code against a real server on a free loopback port.
    api := test_client(*app, .{ over_tcp = true });

    created := expect_json(post(api, "/notes", New_Note.{ title = "milk" }), Note, status = 201);

    expect_equal(created.id, 1);
    expect_equal(store.notes.count, 1);
}
