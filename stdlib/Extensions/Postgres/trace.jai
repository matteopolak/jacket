// A message trace: every protocol message the connection sends or receives, one line each, in the
// text layout of libpq's PQtrace. Off by default.

// Called with each traced line, without its newline. The line is in temporary storage.
Trace_Proc :: #type (connection: *Connection, line: string);

// Starts tracing: `proc` gets a line per message from now on (null stops). With `regress`, the
// output is libpq's PQTRACE_SUPPRESS_TIMESTAMPS | PQTRACE_REGRESS_MODE: OIDs, process ids and the
// server's source locations are replaced by fixed text, and an error's length by NN, so a trace
// is the same from one server run to the next. Without it they show as they are.
//
// ```text
// F<TAB>21<TAB>Parse<TAB> "" "SELECT $1" 1 NNNN
// B<TAB>5<TAB>ReadyForQuery<TAB> I
// ```
//
// F is a message the client sent, B one it received. Messages sent while connecting are not
// traced, as the trace starts after `connect`.
set_trace :: (c: *Connection, proc: Trace_Proc, data: *void = null, regress := true) {
    c.on_trace = proc;
    c.trace_data = data;
    c.trace_regress = regress;
}

#scope_module

// Formats one message and hands it to the connection's hook. `body` is without the type byte and
// the length.
trace_message :: (c: *Connection, to_server: bool, type: u8, body: [] u8) {
    t: Tracer;
    t.builder.allocator = temp;
    t.regress = c.trace_regress;
    t.r = Cursor.{ data = body };

    append(*t.builder, ifx to_server then "F\t" else "B\t");

    if t.regress && !to_server && (type == #char "E" || type == #char "N") {
        append(*t.builder, "NN\t");
    } else {
        print_to_builder(*t.builder, "%\t", body.count + 4);
    }

    if to_server trace_frontend(*t, type);
    else trace_backend(*t, type);

    c.on_trace(c, builder_to_string(*t.builder,, temp));
}

Tracer :: struct {
    builder: String_Builder;
    r: Cursor;
    regress: bool;
}

trace_name :: (t: *Tracer, name: string, fields := true) {
    append(*t.builder, name);
    if fields append(*t.builder, "\t");
}

trace_frontend :: (t: *Tracer, type: u8) {
    if type == {
        case #char "Q";
            trace_name(t, "Query");
            trace_string(t);
        case #char "P";
            trace_name(t, "Parse");
            trace_string(t);
            trace_string(t);
            count := trace_int16(t);
            for 1..count trace_oid(t);
        case #char "B";
            trace_bind(t);
        case #char "D";
            trace_name(t, "Describe");
            trace_byte(t);
            trace_string(t);
        case #char "C";
            trace_name(t, "Close");
            trace_byte(t);
            trace_string(t);
        case #char "E";
            trace_name(t, "Execute");
            trace_string(t);
            trace_int32(t);
        case #char "S";
            trace_name(t, "Sync", false);
        case #char "H";
            trace_name(t, "Flush", false);
        case #char "X";
            trace_name(t, "Terminate", false);
        case #char "d";
            trace_name(t, "CopyData");
            trace_rest(t);
        case #char "c";
            trace_name(t, "CopyDone", false);
        case #char "f";
            trace_name(t, "CopyFail");
            trace_string(t);
        case #char "p";
            trace_name(t, "PasswordMessage", false);
        case;
            trace_name(t, "Unknown", false);
    }
}

trace_backend :: (t: *Tracer, type: u8) {
    if type == {
        case #char "1"; trace_name(t, "ParseComplete", false);
        case #char "2"; trace_name(t, "BindComplete", false);
        case #char "3"; trace_name(t, "CloseComplete", false);
        case #char "n"; trace_name(t, "NoData", false);
        case #char "s"; trace_name(t, "PortalSuspended", false);
        case #char "I"; trace_name(t, "EmptyQueryResponse", false);
        case #char "c"; trace_name(t, "CopyDone", false);
        case #char "R";
            trace_name(t, "Authentication");
            trace_int32(t);
        case #char "K";
            trace_name(t, "BackendKeyData");
            trace_number(t);

            // Protocol 3.2 keys are longer than four bytes.
            if t.r.data.count - t.r.at == 4 trace_number(t);
            else trace_rest(t);
        case #char "A";
            trace_name(t, "NotificationResponse");
            trace_number(t);
            trace_string(t);
            trace_string(t);
        case #char "C";
            trace_name(t, "CommandComplete");
            trace_string(t);
        case #char "D";
            trace_data_row(t);
        case #char "E";
            trace_name(t, "ErrorResponse");
            trace_fields(t);
        case #char "N";
            trace_name(t, "NoticeResponse");
            trace_fields(t);
        case #char "G";
            trace_name(t, "CopyInResponse");
            trace_copy_response(t);
        case #char "H";
            trace_name(t, "CopyOutResponse");
            trace_copy_response(t);
        case #char "W";
            trace_name(t, "CopyBothResponse");
            trace_copy_response(t);
        case #char "d";
            trace_name(t, "CopyData");
            trace_rest(t);
        case #char "S";
            trace_name(t, "ParameterStatus");
            trace_string(t);
            trace_string(t);
        case #char "t";
            trace_name(t, "ParameterDescription");
            count := trace_int16(t);
            for 1..count trace_oid(t);
        case #char "T";
            trace_row_description(t);
        case #char "Z";
            trace_name(t, "ReadyForQuery");
            trace_byte(t);
        case #char "v";
            trace_name(t, "NegotiateProtocolVersion");
            trace_int32(t);
            count := trace_int32(t);
            for 1..count trace_string(t);
        case;
            trace_name(t, "Unknown", false);
    }
}

trace_bind :: (t: *Tracer) {
    trace_name(t, "Bind");
    trace_string(t);
    trace_string(t);

    formats := trace_int16(t);
    for 1..formats trace_int16(t);

    params := trace_int16(t);
    for 1..params {
        length := trace_int32(t);
        if length >= 0 trace_chars(t, length);
    }

    results := trace_int16(t);
    for 1..results trace_int16(t);
}

trace_data_row :: (t: *Tracer) {
    trace_name(t, "DataRow");

    count := trace_int16(t);
    for 1..count {
        length := trace_int32(t);
        if length >= 0 trace_chars(t, length);
    }
}

trace_row_description :: (t: *Tracer) {
    trace_name(t, "RowDescription");

    count := trace_int16(t);
    for 1..count {
        trace_string(t);
        trace_oid(t); // the table
        trace_int16(t);
        trace_oid(t); // the type
        trace_int16(t);
        trace_int32(t);
        trace_int16(t);
    }
}

trace_copy_response :: (t: *Tracer) {
    trace_int8(t);

    count := trace_int16(t);
    for 1..count trace_int16(t);
}

// The fields of an ErrorResponse or NoticeResponse: a type byte and a string each, ending in a
// zero byte.
trace_fields :: (t: *Tracer) {
    while !t.r.bad && t.r.at < t.r.data.count {
        field := t.r.data[t.r.at];

        trace_byte(t);
        if field == 0 break;

        // The server's source file, line and function change with every build.
        if t.regress && (field == #char "F" || field == #char "L" || field == #char "R") {
            get_cstring(*t.r);
            append(*t.builder, " \"SSSS\"");
        } else {
            trace_string(t);
        }
    }
}

trace_string :: (t: *Tracer) {
    text := get_cstring(*t.r);
    print_to_builder(*t.builder, " \"%\"", text);
}

trace_byte :: (t: *Tracer) {
    value := get_u8(*t.r);

    if value >= 32 && value < 127 {
        print_to_builder(*t.builder, " %", string.{ 1, *value });
    } else {
        trace_hex(t, value, " \\x");
    }
}

trace_hex :: (t: *Tracer, value: u8, prefix: string) {
    DIGITS :: "0123456789abcdef";

    append(*t.builder, prefix);
    append(*t.builder, slice(DIGITS, value >> 4, 1));
    append(*t.builder, slice(DIGITS, value & 15, 1));
}

trace_int8 :: (t: *Tracer) {
    print_to_builder(*t.builder, " %", get_u8(*t.r));
}

trace_int16 :: (t: *Tracer) -> s64 {
    value := get_u16(*t.r); // unsigned, as libpq prints it
    print_to_builder(*t.builder, " %", value);

    return value;
}

trace_int32 :: (t: *Tracer) -> s64 {
    value := get_s32(*t.r);
    print_to_builder(*t.builder, " %", value);

    return value;
}

// An OID, a process id or a key: fixed text in regress mode.
trace_oid :: (t: *Tracer) {
    value := get_u32(*t.r);

    if t.regress append(*t.builder, " NNNN");
    else print_to_builder(*t.builder, " %", value);
}

trace_number :: (t: *Tracer) {
    trace_oid(t);
}

// `length` bytes in quotes; bytes that do not print are shown as \xNN.
trace_chars :: (t: *Tracer, length: s64) {
    bytes := get_bytes(*t.r, length);

    append(*t.builder, " '");

    for bytes {
        if it >= 32 && it < 127 {
            append(*t.builder, string.{ 1, *it });
        } else {
            trace_hex(t, it, "\\x");
        }
    }

    append(*t.builder, "'");
}

trace_rest :: (t: *Tracer) {
    trace_chars(t, t.r.data.count - t.r.at);
}
