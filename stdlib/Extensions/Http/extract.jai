// Extractors: handler parameters that say what part of the request they want. The route
// registration (app.jai) reads the handler's parameter types at compile time, checks them
// against the route template and generates the code that fills them in, so a handler is just a
// function:
//
//     get_user :: (id: Path("id", int), search: Query(Search), request: *Request) -> User, Http_Error { ... }
//
// Each extractor is a polymorphic struct with a `value`. The ones that hold a struct also
// `using` it, so `body.name` works as well as `body.value.name`.

// One `{name}` of the route, parsed as a `T`: integers, floats, bools, strings and enums (by
// member name). A value that does not parse is a 400 that names the parameter.
Path :: struct(NAME: string, T: Type) {
    value: T;
}

// Several `{params}` at once: every field of `T` names a parameter of the route.
//
//     Post_Key :: struct { user: int; post: string; }
//     get_post :: (key: Params(Post_Key)) { ... key.user ... }
Params :: struct(T: Type) {
    using value: T;
}

// The `{name...}` tail of the route as one string (the rest of the path, percent-decoded).
Rest :: struct(NAME: string) {
    value: string;
}

// The query string, decoded into a `T` with the rules and notes of JSON (`default`, `rename`,
// `alias`...): a field without `default` is required, repeated keys fill arrays.
Query :: struct(T: Type) {
    using value: T;
}

// The request body as JSON. `Content-Type: application/json` is required (415 otherwise), a body
// that is not valid JSON is a 400 and one of the wrong shape a 422; both name the JSON path.
Json :: struct(T: Type) {
    using value: T;
}

// An `application/x-www-form-urlencoded` body, decoded like a query string.
Form :: struct(T: Type) {
    using value: T;
}

// One request header, parsed as a `T`. A missing header is a 400; `Header_Optional` allows it.
Header :: struct(NAME: string, T: Type) {
    value: T;
}

Header_Optional :: struct(NAME: string, T: Type) {
    value: T;
    present: bool;
}

// The raw body.
Bytes :: struct {
    value: [] u8;
}

// The body as text.
Text :: struct {
    value: string;
}

// The shared state set with `app.state = *my_state`. Take it as `*State(My_State)`; the fields
// of `My_State` are reachable directly. The pointer is the state itself, not a copy: handlers on
// several threads see the same memory, so guard what they change.
State :: struct(T: Type) {
    using value: T;
}

#scope_module

// ---------------------------------------------------------------------------------------------
// Run time: one procedure per extractor. The generated thunk of a route calls them.

extract_path :: (request: *Request, position: int, name: string, $T: Type) -> T, Http_Error {
    text := request.params[position].value;

    #if T == string {
        return text, .{};
    } else {
        value, failure := from_param(T, text,, temp);

        if failure.kind != .NONE {
            return .{}, input_error(400, "path", name, tprint("invalid path parameter `%`: %", name, failure.message));
        }

        return value, .{};
    }
}

extract_params :: (request: *Request, $T: Type) -> T, Http_Error {
    value, failure := from_query(T, request.params,, temp);

    if failure.kind != .NONE {
        return .{}, input_error(400, "path", failure.path, tprint("invalid path parameter%: %", ifx failure.path.count then tprint(" `%`", failure.path) else "", failure.message));
    }

    return value, .{};
}

extract_query :: (request: *Request, $T: Type) -> T, Http_Error {
    pairs := query_pairs(request);

    value, failure := from_query(T, pairs,, temp);

    if failure.kind != .NONE {
        return .{}, input_error(400, "query", failure.path, bad_pairs_detail("query parameter", failure, pairs));
    }

    return value, .{};
}

extract_form :: (request: *Request, $T: Type) -> T, Http_Error {
    content_type, found := header(request, .CONTENT_TYPE);

    if !found || !media_type_is(content_type, "application/x-www-form-urlencoded") {
        return .{}, input_error(415, "body", "", ifx found then tprint("expected a form body (Content-Type: application/x-www-form-urlencoded), got `%`", content_type) else "expected a form body (Content-Type: application/x-www-form-urlencoded), but there was no Content-Type header");
    }

    pairs := parse_query(body_text(request),, temp);

    value, failure := from_query(T, pairs,, temp);

    if failure.kind != .NONE {
        return .{}, input_error(400, "form", failure.path, bad_pairs_detail("form field", failure, pairs));
    }

    return value, .{};
}

extract_json :: (request: *Request, $T: Type) -> T, Http_Error {
    content_type, found := header(request, .CONTENT_TYPE);

    if !found || !(media_type_is(content_type, "application/json") || media_type_has_suffix(content_type, "+json")) {
        return .{}, input_error(415, "body", "", ifx found then tprint("expected a JSON body (Content-Type: application/json), got `%`", content_type) else "expected a JSON body (Content-Type: application/json), but there was no Content-Type header");
    }

    value, failure := from_json(T, body_text(request), request.app.json_options,, temp);

    if failure.kind != .NONE {
        status := ifx failure.kind == .SYNTAX || failure.kind == .LIMIT then 400 else 422;

        where := ifx failure.path.count then tprint("%: ", failure.path) else "";

        return .{}, input_error(status, "body", failure.path, tprint("%%, at line %, column %", where, failure.message, failure.line, failure.column));
    }

    return value, .{};
}

extract_header :: (request: *Request, name: string, $T: Type) -> T, Http_Error {
    text, found := header(request, name);

    if !found {
        return .{}, input_error(400, "header", name, tprint("missing required header `%`", name));
    }

    #if T == string {
        return text, .{};
    } else {
        value, failure := from_param(T, text,, temp);

        if failure.kind != .NONE {
            return .{}, input_error(400, "header", name, tprint("invalid header `%`: %", name, failure.message));
        }

        return value, .{};
    }
}

extract_header_optional :: (request: *Request, name: string, $T: Type) -> T, bool, Http_Error {
    text, found := header(request, name);

    if !found return .{}, false, .{};

    value, error := extract_header(request, name, T);

    return value, true, error;
}

// What went wrong in a query or form: the field, the problem and the text that was given.
bad_pairs_detail :: (what: string, failure: Json_Error, pairs: [] Pair) -> string {
    if failure.path.count == 0 return failure.message;

    for pairs {
        if it.key == failure.path {
            return tprint("% `%`: % (got `%`)", what, failure.path, failure.message, it.value);
        }
    }

    return tprint("% `%`: %", what, failure.path, failure.message);
}

// `application/json; charset=utf-8` is the media type `application/json`.
media_type_is :: (value: string, wanted: string) -> bool {
    return equal_nocase(media_type_of(value), wanted);
}

media_type_has_suffix :: (value: string, suffix: string) -> bool {
    media := media_type_of(value);

    return media.count > suffix.count && equal_nocase(slice(media, media.count - suffix.count, suffix.count), suffix);
}

media_type_of :: (value: string) -> string {
    media := value;
    semicolon := find_index_from_left(media, #char ";");

    if semicolon >= 0 media = slice(media, 0, semicolon);

    while media.count > 0 && is_ows(media[media.count - 1]) media.count -= 1;

    while media.count > 0 && is_ows(media[0]) {
        media.data += 1;
        media.count -= 1;
    }

    return media;
}

// ---------------------------------------------------------------------------------------------
// The value a handler returned, as a response

respond :: (request: *Request, value: Response) -> Response {
    return value;
}

respond :: (request: *Request, value: string) -> Response {
    return text(value);
}

respond :: (request: *Request, value: [] u8) -> Response {
    return bytes_response(value, "application/octet-stream");
}

// A handler whose only result is an error: no error is 204.
respond :: (request: *Request, value: Http_Error) -> Response {
    if value.kind != .NONE return error_response(request, value);

    return no_content();
}

// Anything else is JSON: 201 Created for a POST, 200 otherwise.
respond :: (request: *Request, value: $T) -> Response {
    return json(value, ifx request.method == .POST then 201 else 200);
}

// ---------------------------------------------------------------------------------------------
// Compile time: what a route registration checks and generates

Param_Kind :: enum u8 {
    UNKNOWN;
    PATH;
    PARAMS;
    REST;
    QUERY;
    JSON;
    HEADER;
    HEADER_OPTIONAL;
    FORM;
    BYTES;
    TEXT;
    REQUEST;
    STATE;
}

classify :: (info: *Type_Info) -> Param_Kind {
    if info.type == .STRUCT {
        name := (cast(*Type_Info_Struct) info).name;

        if name == "Path" return .PATH;
        if name == "Params" return .PARAMS;
        if name == "Rest" return .REST;
        if name == "Query" return .QUERY;
        if name == "Json" return .JSON;
        if name == "Header" return .HEADER;
        if name == "Header_Optional" return .HEADER_OPTIONAL;
        if name == "Form" return .FORM;
        if name == "Bytes" return .BYTES;
        if name == "Text" return .TEXT;
    }

    if info.type == .POINTER {
        pointee := (cast(*Type_Info_Pointer) info).pointer_to;

        if pointee.type == .STRUCT {
            name := (cast(*Type_Info_Struct) pointee).name;

            if name == "Request" return .REQUEST;
            if name == "State" return .STATE;
        }
    }

    return .UNKNOWN;
}

is_body_kind :: (kind: Param_Kind) -> bool {
    return kind == .JSON || kind == .FORM || kind == .BYTES || kind == .TEXT;
}

// The type of the `index`th parameter of the procedure type `H`.
arg_type :: (H: Type, index: int) -> Type {
    info := cast(*Type_Info_Procedure) H;

    return cast(Type) info.argument_types[index];
}

// The T of an extractor that holds a struct in its `value` member.
value_info :: (info: *Type_Info) -> *Type_Info {
    for (cast(*Type_Info_Struct) info).members {
        if it.name == "value" return it.type;
    }

    return null;
}

route_name :: (method: Method, template: string) -> string {
    return tprint("%s %s", method_name(method), template);
}

// A problem with a route, reported at the line that registered it.
report :: (loc: Source_Code_Location, method: Method, template: string, message: string) {
    compiler_report(loc.fully_pathed_filename, loc.line_number, loc.character_number, tprint("Http: route `% %`: %", method_name(method), template, message));
}

// A template that cannot be parsed.
check_template :: (template: string, method: Method, loc: Source_Code_Location) {
    parsed := parse_template(template);

    if parsed.error.count report(loc, method, template, parsed.error);
}

describe_names :: (names: [] string) -> string {
    if names.count == 0 return "the route has no parameters";

    builder: String_Builder;

    append(*builder, "the route has ");

    for names {
        if it_index > 0 append(*builder, ", ");

        print(*builder, "`{%}`", it);
    }

    return builder_to_string(*builder,, temp);
}

// The position of the `{name}` among the route's parameters.
param_position :: (template: string, name: string) -> int {
    parsed := parse_template(template);
    position := 0;

    for parsed.segments {
        if it.kind == .STATIC continue;

        if it.text == name return position;

        position += 1;
    }

    return -1;
}

check_param :: (template: string, method: Method, loc: Source_Code_Location, extractor: string, name: string, rest: bool) {
    parsed := parse_template(template);

    if parsed.error.count return;

    names := template_names(parsed);

    for parsed.segments {
        if it.kind == .STATIC || it.text != name continue;

        if rest && it.kind != .REST {
            report(loc, method, template, tprint("the handler asks for %(\"%\"), which needs the rest of the path, but `{%}` takes one segment; write `{%...}` in the route", extractor, name, name, name));
        }

        return;
    }

    if extractor == "Params" {
        report(loc, method, template, tprint("the Params struct has a field `%`, but the route has no `{%}` parameter (%)", name, name, describe_names(names)));

        return;
    }

    report(loc, method, template, tprint("the handler asks for %(\"%\", ...), but the route has no `{%}` parameter (%)", extractor, name, name, describe_names(names)));
}

// A parameter type that text can be parsed into.
check_scalar :: (info: *Type_Info, template: string, method: Method, loc: Source_Code_Location, what: string) {
    ok := info.type == .INTEGER || info.type == .FLOAT || info.type == .BOOL || info.type == .STRING || info.type == .ENUM;

    if !ok {
        report(loc, method, template, tprint("%: that type cannot be parsed from text; use an integer, float, bool, string or enum", what));
    }
}

report_parameter :: (loc: Source_Code_Location, method: Method, template: string, index: int, type_name: string) {
    report(loc, method, template, tprint("handler parameter % has type `%`, which is not an extractor. Use Path(\"name\", T), Params(T), Rest(\"name\"), Query(T), Json(T), Form(T), Header(\"name\", T), Header_Optional(\"name\", T), Bytes, Text, *Request or *State(T)", index + 1, type_name));
}

report_bodies :: (loc: Source_Code_Location, method: Method, template: string) {
    report(loc, method, template, "the handler has more than one body extractor (Json, Form, Bytes, Text), but a request has one body");
}

report_returns :: (loc: Source_Code_Location, method: Method, template: string, message: string) {
    report(loc, method, template, message);
}

// The checks and constants a route needs, as source text for `#insert` in `add_route`.
generate_prelude :: (H: Type) -> string {
    info := cast(*Type_Info_Procedure) H;
    out: String_Builder;

    print(*out, "#run check_template(FULL, METHOD, LOC);\n");

    bodies := 0;

    for argument, index: info.argument_types {
        kind := classify(argument);

        print(*out, "A% :: #run arg_type(H, %);\n", index, index);

        if kind == {
            case .PATH;
                print(*out, "#run check_param(FULL, METHOD, LOC, \"Path\", A%.NAME, false);\n", index);
                print(*out, "#run check_scalar(type_info(A%.T), FULL, METHOD, LOC, tprint(\"Path(\\\"%\\\", ...)\", A%.NAME));\n", index, "%", index);
                print(*out, "PI% :: #run param_position(FULL, A%.NAME);\n", index, index);
            case .HEADER;
                print(*out, "#run check_scalar(type_info(A%.T), FULL, METHOD, LOC, tprint(\"Header(\\\"%\\\", ...)\", A%.NAME));\n", index, "%", index);
            case .HEADER_OPTIONAL;
                print(*out, "#run check_scalar(type_info(A%.T), FULL, METHOD, LOC, tprint(\"Header_Optional(\\\"%\\\", ...)\", A%.NAME));\n", index, "%", index);
            case .REST;
                print(*out, "#run check_param(FULL, METHOD, LOC, \"Rest\", A%.NAME, true);\n", index);
                print(*out, "PI% :: #run param_position(FULL, A%.NAME);\n", index, index);
            case .PARAMS;
                fields := value_info(argument);

                if fields && fields.type == .STRUCT {
                    for (cast(*Type_Info_Struct) fields).members {
                        if it.flags & .CONSTANT continue;

                        print(*out, "#run check_param(FULL, METHOD, LOC, \"Params\", \"%\", false);\n", it.name);
                    }
                }
            case .UNKNOWN;
                print(*out, "#run report_parameter(LOC, METHOD, FULL, %, \"%\");\n", index, tprint("%", cast(Type) argument));
        }

        if is_body_kind(kind) bodies += 1;
    }

    if bodies > 1 print(*out, "#run report_bodies(LOC, METHOD, FULL);\n");

    count := info.return_types.count;

    if count > 2 {
        print(*out, "#run report_returns(LOC, METHOD, FULL, \"the handler returns % values; return a value and an optional Http_Error\");\n", count);
    } else if count == 2 && info.return_types[1] != type_info(Http_Error) {
        print(*out, "#run report_returns(LOC, METHOD, FULL, \"the second value a handler returns must be an Http_Error\");\n");
    }

    return builder_to_string(*out,, temp);
}

// The body of a route's thunk: fills in the arguments, calls the handler and turns the result
// into a response.
generate_call :: (H: Type) -> string {
    info := cast(*Type_Info_Procedure) H;
    out: String_Builder;
    arguments: String_Builder;

    for argument, index: info.argument_types {
        kind := classify(argument);

        if index > 0 append(*arguments, ", ");

        if kind == {
            case .PATH;
                print(*out, "a%: A%;\n{ value, error := extract_path(request, PI%, A%.NAME, A%.T); if error.kind != .NONE return error_response(request, error); a%.value = value; }\n", index, index, index, index, index, index);
                print(*arguments, "a%", index);
            case .PARAMS;
                print(*out, "a%: A%;\n{ value, error := extract_params(request, A%.T); if error.kind != .NONE return error_response(request, error); a%.value = value; }\n", index, index, index, index);
                print(*arguments, "a%", index);
            case .REST;
                print(*out, "a%: A%;\na%.value = request.params[PI%].value;\n", index, index, index, index);
                print(*arguments, "a%", index);
            case .QUERY;
                print(*out, "a%: A%;\n{ value, error := extract_query(request, A%.T); if error.kind != .NONE return error_response(request, error); a%.value = value; }\n", index, index, index, index);
                print(*arguments, "a%", index);
            case .JSON;
                print(*out, "a%: A%;\n{ value, error := extract_json(request, A%.T); if error.kind != .NONE return error_response(request, error); a%.value = value; }\n", index, index, index, index);
                print(*arguments, "a%", index);
            case .FORM;
                print(*out, "a%: A%;\n{ value, error := extract_form(request, A%.T); if error.kind != .NONE return error_response(request, error); a%.value = value; }\n", index, index, index, index);
                print(*arguments, "a%", index);
            case .HEADER;
                print(*out, "a%: A%;\n{ value, error := extract_header(request, A%.NAME, A%.T); if error.kind != .NONE return error_response(request, error); a%.value = value; }\n", index, index, index, index, index);
                print(*arguments, "a%", index);
            case .HEADER_OPTIONAL;
                print(*out, "a%: A%;\n{ value, present, error := extract_header_optional(request, A%.NAME, A%.T); if error.kind != .NONE return error_response(request, error); a%.value = value; a%.present = present; }\n", index, index, index, index, index, index);
                print(*arguments, "a%", index);
            case .BYTES;
                print(*out, "a%: A%;\na%.value = request.body;\n", index, index, index);
                print(*arguments, "a%", index);
            case .TEXT;
                print(*out, "a%: A%;\na%.value = body_text(request);\n", index, index, index);
                print(*arguments, "a%", index);
            case .REQUEST;
                print(*arguments, "request");
            case .STATE;
                print(*out, "if !request.app.state return error_response(request, internal_error(\"this route takes a State, but app.state was never set\"));\n");
                print(*out, "s%: A% = cast(A%) request.app.state;\n", index, index, index);
                print(*arguments, "s%", index);
            case;
                // Reported by the prelude; keep the generated code well formed.
                print(*arguments, "{}");
        }
    }

    call := builder_to_string(*arguments,, temp);

    count := info.return_types.count;

    if count == 0 {
        print(*out, "handler(%);\nreturn no_content();\n", call);
    } else if count == 1 {
        print(*out, "r0 := handler(%);\nreturn respond(request, r0);\n", call);
    } else {
        print(*out, "r0, r1 := handler(%);\nif r1.kind != .NONE return error_response(request, r1);\nreturn respond(request, r0);\n", call);
    }

    return builder_to_string(*out,, temp);
}
