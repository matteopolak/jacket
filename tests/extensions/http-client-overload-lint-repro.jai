// Original reduction of the production client-context diagnostic. Compile and
// interpreter accept the App overload; lint must not encode its handler as a
// client POST payload while inspecting a rejected candidate.
Http :: #import "Extensions/Http";

handler :: (request: *Http.Request) -> Http.Response {
    return Http.text(request.path);
}

main :: () {
    app: Http.App;
    Http.post(*app, "/", handler);
    Http.free_app(*app);
}
