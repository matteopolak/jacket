// Shared by the Http tests: a driver that feeds text to a parser in pieces, and the text form of
// what the parser reported, so two runs can be compared as strings. The form is the one
// tools/gen_http_corpus.py writes for llhttp's fixtures:
//
//     M<SOH>method<SOH>target<SOH>1.1    a request head       S<SOH>status<SOH>reason<SOH>1.1  a response head
//     H<SOH>name<SOH>value               a header             D   the head is complete
//     B<SOH>hex                          body bytes (runs merged)
//     T<SOH>name<SOH>value               a trailer            C   the message is complete
//     E                                  the parser failed    U   the stream switched protocols

#import "Basic";
#import "String";
#import "Extensions/Http";

failures := 0;

check :: (condition: bool, what: string) {
    if condition return;

    print("FAIL: %\n", what);
    failures += 1;
}

NO_HEADERS: [] Header;

as_string :: (data: *u8, count: int) -> string {
    result: string;
    result.data = data;
    result.count = count;

    return result;
}

text_of :: (bytes: [..] u8) -> string {
    return as_string(bytes.data, bytes.count);
}

as_bytes :: (text: string) -> [] u8 {
    result: [] u8;
    result.count = text.count;
    result.data = text.data;

    return result;
}

append_hex :: (builder: *String_Builder, data: [] u8) {
    digits := "0123456789abcdef";

    for data {
        append(builder, digits[it >> 4]);
        append(builder, digits[it & 15]);
    }
}

// How the input is cut into pieces for the parser.
Cut :: struct {
    split: int; // 0: one piece; n > 0: a piece of n bytes, then the rest; -1: one byte at a time
}

Run :: struct {
    kind := Parser_Kind.REQUEST;
    response_to := Method.GET;
    limits: Limits;
    eof := false; // call parse_eof after the input
}

default_run :: (kind: Parser_Kind) -> Run {
    run: Run;
    run.kind = kind;
    run.limits.require_host = false;

    return run;
}

// Feeds `input` as `cut` says and returns what the parser reported, in the text form above. The
// parser's error, when it failed, is left in `error`.
observe :: (run: Run, input: string, cut: Cut, error: *Http_Error = null) -> string {
    positions: [..] int;

    defer array_reset(*positions);

    if cut.split > 0 && cut.split < input.count array_add(*positions, cut.split);

    if cut.split < 0 {
        for 1..input.count - 1 array_add(*positions, it);
    }

    return observe_at(run, input, positions, error);
}

// Feeds `input` cut at the given ascending positions (pieces may be empty if positions repeat).
observe_at :: (run: Run, input: string, positions: [] int, error: *Http_Error = null) -> string {
    parser: Parser;

    parser_init(*parser, run.kind, run.limits);
    defer parser_free(*parser);

    parser.response_to = run.response_to;

    out: String_Builder;
    body: String_Builder;
    have_body := false;
    stopped := false;

    all := as_bytes(input);
    start := 0;
    index := 0;

    while index <= positions.count && !stopped {
        stop := input.count;
        if index < positions.count stop = positions[index];

        data: [] u8;
        data.count = stop - start;
        data.data = all.data + start;
        start = stop;
        index += 1;

        while !stopped {
            used, event := parse(*parser, data);

            data = advance(data, used);

            if event.kind == .BODY {
                append_hex(*body, event.data);
                have_body = true;

                continue;
            }

            if event.kind == .NEED_MORE break;

            if have_body {
                append(*out, "B\x01");
                append(*out, builder_to_string(*body));
                append(*out, "\n");
                reset(*body);
                have_body = false;
            }

            if event.kind == {
                case .HEAD;
                    describe_head(*out, *parser);

                case .END;
                    for parser.trailers print_to_builder(*out, "T\x01%\x01%\n", it.name, it.value);

                    append(*out, "C\n");

                case .SWITCHED;
                    append(*out, "U\n");

                    stopped = true;

                case .ERROR;
                    append(*out, "E\n");

                    stopped = true;
            }
        }
    }

    if have_body {
        append(*out, "B\x01");
        append(*out, builder_to_string(*body));
        append(*out, "\n");
    }

    if run.eof && !stopped {
        event := parse_eof(*parser);

        if event.kind == .END append(*out, "C\n");
        if event.kind == .ERROR append(*out, "E\n");
    }

    if error error.* = parser.error;

    return builder_to_string(*out);
}

describe_head :: (out: *String_Builder, parser: *Parser) {
    head := parser.head;
    version := "1.1";

    if head.version == .HTTP_1_0 version = "1.0";

    if parser.kind == .REQUEST {
        print_to_builder(out, "M\x01%\x01%\x01%\n", head.method_text, head.target, version);
    } else {
        print_to_builder(out, "S\x01%\x01%\x01%\n", head.status, head.reason, version);
    }

    for head.headers print_to_builder(out, "H\x01%\x01%\n", it.name, it.value);

    append(out, "D\n");
}

// Runs `input` at once, one byte at a time and split at every position, and checks that all give
// the same text. Returns that text.
observe_every_cut :: (run: Run, input: string, name: string) -> string {
    whole := observe(run, input, .{});

    check(observe(run, input, .{ -1 }) == whole, tprint("% differs byte by byte", name));

    for 1..input.count - 1 {
        got := observe(run, input, .{ it });

        if got != whole {
            check(false, tprint("% differs when split at %", name, it));

            break;
        }
    }

    return whole;
}

// A parser with the events it has produced so far, for tests that look at one message closely.
Driver :: struct {
    parser: Parser;
    body: [..] u8;
    heads: int;
    ends: int;
    last: Event_Kind;
}

make_driver :: (kind: Parser_Kind, limits := Limits.{}) -> *Driver {
    d := New(Driver);
    parser_init(*d.parser, kind, limits);

    return d;
}

free_driver :: (d: *Driver) {
    parser_free(*d.parser);
    array_reset(*d.body);
    free(d);
}

// Feeds `input` and runs until the parser wants more, fails or switches. Body bytes pile up in
// d.body; d.last is the final event.
drive :: (d: *Driver, input: string) {
    data := as_bytes(input);

    while true {
        used, event := parse(*d.parser, data);

        data = advance(data, used);
        d.last = event.kind;

        if event.kind == {
            case .HEAD; d.heads += 1;
            case .BODY; for event.data array_add(*d.body, it);
            case .END; d.ends += 1;
            case .NEED_MORE; return;
            case .ERROR; return;
            case .SWITCHED; return;
        }
    }
}

body_text :: (d: *Driver) -> string {
    result: string;
    result.data = d.body.data;
    result.count = d.body.count;

    return result;
}

// The error kind a whole input ends with (NONE when it parses to the end of the input), and
// checks that feeding it a byte at a time gives the same kind.
error_kind_of :: (kind: Parser_Kind, input: string, limits := Limits.{}) -> Error_Kind {
    d := make_driver(kind, limits);

    defer free_driver(d);

    drive(d, input);

    result := d.parser.error.kind;

    e := make_driver(kind, limits);

    defer free_driver(e);

    for 0..input.count - 1 {
        drive(e, slice(input, it, 1));

        if e.last == .ERROR break;
    }

    check(e.parser.error.kind == result, tprint("error kind differs byte by byte: % vs % for %", e.parser.error.kind, result, input));

    return result;
}

finish :: () {
    if failures == 0 {
        print("ok\n");

        return;
    }

    print("% failure(s)\n", failures);
    exit(1);
}
