// Real enclosing consumer for contextual lint of Http's loaded dependencies.
#import "Basic";
Http :: #import "Extensions/Http";

handler :: (request: *Http.Request) -> Http.Response {
    return Http.text(request.path);
}

main :: () {
    app: Http.App;

    Http.get(*app, "/", handler);
    defer Http.free_app(*app);
    client: Http.Client;

    Http.set_header(*client, "Accept", "application/json");
    assert(client.headers.count == 1);
    Http.remove_header(*client, "Accept");
    assert(client.headers.count == 0);
    array_free(client.headers);
}
