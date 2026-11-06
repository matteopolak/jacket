// test: skip_on(.WASM, reason = "native RPC/HTTP scheduler")
#load "support/server_support.jai";

answer :: (request: *Request, data: *void) -> Response {
    count := cast(*int) data;
    count.* += 1;
    return text(tprint("% %", request.path, count.*));
}

middleware_calls := 0;

mark :: (request: *Request, next: Next) -> Response {
    middleware_calls += 1;
    return with_header(next(request), "x-context", "through middleware");
}

main :: () {
    app: App;

    defer free_app(*app);
    use(*app, mark);
    first := 10;
    second := 20;
    owned_path := copy_string("/first.Service/Method");

    register_handler(*app, .GET, owned_path, answer, *first);
    free(owned_path.data);
    register_handler(*app, .GET, "/second.Service/Method", answer, *second);
    api := test_client(*app);
    response, error := get(api, "/first.Service/Method");

    assert(error.kind == .NONE);
    body, read_error := read_text(*response);

    assert(read_error.kind == .NONE && body == "/first.Service/Method 11");
    close(*response);
    response, error = get(api, "/second.Service/Method");
    assert(error.kind == .NONE);
    body, read_error = read_text(*response);
    assert(read_error.kind == .NONE && body == "/second.Service/Method 21");
    value, found := header(*response, "x-context");

    assert(found && value == "through middleware" && middleware_calls == 2);
    close(*response);
    assert(first == 11 && second == 21);
    invalid: App;

    defer free_app(*invalid);
    register_handler(*invalid, .GET, "/first", answer, *first);
    register_handler(*invalid, .GET, "/first", answer, *second);
    register_handler(*invalid, .GET, "invalid path", answer, *second);
    register_handler(*invalid, .GET, "/missing", null, *second);
    assert(invalid.setup_errors.count == 3);
    finish();
}
