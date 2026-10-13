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
#load "notes.jai";

main :: () {
    store: Store;
    app: App;

    notes_routes(*app, *store);

    app.config.threads = 1;
    app.config.log_requests = true;

    print("listening on http://localhost:8080 (Ctrl-C to stop)\n");

    serve(*app);
}
