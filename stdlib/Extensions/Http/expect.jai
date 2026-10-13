// Assertions for responses and a small test runner. A failed expectation prints where it
// happened, what was expected and what came back (the request, the status, the headers that
// matter and the start of the body), counts as a failure and lets the test go on; `run_tests`
// ends the program with a summary and a non-zero exit code when anything failed.
//
//     main :: () {
//         run_tests(.[
//             .{ "creates a note", creates_a_note },
//             .{ "rejects an empty title", rejects_an_empty_title },
//         ]);
//     }
//
//     creates_a_note :: () {
//         api := test_client(*app);
//
//         note := expect_json(post(api, "/notes", New_Note.{ title = "milk" }), Note, status = 201);
//
//         expect_equal(note.title, "milk");
//         expect_status(get(api, "/notes/1"), 200);
//     }
//
// A response's body is read from the connection the first time an expectation needs it (to
// check it, or to show it in a failure) and kept, so any number of expectations can look at it
// and `read_text` and friends still get it afterwards.
//
// Every `expect_*` returns whether it held (or the value it decoded, empty when it did not), so
// a test can stop early with `if !expect_status(...) return;` when the rest depends on it.
// `loc` is the caller's location; leave it out.

// The RFC 9457 document the server sends for errors, as `expect_problem` decodes it.
Problem :: struct @json(default) {
    type: string;
    title: string;
    status: int;
    detail: string;
    instance: string;
    location: string; // for bad input: `body`, `query`, `path`, `header` or `form`
    field: string; // for bad input: the parameter, header or JSON path
}

// A condition the test relies on, for anything that is not a response: the state a handler left
// behind, a count. `what` says what the condition means.
expect :: (condition: bool, what := "", loc := #caller_location) -> bool {
    test_state.checks += 1;

    if condition return true;

    record_failure(loc, ifx what.count then tprint("expected: %", what) else "the condition is false");

    return false;
}

// `actual == expected`, showing both when they differ.
expect_equal :: (actual: $T, expected: T, what := "", loc := #caller_location) -> bool {
    test_state.checks += 1;

    if actual == expected return true;

    label := ifx what.count then tprint("%: ", what) else "";

    #if T == string {
        record_failure(loc, tprint("%expected \"%\", got \"%\"", label, expected, actual));
    } else {
        record_failure(loc, tprint("%expected %, got %", label, expected, actual));
    }

    return false;
}

// The status of the response is exactly `status`.
expect_status :: (response: Client_Response, status: int, loc := #caller_location) -> bool {
    test_state.checks += 1;

    if response.status == status return true;

    fail_response(loc, tprint("expected status %, got %", status_label(status), status_label(response.status)), response);

    return false;
}

// The status is 2xx.
expect_success :: (response: Client_Response, loc := #caller_location) -> bool {
    test_state.checks += 1;

    if response.status >= 200 && response.status < 300 return true;

    fail_response(loc, tprint("expected a 2xx status, got %", status_label(response.status)), response);

    return false;
}

// The response has the header, with exactly this value (any case in the name). For
// `Content-Type` a value without parameters matches the media type: `"application/json"` accepts
// `application/json; charset=utf-8`.
expect_header :: (response: Client_Response, name: string, value: string, loc := #caller_location) -> bool {
    test_state.checks += 1;

    actual, found := find_header(response.headers, name);

    if found && header_matches(name, actual, value) return true;

    if found {
        fail_response(loc, tprint("expected header `%: %`, got `%: %`", name, value, name, actual), response, name);
    } else {
        fail_response(loc, tprint("expected header `%: %`, but the response has no `%` header", name, value, name), response, name);
    }

    return false;
}

// The response has the header, whatever its value.
expect_header :: (response: Client_Response, name: string, loc := #caller_location) -> bool {
    test_state.checks += 1;

    _, found := find_header(response.headers, name);

    if found return true;

    fail_response(loc, tprint("expected a `%` header, but the response has none", name), response);

    return false;
}

expect_no_header :: (response: Client_Response, name: string, loc := #caller_location) -> bool {
    test_state.checks += 1;

    actual, found := find_header(response.headers, name);

    if !found return true;

    fail_response(loc, tprint("expected no `%` header, got `%: %`", name, name, actual), response, name);

    return false;
}

// The body is exactly `expected`.
expect_text :: (response: Client_Response, expected: string, loc := #caller_location) -> bool {
    test_state.checks += 1;

    body, problem := checked_body(response);

    if problem.count == 0 && body == expected return true;

    fail_response(loc, tprint("expected the body \"%\"", expected), response);

    return false;
}

// The body contains `needle`.
expect_body_contains :: (response: Client_Response, needle: string, loc := #caller_location) -> bool {
    test_state.checks += 1;

    body, problem := checked_body(response);

    if problem.count == 0 && find_index_from_left(body, needle) >= 0 return true;

    fail_response(loc, tprint("expected the body to contain \"%\"", needle), response);

    return false;
}

// The response redirects (3xx, or exactly `status` when given) to `location`.
expect_redirect :: (response: Client_Response, location: string, status := 0, loc := #caller_location) -> bool {
    test_state.checks += 1;

    actual, found := find_header(response.headers, "Location");

    status_ok := ifx status > 0 then response.status == status else (response.status >= 300 && response.status < 400);

    if status_ok && found && actual == location return true;

    if !status_ok {
        fail_response(loc, tprint("expected %, got %", ifx status > 0 then status_label(status) else "a 3xx redirect", status_label(response.status)), response, "Location");
    } else if !found {
        fail_response(loc, tprint("expected a redirect to `%`, but there is no `Location` header", location), response);
    } else {
        fail_response(loc, tprint("expected a redirect to `%`, got `%`", location, actual), response, "Location");
    }

    return false;
}

// The body decoded as JSON into a `T`. The status must be `status`, or 2xx when it is left out,
// and the `Content-Type` JSON. On failure the value is empty. The strings in it are allocated
// with the context allocator; a test may leave them.
expect_json :: (response: Client_Response, $T: Type, status := 0, loc := #caller_location) -> T {
    test_state.checks += 1;

    result: T;

    if status > 0 {
        if response.status != status {
            fail_response(loc, tprint("expected status %, got %", status_label(status), status_label(response.status)), response);

            return result;
        }
    } else if response.status < 200 || response.status >= 300 {
        fail_response(loc, tprint("expected a 2xx status, got %", status_label(response.status)), response);

        return result;
    }

    content_type, found := find_header(response.headers, "Content-Type");

    if !found || !(media_type_is(content_type, "application/json") || ends_with(media_type_of(content_type), "+json")) {
        fail_response(loc, tprint("expected a JSON body, but the Content-Type is %", ifx found then tprint("`%`", content_type) else "missing"), response, "Content-Type");

        return result;
    }

    body, problem := checked_body(response);

    if problem.count {
        fail_response(loc, tprint("the body cannot be read: %", problem), response);

        return result;
    }

    value, failure := from_json(T, body);

    if failure.kind != .NONE {
        fail_response(loc, tprint("the body is not the JSON expected (a %): %", T, json_error_text(failure)), response);

        free_json(*failure);

        return result;
    }

    return value;
}

// The response is an RFC 9457 problem document with this status and, when `detail` is given, a
// `detail` that contains it. Returns the decoded document (empty when the check failed).
expect_problem :: (response: Client_Response, status: int, detail := "", loc := #caller_location) -> Problem {
    test_state.checks += 1;

    result: Problem;

    if response.status != status {
        fail_response(loc, tprint("expected a problem with status %, got %", status_label(status), status_label(response.status)), response);

        return result;
    }

    content_type, found := find_header(response.headers, "Content-Type");

    if !found || !media_type_is(content_type, "application/problem+json") {
        fail_response(loc, tprint("expected an application/problem+json body, but the Content-Type is %", ifx found then tprint("`%`", content_type) else "missing"), response, "Content-Type");

        return result;
    }

    body, unreadable := checked_body(response);

    if unreadable.count {
        fail_response(loc, tprint("the body cannot be read: %", unreadable), response);

        return result;
    }

    problem, failure := from_json(Problem, body);

    if failure.kind != .NONE {
        fail_response(loc, tprint("the body is not a problem document: %", json_error_text(failure)), response);

        free_json(*failure);

        return result;
    }

    if problem.status != status {
        fail_response(loc, tprint("the problem document says status %, but the response has %", problem.status, status), response);

        return result;
    }

    if detail.count && find_index_from_left(problem.detail, detail) < 0 {
        fail_response(loc, tprint("expected the problem detail to contain \"%\", got \"%\"", detail, problem.detail), response);

        return result;
    }

    return problem;
}

// ---------------------------------------------------------------------------------------------
// Running tests

Test_Proc :: #type ();

Test_Case :: struct {
    name: string;
    run: Test_Proc;
}

// Runs the tests one after another on the calling thread, prints a line per test (and the
// details of what failed), then a summary. Exits with status 1 when any expectation failed.
// Clients made by `test_client` are released after each test.
run_tests :: (cases: [] Test_Case) {
    started := to_milliseconds(current_time_monotonic());

    for cases {
        test_state.name = it.name;
        test_state.announced = false;
        test_state.tests += 1;

        before := test_state.failures;
        checks_before := test_state.checks;

        it.run();

        free_test_clients();
        reset_temporary_storage();

        if test_state.failures > before {
            test_state.failed_tests += 1;
        } else {
            print("  ok    % (% check%)\n", it.name, test_state.checks - checks_before, ifx test_state.checks - checks_before == 1 then "" else "s");
        }
    }

    test_state.name = "";

    elapsed := to_milliseconds(current_time_monotonic()) - started;

    if test_state.failed_tests {
        print("\n% of % tests failed (% failed checks of %, % ms)\n", test_state.failed_tests, test_state.tests, test_state.failures, test_state.checks, elapsed);

        exit(1);
    }

    print("\n% test%, % check%, all passed (% ms)\n", test_state.tests, ifx test_state.tests == 1 then "" else "s", test_state.checks, ifx test_state.checks == 1 then "" else "s", elapsed);
}

// Collects the failure text that would be printed instead of printing it, and does not count the
// failures against the run: for tests of the test tools themselves. `end_failure_capture`
// returns the text and how many failures there were.
begin_failure_capture :: () {
    test_state.capture = New(String_Builder);
    test_state.captured = 0;
}

end_failure_capture :: () -> string, int {
    text := builder_to_string(test_state.capture,, temp);
    count := test_state.captured;

    free(test_state.capture);

    test_state.capture = null;
    test_state.captured = 0;

    return text, count;
}

// Totals so far, for programs that run their own loop instead of `run_tests`.
test_failure_count :: () -> int {
    return test_state.failures;
}

#scope_module

Test_State :: struct {
    tests: int;
    failed_tests: int;
    checks: int;
    failures: int;

    name: string;
    announced: bool;

    capture: *String_Builder;
    captured: int;
}

test_state: Test_State;

// The body of `response` for a check, without using it up, and why it is not available when it
// is not. Copies of a response share the body, so reading through this one is seen by all.
checked_body :: (response: Client_Response) -> string, string {
    local := response;

    return peek_body(*local);
}

status_label :: (status: int) -> string {
    if status == 0 return "no response";

    return tprint("% %", status, status_reason(status));
}

header_matches :: (name: string, actual: string, expected: string) -> bool {
    if actual == expected return true;

    if equal_nocase(name, "Content-Type") && find_index_from_left(expected, #char ";") < 0 {
        return media_type_is(actual, expected);
    }

    return false;
}

// A file name relative to the working directory when it is below it.
shown_path :: (file: string) -> string {
    directory := get_working_directory();

    if directory.count == 0 || !begins_with(file, directory) || file.count <= directory.count + 1 return file;

    separator := file[directory.count];

    if separator != #char "/" && separator != #char "\\" return file;

    return slice(file, directory.count + 1, file.count - directory.count - 1);
}

record_failure :: (loc: Source_Code_Location, text: string) {
    builder: String_Builder;

    if test_state.capture == null {
        test_state.failures += 1;

        if test_state.name.count && !test_state.announced {
            print(*builder, "FAIL  %\n", test_state.name);

            test_state.announced = true;
        }
    } else {
        test_state.captured += 1;
    }

    print(*builder, "  %:%\n", shown_path(loc.fully_pathed_filename), loc.line_number);

    line_start := true;

    for text {
        if line_start append(*builder, "    ");

        append(*builder, string.{ 1, *it });

        line_start = it == #char "\n";
    }

    append(*builder, "\n");

    output := builder_to_string(*builder,, temp);

    if test_state.capture {
        append(test_state.capture, output);
    } else {
        print("%", output);
    }
}

// A failure that shows the response and the request behind it. `focus` names a header to show
// besides the usual ones.
fail_response :: (loc: Source_Code_Location, headline: string, response: Client_Response, focus := "") {
    builder: String_Builder;

    append(*builder, headline);
    append(*builder, "\n");

    print(*builder, "  request   % %", method_name(response.method), response.url);

    if response.request_body_size > 0 {
        append(*builder, "  ");
        append_shown(*builder, response.request_body, response.request_body_size);
    }

    append(*builder, "\n");

    if response.status == 0 {
        print(*builder, "  response  none: %", response.reason);

        record_failure(loc, builder_to_string(*builder,, temp));

        return;
    }

    print(*builder, "  response  %", status_label(response.status));

    for response.headers {
        if shown_header(it.name, focus) print(*builder, "\n            %: %", it.name, it.value);
    }

    append(*builder, "\n            ");

    body, problem := checked_body(response);

    if problem.count && response.source && response.source.sample.count {
        // The test read it already: the start was kept.
        append_shown(*builder, string.{ response.source.sample.count, response.source.sample.data }, response.source.sample_total);
    } else if problem.count {
        print(*builder, "(the body is not available: %)", problem);
    } else if body.count == 0 {
        append(*builder, "(no body)");
    } else {
        append_shown(*builder, body, body.count);
    }

    record_failure(loc, builder_to_string(*builder,, temp));
}

SHOWN_BODY :: 400;

// `text`, cut at SHOWN_BODY bytes, or a note when it is not text. `size` is the full size when
// `text` is only the start.
append_shown :: (builder: *String_Builder, text: string, size: int) {
    for text {
        if it < 9 || (it > 13 && it < 32) {
            print(builder, "<% bytes of binary data>", size);

            return;
        }
    }

    shown := min(text.count, SHOWN_BODY);

    append(builder, slice(text, 0, shown));

    if size > shown print(builder, "... (% more bytes)", size - shown);
}

shown_header :: (name: string, focus: string) -> bool {
    if focus.count && equal_nocase(name, focus) return true;

    return equal_nocase(name, "Content-Type") || equal_nocase(name, "Location") || equal_nocase(name, "Allow") || equal_nocase(name, "WWW-Authenticate") || equal_nocase(name, "Retry-After") || equal_nocase(name, "ETag") || equal_nocase(name, "Set-Cookie");
}
