// The application: routes, middleware, groups and the settings of the server.
//
//     app: App;
//     get(*app, "/users/{id}", get_user);
//     post(*app, "/users", create_user);
//
//     admin := group(*app, "/admin", require_token);
//     delete(admin, "/users/{id}", delete_user);
//
//     serve(*app);

// What an `App` does when nothing else is said. Every number is a limit or a timeout in
// milliseconds unless it says otherwise; 0 turns the ones that allow it off.
Server_Config :: struct {
    // Where to listen: `":8080"`, `"127.0.0.1:3000"`, `"[::1]:8080"`. Port 0 picks a free one.
    address := ":8080";

    // Schedulers (OS threads), each with its own listener. 0 (or less) is one per processor.
    threads := 0;

    // Every timeout below is in milliseconds, and 0 or less means no limit, except `shutdown_timeout`.

    // How long a client may take to send the request head, from its first byte; also the limit for
    // the TLS handshake of an https connection. Over it: 408.
    header_timeout := 10_000;

    // How long a client may stay silent while sending a request body. Over it: 408.
    body_timeout := 30_000;

    // How long a keep-alive connection may wait for its next request before it is closed. With no
    // limit it waits until the client sends something or hangs up (`stop` still closes it).
    idle_timeout := 60_000;

    // How long one write of a response may take; a slower client is disconnected.
    write_timeout := 30_000;

    // How long `stop` waits for requests in progress before it cuts connections off. Here 0 or less
    // does not mean no limit: the wait is over at once and connections are cut without waiting, so
    // there is no way to wait forever.
    shutdown_timeout := 10_000;

    // Request limits. A request over one is refused with 413, 414 or 431 and the connection closes.
    max_body_size := 1024 * 1024;
    max_head_size := 64 * 1024;
    max_headers := 100;
    max_line_size := 8 * 1024;

    // Requests served on one connection before it is closed (0 or less: no limit).
    max_requests_per_connection := 10_000;

    // Bytes read from a socket at a time (one buffer per connection).
    read_buffer_size := 16 * 1024;

    // The `Server` header; empty sends none.
    server_name := "jaic";

    // Print one line per request to standard output: method, target, status and time.
    log_requests := false;

    // Stop on SIGINT and SIGTERM (Ctrl-C): see `serve`.
    handle_signals := true;

    // Serve HTTPS: every accepted connection does a TLS handshake first (within `header_timeout`).
    // A `Tls_Config` that went through `Tls.tls_config_init`, with a certificate chain and private
    // key, that outlives the server; its `alpn` should be empty or `.["http/1.1"]`. Null: plain
    // HTTP.
    tls: *Tls.Tls_Config;
}

// Called for every error response (a handler's `Http_Error`, a failed extractor, 404, 405, 413...)
// to make the response. Set `app.on_error` to use your own format instead of problem+json.
Error_Handler :: #type (error: Http_Error, request: *Request) -> Response;

App :: struct {
    // Shared state: `app.state = *my_state`; handlers take it as `*State(My_State)`.
    state: *void;

    config: Server_Config;

    // Replaces the problem+json format of error responses.
    on_error: Error_Handler;

    // Limits for the JSON bodies `Json(T)` reads.
    json_options: Json_Options;

    // Middleware that wraps every request, including ones no route answers.
    middleware: [..] Middleware;

    router: Router;

    // Problems found while routes were added (two routes that conflict), reported at startup.
    setup_errors: [..] string;

    // Groups made by `group`, released with the app.
    groups: [..] *void;

    // Set by `shutdown`; the servers of `serve` poll it.
    shutdown_flag: s32;
}

// Releases what the app owns after servers and requests have stopped using it.
free_app :: (app: *App) {
    router_free(*app.router);

    for app.groups {
        // PREFIX is a type parameter; every Group has these same runtime fields.
        group := cast(*Group("")) it;
        array_reset(*group.middleware);
        free(it);
    }

    for app.setup_errors free(it);

    array_reset(*app.groups);
    array_reset(*app.middleware);
    array_reset(*app.setup_errors);
} @Releases

// Adds middleware that wraps every request (in the order added), including ones with no route.
use :: (app: *App, middleware: Middleware) {
    array_add(*app.middleware, middleware);
}

// Asks the servers started by `serve` to stop: they stop accepting and finish the requests in
// progress. Safe to call from any thread; `serve` returns once they are done.
shutdown :: (app: *App) {
    atomic_store(*app.shutdown_flag, 1);
}

// ---------------------------------------------------------------------------------------------
// Groups

// A prefix and middleware shared by the routes registered through it. The prefix is part of the
// type, so a handler's `Path("org", ...)` is checked against the whole route, prefix included.
Group :: struct(PREFIX: string) {
    app: *App;
    middleware: [..] Middleware;
}

// Routes under `prefix`, wrapped by `middleware`, which runs after the app's middleware:
//
//     admin := group(*app, "/admin", require_token);
//     get(admin, "/stats", stats);               // GET /admin/stats
//     v1 := group(admin, "/v1");                 // groups nest; the middleware add up
group :: (app: *App, $prefix: string, middleware: ..Middleware) -> *Group(prefix) {
    created := New(Group(prefix));

    created.app = app;

    for middleware array_add(*created.middleware, it);

    array_add(*app.groups, created);

    return created;
}

group :: (parent: *Group($P), $prefix: string, middleware: ..Middleware) -> *Group(#run join_template(P, prefix)) {
    created := New(Group(#run join_template(P, prefix)));

    created.app = parent.app;

    for parent.middleware array_add(*created.middleware, it);
    for middleware array_add(*created.middleware, it);

    array_add(*parent.app.groups, created);

    return created;
}

// Adds middleware to a group for the routes registered after this call.
use :: (group: *Group($P), middleware: Middleware) {
    array_add(*group.middleware, middleware);
}

// ---------------------------------------------------------------------------------------------
// Routes

// `get(*app, "/users/{id}", handler)`. The template and the handler are compile-time arguments:
// the template is parsed and every extractor of the handler is checked against it while the
// program compiles, and the code that fills them in is generated for this route. A route that
// does not fit is an error that points at the call.
get :: (app: *App, $template: string, $handler: $H, $loc := #caller_location) {
    add_route(app, .[], .GET, "", template, handler, loc);
}

get :: (group: *Group($P), $template: string, $handler: $H, $loc := #caller_location) {
    add_route(group.app, group.middleware, .GET, P, template, handler, loc);
}

post :: (app: *App, $template: string, $handler: $H, $loc := #caller_location) {
    add_route(app, .[], .POST, "", template, handler, loc);
}

post :: (group: *Group($P), $template: string, $handler: $H, $loc := #caller_location) {
    add_route(group.app, group.middleware, .POST, P, template, handler, loc);
}

put :: (app: *App, $template: string, $handler: $H, $loc := #caller_location) {
    add_route(app, .[], .PUT, "", template, handler, loc);
}

put :: (group: *Group($P), $template: string, $handler: $H, $loc := #caller_location) {
    add_route(group.app, group.middleware, .PUT, P, template, handler, loc);
}

patch :: (app: *App, $template: string, $handler: $H, $loc := #caller_location) {
    add_route(app, .[], .PATCH, "", template, handler, loc);
}

patch :: (group: *Group($P), $template: string, $handler: $H, $loc := #caller_location) {
    add_route(group.app, group.middleware, .PATCH, P, template, handler, loc);
}

delete :: (app: *App, $template: string, $handler: $H, $loc := #caller_location) {
    add_route(app, .[], .DELETE, "", template, handler, loc);
}

delete :: (group: *Group($P), $template: string, $handler: $H, $loc := #caller_location) {
    add_route(group.app, group.middleware, .DELETE, P, template, handler, loc);
}

// HEAD and OPTIONS are answered for every route by default (HEAD by the GET handler without the
// body, OPTIONS with an `Allow` list); register them only to change that.
head :: (app: *App, $template: string, $handler: $H, $loc := #caller_location) {
    add_route(app, .[], .HEAD, "", template, handler, loc);
}

head :: (group: *Group($P), $template: string, $handler: $H, $loc := #caller_location) {
    add_route(group.app, group.middleware, .HEAD, P, template, handler, loc);
}

options :: (app: *App, $template: string, $handler: $H, $loc := #caller_location) {
    add_route(app, .[], .OPTIONS, "", template, handler, loc);
}

options :: (group: *Group($P), $template: string, $handler: $H, $loc := #caller_location) {
    add_route(group.app, group.middleware, .OPTIONS, P, template, handler, loc);
}

#scope_module

add_route :: (app: *App, middleware: [] Middleware, $METHOD: Method, $PREFIX: string, $TEMPLATE: string, $handler: $H, $LOC: Source_Code_Location) {
    FULL :: #run join_template(PREFIX, TEMPLATE);

    #insert #run generate_prelude(H);

    thunk :: (request: *Request) -> Response {
        #insert #run generate_call(H);
    }

    register_route(app, METHOD, FULL, thunk, middleware);
}

// `/admin` + `/users/{id}` is `/admin/users/{id}`. Runs at compile time.
join_template :: (prefix: string, template: string) -> string {
    if prefix.count == 0 || prefix == "/" return template;

    if template == "/" || template.count == 0 return prefix;

    if ends_with(prefix, "/") && begins_with(template, "/") return tprint("%%", prefix, slice(template, 1, template.count - 1));

    return tprint("%%", prefix, template);
}

register_route :: (app: *App, method: Method, template: string, handler: Handler, middleware: [] Middleware) {
    route := New(Route);

    route.method = method;
    route.template = template;
    route.handler = handler;

    if middleware.count {
        copy := NewArray(middleware.count, Middleware);

        for middleware copy[it_index] = it;

        route.middleware = copy;
    }

    problem := router_add(*app.router, route);

    if problem.count {
        array_add(*app.setup_errors, copy_string(problem));

        array_reset(*route.names);
        array_free(route.middleware);
        free(route);
    }
}

// ---------------------------------------------------------------------------------------------
// Handling one request

// Runs `request` through the app's middleware and then the routes.
handle_request :: (app: *App, request: *Request) -> Response {
    request.app = app;
    request.chain = app.middleware;
    request.cursor = 0;
    request.endpoint = dispatch_route;

    return call_next(request);
}

call_next :: (request: *Request) -> Response {
    if request.cursor < request.chain.count {
        middleware := request.chain[request.cursor];

        request.cursor += 1;

        return middleware(request, call_next);
    }

    return request.endpoint(request);
}

dispatch_route :: (request: *Request) -> Response {
    app := request.app;

    // Methods this server has no route kind for.
    if request.method == .OTHER || request.method == .CONNECT || request.method == .TRACE {
        error := http_error(501, tprint("the method % is not implemented", request.method_text));

        return error_response(request, error);
    }

    path := request.path;

    if path == "*" && request.method == .OPTIONS {
        response := status_only(200);

        add_header(*response, "Allow", "GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS");

        return response;
    }

    raw_segments := split_path(path,, temp);
    segments := NewArray(raw_segments.count, string,, temp);

    for raw_segments segments[it_index] = percent_decode(it,, temp);

    match := router_match(*app.router, segments, path, request.method,, temp);

    if !match.route {
        if match.path_node {
            allow := allowed_methods(match.path_node);

            if request.method == .OPTIONS {
                response := status_only(204);

                add_header(*response, "Allow", allow);

                return response;
            }

            error := http_error(405, tprint("% is not allowed for %; use %", request.method_text, request.path, allow));

            response := error_response(request, error);

            add_header(*response, "Allow", allow);

            return response;
        }

        error := http_error(404, tprint("no route for % %", request.method_text, request.path));

        return error_response(request, error);
    }

    route := match.route;

    values := NewArray(match.values.count, Pair,, temp);

    for match.values {
        values[it_index].key = route.names[it_index];
        values[it_index].value = it;
    }

    request.params = values;
    request.chain = route.middleware;
    request.cursor = 0;
    request.endpoint = route.handler;

    return call_next(request);
}
