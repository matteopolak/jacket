// An echo server for whole-server HTTP checks: every method on every path answers 200 with the
// request line, the headers and the body it received, as text. tools/http_conformance.py builds
// it and probes it over TCP; it is also the program to put in a http-garden image.
//
//     jaic build tools/http_garden/server.jai -o echo_server
//     ./echo_server 127.0.0.1:8080
#import "Basic";
#import "Extensions/Http";

echo :: (request: *Request) -> string {
    builder: String_Builder;

    print(*builder, "% % %\n", request.method_text, request.target, version_name(request.version));

    for request.headers print(*builder, "%: %\n", it.name, it.value);

    print(*builder, "\n%", body_text(request));

    return builder_to_string(*builder,, temp);
}

main :: () {
    app: App;

    app.config.address = "127.0.0.1:8080";
    app.config.max_body_size = 16 * 1024 * 1024;

    args := get_command_line_arguments();

    if args.count > 1 app.config.address = args[1];

    get(*app, "/{rest...}", echo);
    post(*app, "/{rest...}", echo);
    put(*app, "/{rest...}", echo);
    patch(*app, "/{rest...}", echo);
    delete(*app, "/{rest...}", echo);
    get(*app, "/", echo);
    post(*app, "/", echo);

    if !serve(*app) exit(1);
}
