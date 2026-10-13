// Every extractor through the test client (no socket), with the errors they give: Path, Params, Rest, Query,
// Json, Form, Header, Header_Optional, Bytes, Text, *Request and *State.
#load "lib/server_support.jai";

Color :: enum {
    red;
    green;
    blue;
}

Point :: struct {
    x: int;
    y: int;
}

Search :: struct @json(rename_all=camelCase) {
    q: string;
    page_size := 20; @json(default)
    tags: [..] string; @json(default)
    color := Color.red; @json(default)
}

Strict :: struct @json(deny_unknown_fields) {
    a: int;
}

New_Item :: struct {
    name: string;
    tags: [..] string; @json(default)
    point: Point; @json(default)
}

Login :: struct {
    user: string;
    remember: bool; @json(default)
}

Key :: struct {
    org: string;
    id: int;
}

Counter :: struct {
    hits: int;
    label: string;
}

// ---- Path

path_float :: (x: Path("x", float64)) -> string {
    return tprint("%", x.value);
}

path_bool :: (flag: Path("flag", bool)) -> string {
    return ifx flag.value then "yes" else "no";
}

path_enum :: (color: Path("color", Color)) -> string {
    return tprint("%", color.value);
}

path_small :: (n: Path("n", s8)) -> string {
    return tprint("%", n.value);
}

path_string :: (s: Path("s", string)) -> string {
    return s.value;
}

params :: (key: Params(Key)) -> string {
    return tprint("% %", key.org, key.id);
}

// ---- Query

search :: (query: Query(Search)) -> string {
    return tprint("q=% size=% tags=% color=%", query.q, query.page_size, query.tags.count, query.color);
}

strict :: (query: Query(Strict)) -> string {
    return tprint("a=%", query.a);
}

// ---- Json

create_item :: (body: Json(New_Item)) -> New_Item {
    return body.value;
}

echo_point :: (body: Json(Point)) -> Point {
    return .{ body.y, body.x };
}

sum_numbers :: (body: Json([] int)) -> int {
    total := 0;
    for body.value total += it;
    return total;
}

// ---- Form, Header, bodies

login :: (form: Form(Login)) -> string {
    return tprint("%:%", form.user, form.remember);
}

need_agent :: (agent: Header("user-agent", string)) -> string {
    return agent.value;
}

need_count :: (count: Header("x-count", int)) -> string {
    return tprint("%", count.value + 1);
}

maybe_trace :: (trace: Header_Optional("x-trace", string)) -> string {
    return ifx trace.present then tprint("trace=%", trace.value) else "no trace";
}

raw_bytes :: (body: Bytes) -> string {
    return tprint("% bytes", body.value.count);
}

raw_text :: (body: Text) -> string {
    return tprint("<%>", body.value);
}

about_request :: (request: *Request) -> string {
    agent, _ := header(request, "x-agent");
    q, has_q := query_param(request, "q");
    return tprint("% % q=%/% agent=% peer=%", request.method_text, request.path, has_q, q, agent, request.peer.family);
}

bump :: (counter: *State(Counter)) -> string {
    counter.hits += 1;
    return tprint("% %", counter.label, counter.hits);
}

mixed :: (id: Path("id", int), query: Query(Search), body: Json(Point), counter: *State(Counter), request: *Request) -> string {
    return tprint("id=% q=% x=% label=% method=%", id.value, query.q, body.x, counter.label, request.method_text);
}

counter: Counter;
app: App;

extractors :: (api: *Client) {
    // ---- Path
    response := get_path(api, "/path/float/2.5");

    check(body_of(response) == "2.5", "float: %", body_of(response));

    response = get_path(api, "/path/float/x");
    check(response.status == 400 && contains(body_of(response), "expected a number"), "bad float: % %", response.status, body_of(response));

    response = get_path(api, "/path/bool/true");
    check(body_of(response) == "yes", "bool: %", body_of(response));

    response = get_path(api, "/path/bool/maybe");
    check(response.status == 400 && contains(body_of(response), "\"field\":\"flag\""), "bad bool names the field: %", body_of(response));

    response = get_path(api, "/path/enum/green");
    check(body_of(response) == "green", "enum: %", body_of(response));

    response = get_path(api, "/path/enum/purple");
    check(response.status == 400 && contains(body_of(response), "purple"), "unknown enum member: % %", response.status, body_of(response));

    response = get_path(api, "/path/small/-128");
    check(body_of(response) == "-128", "s8 low end: %", body_of(response));

    response = get_path(api, "/path/small/300");
    check(response.status == 400 && contains(body_of(response), "`300` is out of range"), "s8 out of range: % %", response.status, body_of(response));

    response = get_path(api, "/path/string/a%2Bb%20c");
    check(body_of(response) == "a+b c", "string: %", body_of(response));

    response = get_path(api, "/params/acme/7");
    check(body_of(response) == "acme 7", "Params: %", body_of(response));

    response = get_path(api, "/params/acme/seven");
    check(response.status == 400 && contains(body_of(response), "`id`"), "Params with a bad field: % %", response.status, body_of(response));

    // ---- Query
    response = get_path(api, "/search?q=jai");
    check(body_of(response) == "q=jai size=20 tags=0 color=red", "query defaults: %", body_of(response));

    response = get_path(api, "/search?q=hello%20world&pageSize=5&tags=a&tags=b&tags=c&color=blue");
    check(body_of(response) == "q=hello world size=5 tags=3 color=blue", "query values: %", body_of(response));

    response = get_path(api, "/search?q=a+b");
    check(body_of(response) == "q=a b size=20 tags=0 color=red", "+ is a space in a query: %", body_of(response));

    response = get_path(api, "/search");
    check(response.status == 400 && contains(body_of(response), "missing field `q`") && contains(body_of(response), "\"location\":\"query\""), "missing query field: % %", response.status, body_of(response));

    response = get_path(api, "/search?q=x&pageSize=many");
    check(response.status == 400 && contains(body_of(response), "query parameter `pageSize`") && contains(body_of(response), "got `many`"), "bad query value: % %", response.status, body_of(response));

    response = get_path(api, "/search?q=x&color=pink");
    check(response.status == 400 && contains(body_of(response), "color"), "bad query enum: % %", response.status, body_of(response));

    response = get_path(api, "/search?q=x&unknown=1");
    check(response.status == 200, "unknown query keys are ignored: %", response.status);

    response = get_path(api, "/strict?a=1");
    check(body_of(response) == "a=1", "strict ok: %", body_of(response));

    response = get_path(api, "/strict?a=1&b=2");
    check(response.status == 400 && contains(body_of(response), "unknown field"), "deny_unknown_fields: % %", response.status, body_of(response));

    // ---- Json
    response = post_body(api, "/items", "{\"name\":\"pen\",\"tags\":[\"a\",\"b\"],\"point\":{\"x\":1,\"y\":2}}");
    check(response.status == 201, "POST of a value is 201: %", response.status);
    check(header_of(response, "Content-Type") == "application/json", "JSON content type: %", header_of(response, "Content-Type"));
    check(contains(body_of(response), "\"name\":\"pen\"") && contains(body_of(response), "\"tags\":[\"a\",\"b\"]"), "item echoed: %", body_of(response));

    response = post_body(api, "/items", "{\"name\":\"pen\"}", "application/json; charset=utf-8");
    check(response.status == 201, "Content-Type with a charset: %", response.status);

    response = post_body(api, "/items", "{\"name\":\"pen\"}", "application/vnd.api+json");
    check(response.status == 201, "a +json media type: %", response.status);

    response = post_body(api, "/items", "{\"name\":\"pen\"}", "text/plain");
    check(response.status == 415 && contains(body_of(response), "text/plain"), "wrong media type: % %", response.status, body_of(response));

    response = fetch(api, .POST, "/items", "{\"name\":\"pen\"}");
    check(response.status == 415 && contains(body_of(response), "no Content-Type"), "no media type: % %", response.status, body_of(response));

    response = post_body(api, "/items", "{\"name\":");
    check(response.status == 400 && contains(body_of(response), "\"location\":\"body\""), "broken JSON: % %", response.status, body_of(response));

    response = post_body(api, "/items", "{\"name\":\"pen\",\"tags\":[\"a\",3]}");
    check(response.status == 422 && contains(body_of(response), "tags[1]") && contains(body_of(response), "\"field\":\"tags[1]\""), "wrong type names the path: % %", response.status, body_of(response));

    response = post_body(api, "/items", "{\"tags\":[]}");
    check(response.status == 422 && contains(body_of(response), "missing field `name`"), "missing field: % %", response.status, body_of(response));

    response = post_body(api, "/items", "");
    check(response.status == 400, "empty body: %", response.status);

    response = post_body(api, "/point", "{\"x\":1,\"y\":2}");
    check(body_of(response) == "{\"x\":2,\"y\":1}", "Json(Point) using: %", body_of(response));

    response = post_body(api, "/sum", "[1,2,3,4]");
    check(response.status == 201 && body_of(response) == "10", "Json of an array: % %", response.status, body_of(response));

    // ---- Form
    response = post_body(api, "/login", "user=ada%40example.com&remember=true", "application/x-www-form-urlencoded");
    check(body_of(response) == "ada@example.com:true", "form: %", body_of(response));

    response = post_body(api, "/login", "user=ada", "application/x-www-form-urlencoded");
    check(body_of(response) == "ada:false", "a form field with a default may be left out: % %", response.status, body_of(response));

    response = post_body(api, "/login", "remember=true", "application/x-www-form-urlencoded");
    check(response.status == 400 && contains(body_of(response), "missing field `user`") && contains(body_of(response), "\"location\":\"form\""), "form missing a field: % %", response.status, body_of(response));

    response = post_body(api, "/login", "user=ada", "application/json");
    check(response.status == 415, "form with a JSON type: %", response.status);

    // ---- Headers
    response = get_path(api, "/agent", Header_Field.[.{ "User-Agent", "tests/1" }]);
    check(body_of(response) == "tests/1", "Header: %", body_of(response));

    response = get_path(api, "/count", Header_Field.[.{ "X-Count", "41" }]);
    check(body_of(response) == "42", "Header int: %", body_of(response));

    response = get_path(api, "/count", Header_Field.[.{ "X-Count", "forty" }]);
    check(response.status == 400 && contains(body_of(response), "invalid header `x-count`"), "bad header: % %", response.status, body_of(response));

    response = get_path(api, "/count");
    check(response.status == 400 && contains(body_of(response), "missing required header `x-count`") && contains(body_of(response), "\"location\":\"header\""), "missing header: % %", response.status, body_of(response));

    response = get_path(api, "/trace");
    check(body_of(response) == "no trace", "Header_Optional absent: %", body_of(response));

    response = get_path(api, "/trace", Header_Field.[.{ "X-Trace", "abc" }]);
    check(body_of(response) == "trace=abc", "Header_Optional present: %", body_of(response));

    // ---- Bodies
    response = post_body(api, "/bytes", "hello", "application/octet-stream");
    check(body_of(response) == "5 bytes", "Bytes: %", body_of(response));

    response = post_body(api, "/text", "héllo", "text/plain");
    check(body_of(response) == "<héllo>", "Text: %", body_of(response));

    response = post_body(api, "/bytes", "", "application/octet-stream");
    check(body_of(response) == "0 bytes", "Bytes of nothing: %", body_of(response));

    // ---- Request and State
    response = get_path(api, "/request?q=zz", Header_Field.[.{ "X-Agent", "me" }]);
    check(body_of(response) == "GET /request q=true/zz agent=me peer=IPV4", "Request: %", body_of(response));

    response = get_path(api, "/state");
    response = get_path(api, "/state");
    check(body_of(response) == "counted 2" && counter.hits == 2, "State: % %", body_of(response), counter.hits);

    // ---- All together
    response = post_body(api, "/mixed/9?q=hi", "{\"x\":4,\"y\":5}");
    check(body_of(response) == "id=9 q=hi x=4 label=counted method=POST", "everything at once: %", body_of(response));
}

main :: () {
    if !supported() return;

    counter.label = "counted";
    app.state = *counter;

    get(*app, "/path/float/{x}", path_float);
    get(*app, "/path/bool/{flag}", path_bool);
    get(*app, "/path/enum/{color}", path_enum);
    get(*app, "/path/small/{n}", path_small);
    get(*app, "/path/string/{s}", path_string);
    get(*app, "/params/{org}/{id}", params);
    get(*app, "/search", search);
    get(*app, "/strict", strict);
    post(*app, "/items", create_item);
    post(*app, "/point", echo_point);
    post(*app, "/sum", sum_numbers);
    post(*app, "/login", login);
    get(*app, "/agent", need_agent);
    get(*app, "/count", need_count);
    get(*app, "/trace", maybe_trace);
    post(*app, "/bytes", raw_bytes);
    post(*app, "/text", raw_text);
    get(*app, "/request", about_request);
    get(*app, "/state", bump);
    post(*app, "/mixed/{id}", mixed);

    run_app(*app, extractors);

    finish();
}
