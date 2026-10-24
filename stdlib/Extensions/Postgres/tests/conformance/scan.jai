// Reproduces psql's input scanning: how input lines become SQL statements and backslash
// commands, how variables are substituted, and how backslash-command arguments are read.
//
// This file expects `Basic` to be imported by whoever loads it.
//
// Using it (this is the loop psql's MainLoop runs for each source of input)
//
//     scanner: Scanner;
//     scanner.lookup = variables_lookup;     // :name, :'name', :"name" and :{?name}
//     scanner.hook_data = *vars;
//     scanner.std_strings = true;            // refresh from the server's standard_conforming_strings
//     query: [..] u8;                        // the query buffer
//
//     for each line of input (without its newline; an empty line outside a quote is skipped):
//         if query.count > 0 array_add(*query, #char "\n");
//         scan_setup(*scanner, line);
//         while true {
//             scanner.active = <the current \if branch is live>;
//             result, prompt := scan(*scanner, *query);
//
//             if result == {
//                 case .SEMICOLON;
//                     // `query` is one complete statement: send it (unless the \if branch is dead),
//                     // then clear it and call scan_reset(*scanner). The rest of the line is still
//                     // unread; the next call to `scan` continues right after the semicolon.
//                 case .BACKSLASH;
//                     // If `query` still holds only the newline added above, remove it again.
//                     name := scan_slash_command(*scanner);
//                     // read arguments with scan_slash_option, run the command, then:
//                     // (see "After a backslash command" below), and finally
//                     scan_slash_command_end(*scanner);
//                     // A command that sends the query buffer clears it and calls scan_reset.
//                 case .INCOMPLETE; #through;
//                 case .EOL;
//                     break;   // the line is used up; read the next one
//             }
//         }
//         scan_finish(*scanner);
//
//     At end of input a non-empty `query` is sent as a last statement
//     (`scan_count_copy_from_stdin` says how many COPY FROM STDIN it contains).
//
// After a backslash command: psql reads any arguments the command did not consume (warning
// `\cmd: extra argument "x" ignored` for each, but only if the branch is live; `scanner.active`
// is false while reading them, so that variables are not expanded and backticks do not run), or,
// if the command failed (or does not exist), discards the
// rest of the line with `.WHOLE_LINE`. Then it calls `scan_slash_command_end`.
//
// Inactive `\if` branches: psql keeps scanning them as usual (statements still end at semicolons
// and backslash commands are still tokenised), but does not send anything, and it does not clear
// the query buffer either: dead text piles up in it. `\if` therefore records the buffer length
// and `scan_save_lex_state`; when a branch that was not taken ends (`\elif`, `\else`, `\endif`),
// the buffer is cut back to that length and `scan_restore_lex_state` is called. When the branch
// that did run ends, the record is refreshed instead. The scanner's own part is `active`: while
// it is false, variables are not looked up (every `:name` stays as written) and backticks do
// not run. Set it before every `scan` and before reading a command's arguments.
//
// Not the scanner's job, but the loop around it must do these (see psql's mainloop.c):
//   - skip an empty line unless `scan_in_quote`;
//   - strip a UTF-8 byte-order mark from the first line, and refuse input that starts with PGDMP;
//   - after a statement with COPY FROM STDIN is sent, read the data lines (up to `\.`) from the
//     input directly instead of scanning them;
//   - keep `std_strings` in step with the server's standard_conforming_strings parameter.

// What stopped a call to `scan`.
Scan_Result :: enum {
    // A semicolon ended the statement. The semicolon is in the query buffer.
    SEMICOLON;
    // A backslash command starts at the current position (the backslash is consumed). Any SQL
    // before it on the line is already in the query buffer.
    BACKSLASH;
    // The line ended inside a quote, comment or parentheses, or in a BEGIN ... END body, or
    // there is nothing in the buffer at all. More input is needed.
    INCOMPLETE;
    // The line ended outside everything with text in the buffer; the statement may continue on
    // the next line (psql only sends it now with `\set SINGLELINE on`).
    EOL;
}

// What kind of prompt would be shown for the next line (only `-q` interactive psql shows them,
// but `%R`-style output and `exit`/`quit` hints depend on it).
Prompt :: enum {
    READY;
    CONTINUE;
    COMMENT;
    SINGLE_QUOTE;
    DOUBLE_QUOTE;
    DOLLAR_QUOTE;
    PAREN;
}

// How a variable's value is quoted when it is substituted.
Quote_Style :: enum {
    PLAIN; // :name
    SQL_LITERAL; // :'name'
    SQL_IDENT; // :"name"
    SHELL_ARG; // :'name' inside backticks
}

// How `scan_slash_option` reads one argument. Which one a command uses is up to psql's
// command.c: e.g. `\echo`, `\set`, `\if`, `\i` read NORMAL arguments, `\o` and `\g` FILE_OR_PIPE,
// `\!` and `\copy` WHOLE_LINE, `\d`-family patterns SQL_ID or NORMAL, `\c` SQL_ID_KEEP_CASE.
Option_Kind :: enum {
    // Whitespace ends the argument. 'single quotes' process \n \t \b \r \f \digits \xhh \c and
    // '' escapes; "double quotes" are kept, quotes included; `backticks` run a shell command.
    NORMAL;
    // Like NORMAL, then unquoted letters are lowercased and the double quotes removed.
    SQL_ID;
    // Like SQL_ID without the lowercasing.
    SQL_ID_KEEP_CASE;
    // Like NORMAL, except an argument that starts with `|` extends to the end of the line.
    FILE_OR_PIPE;
    // Everything up to the end of the line, minus leading whitespace. No quoting, no variables.
    WHOLE_LINE;
}

Message_Kind :: enum {
    WARNING;
    ERROR;
}

// Looks a variable up for `:name` and friends. Returns its value and true, or false when there
// is no such variable. Return the raw value: the scanner quotes it as the syntax asks. The text
// is copied, so it may point into the variable store.
Variable_Lookup :: #type (name: string, data: *void) -> value: string, found: bool;

// Runs a `backtick` command and returns its output (the scanner removes one trailing newline).
// Return false if it could not be run.
Shell_Proc :: #type (command: string, data: *void) -> output: string, ok: bool;

// Called for what psql prints with pg_log_warning / pg_log_error while scanning:
// `skipping recursive expansion of variable "x"` and `unterminated quoted string`.
Report_Proc :: #type (kind: Message_Kind, text: string, data: *void);

Scanner :: struct {
    // --- Configuration: set these directly, any time before `scan` ---

    // standard_conforming_strings. Off makes plain '...' strings treat backslash as an escape.
    std_strings := true;
    // False inside a dead \if branch: variables are not expanded and backticks do not run.
    active := true;
    lookup: Variable_Lookup;
    run_shell: Shell_Proc;
    report: Report_Proc;
    hook_data: *void; // passed to all three procedures above

    // Set when a backslash argument held a `backtick` and `run_shell` was missing or failed.
    backtick_failed: bool;

    // --- State: use the procedures below ---

    sources: [..] Source;
    out: *[..] u8;
    state: Scan_State;
    state_before_string_end: Scan_State;
    paren_depth: s64;
    begin_depth: s64;
    comment_depth: s64;
    dollar_tag: string;
    idents: [8] u8;
    idents_count: s64;
    copy_stdin_count: s64;
}

// Starts scanning one input line (without its newline). The text must stay valid until
// `scan_finish`. Mid-statement state (quotes, parentheses, ...) is kept from the previous line.
scan_setup :: (s: *Scanner, line: string) {
    assert(s.sources.count == 0, "scan_setup called while a line is still being scanned");
    array_add(*s.sources, .{ text = line });
}

// Scans the rest of the line, appending the text that psql would keep to `query`. Variable
// references are replaced, and leading whitespace and `--` comments are dropped while `query`
// is empty (a `/* */` comment is kept). Returns why it stopped and the prompt for the next line.
// `query` must contain the whole statement so far, including the newlines between lines:
// whether it is empty decides what is dropped.
scan :: (s: *Scanner, query: *[..] u8) -> Scan_Result, Prompt {
    s.out = query;

    while settle(s) {
        result, stop := scan_token(s);
        if stop return result, .READY;
    }

    return end_of_input(s);
}

// Ends the current line. Unread text (after a stop at a semicolon you never resumed from, say)
// and unfinished variable expansions are dropped; quote/comment/paren state is kept.
scan_finish :: (s: *Scanner) {
    while s.sources.count > 0 pop_source(s);
    s.out = null;
}

// Forgets the statement in progress: quote, comment, parenthesis and BEGIN state and the COPY
// count. Call it whenever the query buffer is cleared (statement sent, `\r`, an error).
scan_reset :: (s: *Scanner) {
    s.state = .INITIAL;
    s.paren_depth = 0;
    s.begin_depth = 0;
    s.comment_depth = 0;
    free(s.dollar_tag);
    s.dollar_tag = .{};
    s.idents_count = 0;
    s.copy_stdin_count = 0;
}

// Releases everything. The scanner can be reused afterwards.
scan_destroy :: (s: *Scanner) {
    scan_finish(s);
    scan_reset(s);
    array_free(s.sources);
    s.sources = .{};
}

// True while the scanner is inside a quoted string, identifier, dollar quote or comment. An
// empty input line is only skipped when this is false, because inside a quote it is data.
scan_in_quote :: (s: *Scanner) -> bool {
    return s.state != .INITIAL && s.state != .STRING_END;
}

// How many COPY ... FROM STDIN commands the statements in the query buffer hold. Call it just
// before sending the buffer: each one makes the server ask the client for data.
scan_count_copy_from_stdin :: (s: *Scanner) -> s64 {
    if s.idents_count > 0 {
        if is_copy_from_stdin(s) s.copy_stdin_count += 1;
        s.idents_count = 0;
    }

    return s.copy_stdin_count;
}

// The part of the scanner state that psql saves when an `\if` branch starts and puts back when
// it throws the branch's text away (see `discard_query_text` in psql's command.c): parenthesis and
// BEGIN depth, and what is known about the current statement's first words.
Lex_State :: struct {
    paren_depth: s64;
    begin_depth: s64;
    copy_stdin_count: s64;
    idents_count: s64;
    idents: [8] u8;
}

scan_save_lex_state :: (s: *Scanner) -> Lex_State {
    return .{ s.paren_depth, s.begin_depth, s.copy_stdin_count, s.idents_count, s.idents };
}

scan_restore_lex_state :: (s: *Scanner, saved: Lex_State) {
    s.paren_depth = saved.paren_depth;
    s.begin_depth = saved.begin_depth;
    s.copy_stdin_count = saved.copy_stdin_count;
    s.idents_count = saved.idents_count;
    s.idents = saved.idents;
}

// Reads the name of the backslash command that `scan` just reported (everything up to
// whitespace or the next backslash). `\d+` yields "d+".
scan_slash_command :: (s: *Scanner) -> string {
    name: [..] u8;

    while true {
        c := next_byte(s);
        if c < 0 || is_space(c) || c == #char "\\" break;
        array_add(*name, cast(u8) c);
        top(s).pos += 1;
    }

    return string.{ name.count, name.data };
}

// Reads the next argument of a backslash command. Returns the text, whether there was an
// argument at all, and how it was quoted: 0 when nothing in it was quoted, otherwise the last
// kind used: `'`, `"`, a backtick, or `:` for a variable reference.
//
// With `semicolon` set, unquoted trailing semicolons are removed (for WHOLE_LINE also trailing
// whitespace). An argument ends at whitespace or at a backslash, which stays unread. A missing
// argument, and an unterminated quote (which is also reported), give found == false.
scan_slash_option :: (s: *Scanner, kind: Option_Kind = .NORMAL, semicolon := false) -> string, bool, u8 {
    buf: [..] u8;
    quote: u8;
    unquoted := 0; // unquoted characters at the end of the argument

    whole_line := kind == .WHOLE_LINE;

    if !whole_line {
        while is_space(next_byte(s)) top(s).pos += 1;

        if next_byte(s) < 0 return "", false, 0;

        if kind == .FILE_OR_PIPE && next_byte(s) == #char "|" {
            array_add(*buf, #char "|");
            top(s).pos += 1;
            whole_line = true;
        }
    }

    if whole_line {
        while true {
            c := next_byte(s);
            if c < 0 break;

            top(s).pos += 1;
            if is_space(c) && buf.count == 0 continue;

            array_add(*buf, cast(u8) c);
        }
    } else {
        ok := read_argument(s, *buf, *quote, *unquoted);
        if !ok {
            array_free(buf);
            report_message(s, .ERROR, "unterminated quoted string");
            return "", false, 0;
        }
    }

    if semicolon {
        if whole_line {
            while buf.count > 0 && (buf[buf.count - 1] == #char ";" || is_space(buf[buf.count - 1])) buf.count -= 1;
        } else {
            while unquoted > 0 && buf.count > 0 && buf[buf.count - 1] == #char ";" {
                buf.count -= 1;
                unquoted -= 1;
            }
        }
    }

    // Strings are C strings in psql: a \0 escape ends the argument.
    for 0..buf.count - 1 {
        if buf[it] == 0 {
            buf.count = it;
            break;
        }
    }

    if kind == .SQL_ID || kind == .SQL_ID_KEEP_CASE {
        if !whole_line dequote_identifier(*buf, kind == .SQL_ID);
    }

    if buf.count == 0 && quote == 0 {
        array_free(buf);
        return "", false, 0;
    }

    return string.{ buf.count, buf.data }, true, quote;
}

// Ends a backslash command: a `\\` right here is consumed (it lets SQL follow on the same
// line); anything else is left for `scan`. Call it after the command's arguments are read.
scan_slash_command_end :: (s: *Scanner) {
    if next_byte(s) == #char "\\" && byte_at(s, 1) == #char "\\" top(s).pos += 2;
}

// The query buffer as text. Valid until the buffer changes.
buffer_string :: (buffer: [..] u8) -> string {
    return string.{ buffer.count, buffer.data };
}

// `value` as psql quotes it for `:'name'`: 'it''s', or E'back\\slash' (with a leading space) if it
// holds backslashes. Text after a zero byte is dropped.
escape_literal :: (value: string) -> string {
    out: [..] u8;
    append_literal(*out, value);
    return buffer_string(out);
}

// `value` as psql quotes it for `:"name"`: "say ""hi""".
escape_identifier :: (value: string) -> string {
    out: [..] u8;
    append_identifier(*out, value);
    return buffer_string(out);
}

// `value` quoted for a POSIX shell, as `:'name'` is inside backticks. False if it holds a
// newline or carriage return, which cannot be quoted safely.
shell_quote :: (value: string) -> string, bool {
    out: [..] u8;
    ok := append_shell_arg(*out, value);

    if !ok {
        array_free(out);
        return "", false;
    }

    return buffer_string(out), true;
}

// A set of psql variables, kept sorted by name as `\set` lists them.
Variables :: struct {
    entries: [..] Variable;
}

Variable :: struct {
    name: string;
    value: string;
}

// Characters allowed in a variable name: letters, digits, underscore and non-ASCII bytes.
valid_variable_name :: (name: string) -> bool {
    if !name.count return false;

    for 0..name.count - 1 {
        if !is_variable_char(name.data[it]) return false;
    }

    return true;
}

// Sets (or adds) a variable; name and value are copied.
variables_set :: (vars: *Variables, name: string, value: string) {
    index, found := variable_position(vars, name);
    if found {
        free(vars.entries[index].value);
        vars.entries[index].value = copy_string(value);
        return;
    }

    array_add(*vars.entries, .{});
    for < index + 1..vars.entries.count - 1 vars.entries[it] = vars.entries[it - 1];

    vars.entries[index] = .{ copy_string(name), copy_string(value) };
}

variables_get :: (vars: *Variables, name: string) -> string, bool {
    index, found := variable_position(vars, name);
    if !found return "", false;
    return vars.entries[index].value, true;
}

// Returns false if there was no such variable.
variables_unset :: (vars: *Variables, name: string) -> bool {
    index, found := variable_position(vars, name);
    if !found return false;

    free(vars.entries[index].name);
    free(vars.entries[index].value);
    for index..vars.entries.count - 2 vars.entries[it] = vars.entries[it + 1];

    vars.entries.count -= 1;

    return true;
}

// A `Variable_Lookup` over a `Variables`: set `scanner.lookup = variables_lookup` and
// `scanner.hook_data = *vars`.
variables_lookup :: (name: string, data: *void) -> string, bool {
    return variables_get(cast(*Variables) data, name);
}

// An argument read as NORMAL, dequoted and lowercased the way SQL_ID would have done it.
sql_identifier :: (text: string) -> string {
    buf: [..] u8;
    buf.allocator = temp;
    array_add(*buf, ..cast([] u8) text);
    dequote_identifier(*buf, true);

    return cast(string) buf;
}

#scope_file

Scan_State :: enum {
    INITIAL;
    STANDARD_STRING; // '...' with standard_conforming_strings on
    ESCAPE_STRING; // E'...', or '...' with it off
    BIT_STRING; // B'...'
    HEX_STRING; // X'...'
    UNICODE_STRING; // U&'...'
    STRING_END; // just past a closing quote: a string may continue after a newline
    COMMENT; // /* ... */
    QUOTED_IDENT; // "..."
    UNICODE_IDENT; // U&"..."
    DOLLAR; // $tag$ ... $tag$
}

// Text being scanned: the input line, or a variable's value that replaced a `:name`. Expansions
// are scanned like the line, and tokens never span two of them.
Source :: struct {
    text: string;
    pos: s64;
    name: string; // the variable it came from; empty for the line
    owned: bool;
}

top :: (s: *Scanner) -> *Source {
    return *s.sources[s.sources.count - 1];
}

// The byte `ahead` places after the read position in the current source, or -1.
byte_at :: (s: *Scanner, ahead: s64) -> s32 {
    t := top(s);
    if t.pos + ahead >= t.text.count return -1;
    return t.text.data[t.pos + ahead];
}

// Pops finished variable expansions and reports whether any input is left.
settle :: (s: *Scanner) -> bool {
    while s.sources.count > 0 {
        t := top(s);
        if t.pos < t.text.count return true;
        if s.sources.count == 1 return false;
        pop_source(s);
    }

    return false;
}

// The next byte of input (looking through finished expansions), or -1.
next_byte :: (s: *Scanner) -> s32 {
    if !settle(s) return -1;
    return byte_at(s, 0);
}

pop_source :: (s: *Scanner) {
    t := top(s);
    if t.owned {
        free(t.text);
        free(t.name);
    }

    s.sources.count -= 1;
}

push_expansion :: (s: *Scanner, name: string, value: string) {
    array_add(*s.sources, .{ text = copy_string(value), name = copy_string(name), owned = true });
}

// Copies `count` bytes of the current source into the query buffer and moves past them.
take :: (s: *Scanner, count: s64) {
    t := top(s);
    for 0..count - 1 array_add(s.out, t.text.data[t.pos + it]);
    t.pos += count;
}

skip :: (s: *Scanner, count: s64) {
    top(s).pos += count;
}

piece :: (s: *Scanner, from: s64, count: s64) -> string {
    t := top(s);
    return string.{ count, t.text.data + t.pos + from };
}

report_message :: (s: *Scanner, kind: Message_Kind, text: string) {
    if s.report s.report(kind, text, s.hook_data);
}

find_variable :: (s: *Scanner, name: string) -> string, bool {
    if !s.active || !s.lookup return "", false;
    return s.lookup(name, s.hook_data);
}

is_space :: (c: s32) -> bool {
    return c == #char " " || c == #char "\t" || c == #char "\n" || c == #char "\r" || c == 12 || c == 11;
}

is_newline :: (c: s32) -> bool {
    return c == #char "\n" || c == #char "\r";
}

is_digit :: (c: s32) -> bool {
    return c >= #char "0" && c <= #char "9";
}

is_ident_start :: (c: s32) -> bool {
    return (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c == #char "_" || c >= 128;
}

is_ident_cont :: (c: s32) -> bool {
    return is_ident_start(c) || is_digit(c) || c == #char "$";
}

// Dollar-quote tags are identifiers without the dollar sign.
is_tag_cont :: (c: s32) -> bool {
    return is_ident_start(c) || is_digit(c);
}

is_variable_char :: (c: s32) -> bool {
    return is_ident_start(c) || is_digit(c);
}

is_operator_char :: (c: s32) -> bool {
    if c == {
        case #char "~"; #through;
        case #char "!"; #through;
        case #char "@"; #through;
        case #char "#"; #through;
        case #char "^"; #through;
        case #char "&"; #through;
        case #char "|"; #through;
        case #char "`"; #through;
        case #char "?"; #through;
        case #char "+"; #through;
        case #char "-"; #through;
        case #char "*"; #through;
        case #char "/"; #through;
        case #char "%"; #through;
        case #char "<"; #through;
        case #char ">"; #through;
        case #char "=";
            return true;
    }

    return false;
}

is_hex_digit :: (c: s32) -> bool {
    return is_digit(c) || (c >= #char "a" && c <= #char "f") || (c >= #char "A" && c <= #char "F");
}

hex_value :: (c: s32) -> s32 {
    if is_digit(c) return c - #char "0";
    if c >= #char "a" return c - #char "a" + 10;
    return c - #char "A" + 10;
}

lower :: (c: u8) -> u8 {
    if c >= #char "A" && c <= #char "Z" return c + 32;
    return c;
}

equals_ignoring_case :: (a: string, b: string) -> bool {
    if a.count != b.count return false;

    for 0..a.count - 1 {
        if lower(a.data[it]) != lower(b.data[it]) return false;
    }

    return true;
}

// Length of the run of bytes from `from` that `s.sources` top holds and `test` accepts.
run_length :: (s: *Scanner, from: s64, test: (s32) -> bool) -> s64 {
    n := from;
    while test(byte_at(s, n)) n += 1;
    return n - from;
}

// ---- Statement scanning ----

// Scans one token in the current state. Returns true as the second value when `scan` must
// return the first.
scan_token :: (s: *Scanner) -> Scan_Result, bool {
    if s.state == {
        case .INITIAL;
            return scan_initial(s);

        case .STANDARD_STRING; #through;
        case .UNICODE_STRING; #through;
        case .ESCAPE_STRING; #through;
        case .BIT_STRING; #through;
        case .HEX_STRING;
            scan_string(s);

        case .STRING_END;
            // Adjacent string literals separated by a newline are one literal. The scanner sees
            // that only when the newline is in the same text, which is the case for variable
            // values and re-scanned buffers but not for separate input lines.
            n := quote_continuation_length(s);
            if n > 0 {
                take(s, n);
                s.state = s.state_before_string_end;
            } else {
                s.state = .INITIAL;
            }

        case .COMMENT;
            scan_comment(s);

        case .QUOTED_IDENT; #through;
        case .UNICODE_IDENT;
            n := 0;
            while byte_at(s, n) >= 0 && byte_at(s, n) != #char "\"" n += 1;

            if byte_at(s, n) == #char "\"" {
                n += 1;
                s.state = .INITIAL;
            }

            take(s, n);

        case .DOLLAR;
            scan_dollar_body(s);
    }

    return .EOL, false;
}

scan_initial :: (s: *Scanner) -> Scan_Result, bool {
    c := byte_at(s, 0);

    if is_space(c) {
        n := run_length(s, 0, is_space);
        keep_or_drop(s, n);
        return .EOL, false;
    }

    if c == #char "-" && byte_at(s, 1) == #char "-" {
        n := 2;
        while byte_at(s, n) >= 0 && !is_newline(byte_at(s, n)) n += 1;
        keep_or_drop(s, n);
        return .EOL, false;
    }

    if c == #char "/" && byte_at(s, 1) == #char "*" {
        s.state = .COMMENT;
        s.comment_depth = 0;
        take(s, 2);
        return .EOL, false;
    }

    if is_ident_start(c) {
        scan_word(s);
        return .EOL, false;
    }

    if is_digit(c) || (c == #char "." && is_digit(byte_at(s, 1))) {
        take(s, number_length(s));
        return .EOL, false;
    }

    if c == {
        case #char "'";
            s.state = ifx s.std_strings then Scan_State.STANDARD_STRING else .ESCAPE_STRING;
            take(s, 1);

        case #char "\"";
            s.state = .QUOTED_IDENT;
            take(s, 1);

        case #char "$";
            scan_dollar(s);

        case #char "(";
            s.paren_depth += 1;
            take(s, 1);

        case #char ")";
            if s.paren_depth > 0 s.paren_depth -= 1;

            take(s, 1);

        case #char ";";
            take(s, 1);

            if s.paren_depth == 0 && s.begin_depth == 0 {
                if is_copy_from_stdin(s) s.copy_stdin_count += 1;
                s.idents_count = 0;
                return .SEMICOLON, true;
            }

        case #char "\\";
            next := byte_at(s, 1);
            if next == #char ";" || next == #char ":" {
                // `\;` keeps a semicolon inside the statement, `\:` a literal colon.
                skip(s, 1);
                take(s, 1);

                if next == #char ";" && s.paren_depth == 0 && s.begin_depth == 0 {
                    if is_copy_from_stdin(s) s.copy_stdin_count += 1;
                    s.idents_count = 0;
                }
            } else {
                skip(s, 1);
                return .BACKSLASH, true;
            }

        case #char ":";
            scan_colon(s);

        case #char ".";
            take(s, ifx byte_at(s, 1) == #char "." then 2 else 1);

        case;
            if is_operator_char(c) {
                // An operator ends where a comment starts.
                n := 1;
                while is_operator_char(byte_at(s, n)) {
                    if byte_at(s, n) == #char "-" && byte_at(s, n + 1) == #char "-" break;
                    if byte_at(s, n) == #char "/" && byte_at(s, n + 1) == #char "*" break;
                    n += 1;
                }

                take(s, n);
            } else {
                take(s, 1);
            }
    }

    return .EOL, false;
}

// Whitespace and `--` comments before the first real text of a statement are not sent. (The
// server counts error positions from what it receives, so this matters.)
keep_or_drop :: (s: *Scanner, count: s64) {
    if s.out.count > 0 take(s, count);
    else skip(s, count);
}

// An identifier or keyword, or a prefixed string: E'..', B'..', X'..', N'..', U&'..', U&"..".
scan_word :: (s: *Scanner) {
    c := byte_at(s, 0);
    next := byte_at(s, 1);

    if next == #char "'" {
        if c == #char "b" || c == #char "B" {
            s.state = .BIT_STRING;
            take(s, 2);
            return;
        }

        if c == #char "x" || c == #char "X" {
            s.state = .HEX_STRING;
            take(s, 2);
            return;
        }

        if c == #char "e" || c == #char "E" {
            s.state = .ESCAPE_STRING;
            take(s, 2);
            return;
        }

        if c == #char "n" || c == #char "N" {
            // N'..' is an ordinary string; the quote is scanned next.
            take(s, 1);
            return;
        }
    }

    if (c == #char "u" || c == #char "U") && next == #char "&" {
        quote := byte_at(s, 2);
        if quote == #char "'" {
            s.state = .UNICODE_STRING;
            take(s, 3);
        } else if quote == #char "\"" {
            s.state = .UNICODE_IDENT;
            take(s, 3);
        } else {
            take(s, 1);
        }

        return;
    }

    n := 1 + run_length(s, 1, is_ident_cont);

    track_identifier(s, piece(s, 0, n));
    take(s, n);
}

// A numeric literal, including the identifier characters that would make it invalid (1abc, 0x1F).
// They must stay one token so that a keyword glued to a number is not seen as a keyword.
number_length :: (s: *Scanner) -> s64 {
    n := 0;

    digits :: (s: *Scanner, from: s64) -> s64 {
        n := from;
        while is_digit(byte_at(s, n)) || (byte_at(s, n) == #char "_" && is_digit(byte_at(s, n + 1)) && n > from) n += 1;
        return n;
    }

    if is_digit(byte_at(s, 0)) {
        n = digits(s, 0);
        // `1..10` is 1, "..", 10.
        if byte_at(s, n) == #char "." && byte_at(s, n + 1) != #char "." {
            n = digits(s, n + 1);
        }
    } else {
        n = digits(s, 1);
    }

    e := byte_at(s, n);
    if e == #char "e" || e == #char "E" {
        sign := byte_at(s, n + 1);
        if sign == #char "+" || sign == #char "-" {
            n = digits(s, n + 2);
        } else if is_digit(sign) {
            n = digits(s, n + 1);
        }
    }

    if is_ident_start(byte_at(s, n)) n += run_length(s, n, is_ident_cont);

    return n;
}

// At a `$`: a dollar-quote opener ($$ or $tag$), a parameter ($1), or a lone dollar sign.
scan_dollar :: (s: *Scanner) {
    next := byte_at(s, 1);

    if next == #char "$" {
        open_dollar_quote(s, 2);
    } else if is_ident_start(next) {
        n := 1 + run_length(s, 1, is_tag_cont);
        if byte_at(s, n) == #char "$" {
            open_dollar_quote(s, n + 1);
        } else {
            take(s, 1); // `$tag` without a closing dollar: just a dollar sign, then a word
        }
    } else if is_digit(next) {
        n := 1 + run_length(s, 1, is_digit);
        if is_ident_start(byte_at(s, n)) n += run_length(s, n, is_ident_cont);
        take(s, n);
    } else {
        take(s, 1);
    }
}

open_dollar_quote :: (s: *Scanner, length: s64) {
    free(s.dollar_tag);
    s.dollar_tag = copy_string(piece(s, 0, length));
    s.state = .DOLLAR;
    take(s, length);
}

// Inside a dollar-quoted string: everything is text until the opener appears again.
scan_dollar_body :: (s: *Scanner) {
    n := 0;
    while byte_at(s, n) >= 0 && byte_at(s, n) != #char "$" n += 1;

    if n > 0 {
        take(s, n);
        return;
    }

    next := byte_at(s, 1);
    delimiter_length := 0;

    if next == #char "$" {
        delimiter_length = 2;
    } else if is_ident_start(next) {
        n = 1 + run_length(s, 1, is_tag_cont);
        if byte_at(s, n) == #char "$" {
            delimiter_length = n + 1;
        } else {
            take(s, n); // `$word` that is not closed: plain text
            return;
        }
    } else {
        take(s, 1);
        return;
    }

    if piece(s, 0, delimiter_length) == s.dollar_tag {
        take(s, delimiter_length);
        free(s.dollar_tag);
        s.dollar_tag = .{};
        s.state = .INITIAL;
    } else {
        // `$a$b$a$` inside `$a$`: the second dollar may open the real terminator.
        take(s, delimiter_length - 1);
    }
}

scan_string :: (s: *Scanner) {
    escapes := s.state == .ESCAPE_STRING;
    doubles := s.state != .BIT_STRING && s.state != .HEX_STRING; // '' inside the string

    n := 0;

    while true {
        c := byte_at(s, n);
        if c < 0 break;

        if c == #char "'" {
            if doubles && byte_at(s, n + 1) == #char "'" {
                n += 2;
                continue;
            }

            // The closing quote. A string literal may continue after it, so the decision of
            // whether this is the end waits in STRING_END.
            n += 1;
            s.state_before_string_end = s.state;
            s.state = .STRING_END;
            break;
        }

        if escapes && c == #char "\\" && byte_at(s, n + 1) >= 0 {
            n += 2; // the escaped character, which may be a quote
            continue;
        }

        n += 1;
    }

    take(s, n);
}

// Where a quote continues a string literal: blanks and comments, a newline, more whitespace and
// comments, then another quote. Returns the length through that quote, or 0.
quote_continuation_length :: (s: *Scanner) -> s64 {
    j := 0;

    while true {
        c := byte_at(s, j);
        if c == #char " " || c == #char "\t" || c == 12 || c == 11 {
            j += 1;
        } else if c == #char "-" && byte_at(s, j + 1) == #char "-" {
            j += 2;
            while byte_at(s, j) >= 0 && !is_newline(byte_at(s, j)) j += 1;
        } else {
            break;
        }
    }

    if !is_newline(byte_at(s, j)) return 0;

    j += 1;

    while true {
        c := byte_at(s, j);
        if is_space(c) {
            j += 1;
        } else if c == #char "-" && byte_at(s, j + 1) == #char "-" {
            k := j + 2;
            while byte_at(s, k) >= 0 && !is_newline(byte_at(s, k)) k += 1;
            if !is_newline(byte_at(s, k)) return 0;
            j = k + 1;
        } else {
            break;
        }
    }

    if byte_at(s, j) == #char "'" return j + 1;

    return 0;
}

// Inside /* */: comments nest, and everything is kept.
scan_comment :: (s: *Scanner) {
    c := byte_at(s, 0);

    if c == #char "/" && byte_at(s, 1) == #char "*" {
        s.comment_depth += 1;
        take(s, 2);
    } else if c == #char "*" {
        n := 1 + run_length(s, 1, is_star);
        if byte_at(s, n) == #char "/" {
            n += 1;
            if s.comment_depth > 0 s.comment_depth -= 1;
            else s.state = .INITIAL;
        }

        take(s, n);
    } else {
        n := 1;
        while byte_at(s, n) >= 0 && byte_at(s, n) != #char "*" && byte_at(s, n) != #char "/" n += 1;
        take(s, n);
    }
}

is_star :: (c: s32) -> bool {
    return c == #char "*";
}

// At a colon: a cast (::), assignment (:=), a variable reference, or just a colon.
scan_colon :: (s: *Scanner) {
    next := byte_at(s, 1);

    if next == #char ":" || next == #char "=" {
        take(s, 2);
        return;
    }

    if is_variable_char(next) {
        n := run_length(s, 1, is_variable_char);
        name := piece(s, 1, n);
        value, found := find_variable(s, name);

        if !found {
            take(s, 1 + n);
        } else if expanding(s, name) {
            // A value that mentions itself would expand forever.
            report_message(s, .WARNING, tprint("skipping recursive expansion of variable \"%\"", name));
            take(s, 1 + n);
        } else {
            name_copy := copy_string(name); // the push may move the array that `name` points into
            skip(s, 1 + n);
            push_expansion(s, name_copy, value);
            free(name_copy);
        }

        return;
    }

    if next == #char "'" || next == #char "\"" {
        n := run_length(s, 2, is_variable_char);
        if n > 0 && byte_at(s, 2 + n) == next {
            value, found := find_variable(s, piece(s, 2, n));
            if found {
                if next == #char "'" append_literal(s.out, value);
                else append_identifier(s.out, value);
                skip(s, 3 + n);
                return;
            }

            take(s, 3 + n);

            return;
        }
    }

    if next == #char "{" && byte_at(s, 2) == #char "?" {
        n := run_length(s, 3, is_variable_char);
        if n > 0 && byte_at(s, 3 + n) == #char "}" {
            _, found := find_variable(s, piece(s, 3, n));
            append_text(s.out, ifx found then "TRUE" else "FALSE");
            skip(s, 4 + n);
            return;
        }
    }

    take(s, 1);
}

expanding :: (s: *Scanner, name: string) -> bool {
    for s.sources if it.name == name return true;
    return false;
}

append_text :: (out: *[..] u8, text: string) {
    for 0..text.count - 1 array_add(out, text.data[it]);
}

// Text up to the first zero byte, as a C string would end.
c_prefix :: (value: string) -> string {
    for 0..value.count - 1 if value.data[it] == 0 return string.{ it, value.data };
    return value;
}

append_literal :: (out: *[..] u8, text: string) {
    value := c_prefix(text);

    has_backslash := false;

    for 0..value.count - 1 if value.data[it] == #char "\\" has_backslash = true;

    if has_backslash append_text(out, " E'");
    else array_add(out, #char "'");

    for 0..value.count - 1 {
        c := value.data[it];
        if c == #char "'" || (has_backslash && c == #char "\\") array_add(out, c);
        array_add(out, c);
    }

    array_add(out, #char "'");
}

append_identifier :: (out: *[..] u8, text: string) {
    value := c_prefix(text);

    array_add(out, #char "\"");

    for 0..value.count - 1 {
        c := value.data[it];
        if c == #char "\"" array_add(out, c);
        array_add(out, c);
    }

    array_add(out, #char "\"");
}

append_shell_arg :: (out: *[..] u8, value: string) -> bool {
    for 0..value.count - 1 if value.data[it] == #char "\n" || value.data[it] == #char "\r" return false;

    array_add(out, #char "'");

    for 0..value.count - 1 {
        c := value.data[it];
        if c == #char "'" append_text(out, "'\\''");
        else array_add(out, c);
    }

    array_add(out, #char "'");

    return true;
}

// ---- COPY FROM STDIN and BEGIN ... END tracking ----

// Notes the first few words of a statement, and counts BEGIN ... END nesting inside
// CREATE [OR REPLACE] FUNCTION/PROCEDURE, whose bodies may hold semicolons.
track_identifier :: (s: *Scanner, word: string) {
    if s.paren_depth != 0 return;

    if s.idents_count == 0 for *s.idents it.* = 0;

    if s.idents_count < s.idents.count {
        // Keywords for CREATE ... FUNCTION are recorded as a lower-case letter, those for
        // COPY ... FROM STDIN as an upper-case one; anything else leaves zero.
        mark: u8;

        if equals_ignoring_case(word, "create") || equals_ignoring_case(word, "function") || equals_ignoring_case(word, "procedure") || equals_ignoring_case(word, "or") || equals_ignoring_case(word, "replace") {
            mark = lower(word.data[0]);
        } else if equals_ignoring_case(word, "copy") || equals_ignoring_case(word, "from") || equals_ignoring_case(word, "stdin") || equals_ignoring_case(word, "stdout") {
            mark = lower(word.data[0]) - 32;
        }

        s.idents[s.idents_count] = mark;
        s.idents_count += 1;
    }

    if !is_create_routine(s) return;

    if equals_ignoring_case(word, "begin") {
        s.begin_depth += 1;
    } else if equals_ignoring_case(word, "case") {
        // CASE ends with END as well, but only matters inside a BEGIN body.
        if s.begin_depth >= 1 s.begin_depth += 1;
    } else if equals_ignoring_case(word, "end") {
        if s.begin_depth > 0 s.begin_depth -= 1;
    }
}

is_create_routine :: (s: *Scanner) -> bool {
    i := s.idents;
    if i[0] != #char "c" return false;
    if i[1] == #char "f" || i[1] == #char "p" return true;
    return i[1] == #char "o" && i[2] == #char "r" && (i[3] == #char "f" || i[3] == #char "p");
}

is_copy_from_stdin :: (s: *Scanner) -> bool {
    if s.idents[0] != #char "C" return false;

    // Up to four words (BINARY db.schema.table) may sit between COPY and FROM.
    for 1..s.idents.count - 2 {
        if s.idents[it] != #char "F" continue;
        return s.idents[it + 1] == #char "S";
    }

    return false;
}

end_of_input :: (s: *Scanner) -> Scan_Result, Prompt {
    if s.state == {
        case .INITIAL; #through;
        case .STRING_END;
            if s.paren_depth > 0 return .INCOMPLETE, .PAREN;
            if s.begin_depth > 0 return .INCOMPLETE, .CONTINUE;
            if s.out.count > 0 return .EOL, .CONTINUE;

            // Nothing worth sending.
            return .INCOMPLETE, .READY;

        case .COMMENT;
            return .INCOMPLETE, .COMMENT;

        case .QUOTED_IDENT; #through;
        case .UNICODE_IDENT;
            return .INCOMPLETE, .DOUBLE_QUOTE;

        case .DOLLAR;
            return .INCOMPLETE, .DOLLAR_QUOTE;
    }

    return .INCOMPLETE, .SINGLE_QUOTE;
}

// ---- Backslash command arguments ----

// Reads a NORMAL argument. Returns false for an unterminated quote.
read_argument :: (s: *Scanner, buf: *[..] u8, quote: *u8, unquoted: *s64) -> bool {
    while true {
        c := next_byte(s);
        if c < 0 || is_space(c) || c == #char "\\" break;

        if c == #char "'" {
            quote.* = #char "'";
            unquoted.* = 0;
            skip(s, 1);
            if !read_single_quoted(s, buf) return false;
        } else if c == #char "`" {
            quote.* = #char "`";
            unquoted.* = 0;
            skip(s, 1);
            if !read_backticked(s, buf) return false;
        } else if c == #char "\"" {
            array_add(buf, #char "\"");
            quote.* = #char "\"";
            unquoted.* = 0;
            skip(s, 1);

            // Double-quoted text is kept as it is, quotes included.
            while true {
                d := next_byte(s);
                if d < 0 return false;

                array_add(buf, cast(u8) d);
                skip(s, 1);
                if d == #char "\"" break;
            }
        } else if c == #char ":" {
            read_argument_colon(s, buf, quote, unquoted);
        } else {
            array_add(buf, cast(u8) c);
            unquoted.* += 1;
            skip(s, 1);
        }
    }

    return true;
}

read_argument_colon :: (s: *Scanner, buf: *[..] u8, quote: *u8, unquoted: *s64) {
    next := byte_at(s, 1);

    if is_variable_char(next) {
        n := run_length(s, 1, is_variable_char);
        value, found := find_variable(s, piece(s, 1, n));

        // The value goes in as it is: it is not scanned again.
        if found append_text(buf, value);
        else append_text(buf, piece(s, 0, 1 + n));

        skip(s, 1 + n);
        quote.* = #char ":";
        unquoted.* = 0;

        return;
    }

    if next == #char "'" || next == #char "\"" {
        n := run_length(s, 2, is_variable_char);
        if n > 0 && byte_at(s, 2 + n) == next {
            value, found := find_variable(s, piece(s, 2, n));
            if found {
                if next == #char "'" append_literal(buf, value);
                else append_identifier(buf, value);
            } else {
                append_text(buf, piece(s, 0, 3 + n));
            }

            skip(s, 3 + n);
            quote.* = #char ":";
            unquoted.* = 0;

            return;
        }
    }

    if next == #char "{" && byte_at(s, 2) == #char "?" {
        n := run_length(s, 3, is_variable_char);
        if n > 0 && byte_at(s, 3 + n) == #char "}" {
            _, found := find_variable(s, piece(s, 3, n));
            append_text(buf, ifx found then "TRUE" else "FALSE");
            skip(s, 4 + n);
            return;
        }
    }

    array_add(buf, #char ":");
    unquoted.* += 1;
    skip(s, 1);
}

// After the opening quote, up to and including the closing one.
read_single_quoted :: (s: *Scanner, buf: *[..] u8) -> bool {
    while true {
        c := next_byte(s);
        if c < 0 return false;

        if c == #char "'" {
            if byte_at(s, 1) == #char "'" {
                array_add(buf, #char "'");
                skip(s, 2);
                continue;
            }

            skip(s, 1);

            return true;
        }

        if c != #char "\\" || byte_at(s, 1) < 0 {
            array_add(buf, cast(u8) c);
            skip(s, 1);
            continue;
        }

        e := byte_at(s, 1);
        if e == #char "n" {
            array_add(buf, #char "\n");
        } else if e == #char "t" {
            array_add(buf, #char "\t");
        } else if e == #char "b" {
            array_add(buf, 8);
        } else if e == #char "r" {
            array_add(buf, #char "\r");
        } else if e == #char "f" {
            array_add(buf, 12);
        } else if e >= #char "0" && e <= #char "7" {
            value: s32;
            n := 1;

            while n <= 3 && byte_at(s, n) >= #char "0" && byte_at(s, n) <= #char "7" {
                value = value * 8 + (byte_at(s, n) - #char "0");
                n += 1;
            }

            array_add(buf, cast,trunc(u8) value);
            skip(s, n);
            continue;
        } else if e == #char "x" && is_hex_digit(byte_at(s, 2)) {
            value := hex_value(byte_at(s, 2));
            n := 3;

            if is_hex_digit(byte_at(s, 3)) {
                value = value * 16 + hex_value(byte_at(s, 3));
                n = 4;
            }

            array_add(buf, cast,trunc(u8) value);
            skip(s, n);
            continue;
        } else {
            array_add(buf, cast(u8) e); // a backslash before anything else just quotes it
        }

        skip(s, 2);
    }

    return false;
}

// After the opening backtick: the command text, with variables substituted, runs in a shell
// and its output replaces it.
read_backticked :: (s: *Scanner, buf: *[..] u8) -> bool {
    command: [..] u8;

    defer array_free(command);

    while true {
        c := next_byte(s);
        if c < 0 return false;

        if c == #char "`" {
            skip(s, 1);
            break;
        }

        if c == #char ":" {
            next := byte_at(s, 1);

            if is_variable_char(next) {
                n := run_length(s, 1, is_variable_char);
                value, found := find_variable(s, piece(s, 1, n));

                if found append_text(*command, value);
                else append_text(*command, piece(s, 0, 1 + n));

                skip(s, 1 + n);
                continue;
            }

            if next == #char "'" {
                n := run_length(s, 2, is_variable_char);
                if n > 0 && byte_at(s, 2 + n) == #char "'" {
                    value, found := find_variable(s, piece(s, 2, n));
                    quoted := false;

                    if found quoted = append_shell_arg(*command, value);
                    if !quoted append_text(*command, piece(s, 0, 3 + n));

                    skip(s, 3 + n);
                    continue;
                }
            }
        }

        array_add(*command, cast(u8) c);
        skip(s, 1);
    }

    // In a dead \if branch the text is not run and just stays where it is.
    if !s.active {
        append_text(buf, buffer_string(command));
        return true;
    }

    if !s.run_shell {
        s.backtick_failed = true;
        report_message(s, .ERROR, "backtick expansion is not supported");
        return true;
    }

    output, ok := s.run_shell(buffer_string(command), s.hook_data);
    if !ok {
        s.backtick_failed = true;
        return true;
    }

    if output.count > 0 && output.data[output.count - 1] == #char "\n" output.count -= 1;

    append_text(buf, output);

    return true;
}

// Removes the double quotes from an identifier argument and, if asked, lowercases the rest.
// "" inside quotes is one quote; a name like FOO"BAR"BAZ becomes fooBARbaz.
dequote_identifier :: (buf: *[..] u8, downcase: bool) {
    result: [..] u8;
    in_quotes := false;

    i := 0;

    while i < buf.count {
        c := buf.*[i];
        if c == #char "\"" {
            if in_quotes && i + 1 < buf.count && buf.*[i + 1] == #char "\"" {
                array_add(*result, c);
                i += 1;
            } else {
                in_quotes = !in_quotes;
            }
        } else if downcase && !in_quotes {
            array_add(*result, lower(c));
        } else {
            array_add(*result, c);
        }

        i += 1;
    }

    array_free(buf.*);
    buf.* = result;
}

// ---- Variables ----

// Where `name` is, or would be inserted to keep the entries sorted.
variable_position :: (vars: *Variables, name: string) -> s64, bool {
    for vars.entries {
        if it.name == name return it_index, true;
        if compare_bytes(name, it.name) < 0 return it_index, false;
    }

    return vars.entries.count, false;
}

compare_bytes :: (a: string, b: string) -> s32 {
    n := ifx a.count < b.count then a.count else b.count;
    for 0..n - 1 {
        if a.data[it] != b.data[it] return ifx a.data[it] < b.data[it] then -1 else 1;
    }

    if a.count == b.count return 0;

    return ifx a.count < b.count then -1 else 1;
}
