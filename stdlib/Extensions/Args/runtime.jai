// The part of Args that does not depend on the user's struct: the token loop, value conversion
// and error construction. The generated code (generate.jai) supplies the per-struct handlers
// that this loop calls.

#scope_module

Outcome :: enum u8 {
    NOT_FOUND; // not one of this command's options
    FLAG; // consumed one flag; the rest of a short bundle, if any, is still to be read
    DONE; // consumed the whole token (and a following value, if the option takes one)
    ERROR; // state.error says why
}

Option_Handler :: #type (out: *void, set: *void, state: *Parse_State, name: string, is_short: bool, rest: string, has_rest: bool, only_globals: bool) -> Outcome;
Positional_Handler :: #type (out: *void, set: *void, state: *Parse_State, text: string) -> bool;
Subcommand_Handler :: #type (out: *void, set: *void, state: *Parse_State, name: string) -> Outcome;
Finish_Handler :: #type (out: *void, set: *void, state: *Parse_State) -> bool;
Describe_Proc :: #type (program: string) -> Command_Spec;

// Handlers generated for one struct.
Command_Handlers :: struct {
    option: Option_Handler;
    positional: Positional_Handler;
    subcommand: Subcommand_Handler;
    finish: Finish_Handler;
    has_subcommands: bool;
    has_version: bool;
}

// The `@global` options of the commands above the one being parsed.
Global_Link :: struct {
    out: *void;
    set: *void;
    handle: Option_Handler;
    next: *Global_Link;
}

Parse_State :: struct {
    argv: [] string;
    cursor: s64;
    positional_index: s64;
    error: Error;
    globals: *Global_Link;
    program: string;
    path: [..] string; // subcommand names entered so far
    describe: Describe_Proc;
    environment: [] string; // NAME=value entries that win over the process environment
}

Range :: struct {
    present: bool;
    lo: s64;
    hi: s64;
    flo: float64;
    fhi: float64;
    text: string;
}

NO_RANGE :: Range.{};

fail :: (state: *Parse_State, kind: Error_Kind, option: string, value: string, format: string, args: ..Any) {
    if state.error.kind != .NONE return;
    state.error.kind = kind;
    state.error.option = copy_string(option);
    state.error.value = copy_string(value);
    state.error.message = copy_string(tprint(format, ..args));
}

// Reads the token loop for one command. Returns false with state.error set on any failure
// (including a request for --help or --version, which are errors of kind .HELP and .VERSION).
run_command :: (handlers: *Command_Handlers, out: *void, set: *void, state: *Parse_State) -> bool {
    options_ended := false;
    while state.cursor < state.argv.count {
        token := state.argv[state.cursor];
        state.cursor += 1;

        if options_ended || token.count < 2 || token[0] != #char "-" {
            if handlers.has_subcommands {
                outcome := handlers.subcommand(out, set, state, token);
                if outcome == .ERROR return false;
                if outcome == .NOT_FOUND {
                    fail(state, .UNKNOWN_COMMAND, token, token, "unrecognized subcommand `%`", token);
                    return false;
                }
                continue;
            }
            if !handlers.positional(out, set, state, token) return false;
            continue;
        }
        if token == "--" {
            options_ended = true;
            continue;
        }

        if token[1] == #char "-" {
            name := slice(token, 2, token.count - 2);
            rest: string;
            has_rest := false;
            equals := find_index_from_left(name, #char "=");
            if equals >= 0 {
                rest = slice(name, equals + 1, name.count - equals - 1);
                name = slice(name, 0, equals);
                has_rest = true;
            }
            if name == "help" {
                request_help(state);
                return false;
            }
            if name == "version" && handlers.has_version {
                request_version(state);
                return false;
            }
            outcome := find_option(handlers, out, set, state, name, false, rest, has_rest);
            if outcome == .NOT_FOUND {
                fail(state, .UNKNOWN_OPTION, tprint("--%", name), name, "unknown option `--%`", name);
                return false;
            }
            if outcome == .ERROR return false;
            continue;
        }

        i := 1;
        while i < token.count {
            name := slice(token, i, 1);
            rest := slice(token, i + 1, token.count - i - 1);
            if name == "h" {
                request_help(state);
                return false;
            }
            if name == "V" && handlers.has_version {
                request_version(state);
                return false;
            }
            outcome := find_option(handlers, out, set, state, name, true, rest, rest.count > 0);
            if outcome == .NOT_FOUND {
                fail(state, .UNKNOWN_OPTION, tprint("-%", name), name, "unknown option `-%`", name);
                return false;
            }
            if outcome == .ERROR return false;
            if outcome == .DONE break;
            i += 1;
        }
    }
    return handlers.finish(out, set, state);
}

find_option :: (handlers: *Command_Handlers, out: *void, set: *void, state: *Parse_State, name: string, is_short: bool, rest: string, has_rest: bool) -> Outcome {
    outcome := handlers.option(out, set, state, name, is_short, rest, has_rest, false);
    if outcome != .NOT_FOUND return outcome;
    link := state.globals;
    while link {
        outcome = link.handle(link.out, link.set, state, name, is_short, rest, has_rest, true);
        if outcome != .NOT_FOUND return outcome;
        link = link.next;
    }
    return .NOT_FOUND;
}

request_help :: (state: *Parse_State) {
    if state.error.kind != .NONE return;
    state.error.kind = .HELP;
}

request_version :: (state: *Parse_State) {
    if state.error.kind != .NONE return;
    state.error.kind = .VERSION;
}

// The value of an option: the rest of the token (`--jobs=4`, `-j4`), else the next argument.
take_value :: (state: *Parse_State, display: string, rest: string, has_rest: bool, is_short: bool, value: *string) -> bool {
    if has_rest {
        text := rest;
        if is_short && text.count > 0 && text[0] == #char "=" text = slice(text, 1, text.count - 1);
        value.* = text;
        return true;
    }
    if state.cursor >= state.argv.count {
        fail(state, .MISSING_VALUE, display, "", "option `%` needs a value", display);
        return false;
    }
    value.* = state.argv[state.cursor];
    state.cursor += 1;
    return true;
}

unexpected_value :: (state: *Parse_State, display: string) -> Outcome {
    fail(state, .UNEXPECTED_VALUE, display, "", "option `%` does not take a value", display);
    return .ERROR;
}

bump :: (counter: *$V) {
    next := counter.* + 1;
    if next > counter.* counter.* = next;
}

parse_bool_text :: (text: string, value: *bool) -> bool {
    lowered := to_lower_copy(text,, temp);
    if lowered == "1" || lowered == "true" || lowered == "yes" || lowered == "on" {
        value.* = true;
        return true;
    }
    if lowered == "0" || lowered == "false" || lowered == "no" || lowered == "off" {
        value.* = false;
        return true;
    }
    return false;
}

store_flag_text :: (state: *Parse_State, display: string, text: string, dest: *bool) -> bool {
    if parse_bool_text(text, dest) return true;
    fail(state, .INVALID_VALUE, display, text, "invalid value `%` for `%`: expected true or false", text, display);
    return false;
}

// Converts `text` to the field's type. Enums are generated per field (store_enum_*), so this
// handles strings, integers and floats.
store_value :: (state: *Parse_State, display: string, text: string, dest: *$V, range: Range) -> bool {
    #if V == string {
        dest.* = text;
        return true;
    } else #if type_info(V).type == .INTEGER {
        number, ok, remainder := string_to_int(text, T = V);
        if !ok || remainder.count {
            if looks_like_integer(text) fail(state, .INVALID_VALUE, display, text, "value `%` for `%` does not fit in %", text, display, V);
            else fail(state, .INVALID_VALUE, display, text, "invalid value `%` for `%`: expected an integer", text, display);
            return false;
        }
        if range.present {
            wide := cast,no_check(s64) number;
            below := wide < range.lo;
            above := wide > range.hi;
            if below || above {
                fail(state, .OUT_OF_RANGE, display, text, "value `%` for `%` is out of range (%)", text, display, range.text);
                return false;
            }
        }
        dest.* = number;
        return true;
    } else #if type_info(V).type == .FLOAT {
        number, ok, remainder := string_to_float64(text);
        if !ok || remainder.count {
            fail(state, .INVALID_VALUE, display, text, "invalid value `%` for `%`: expected a number", text, display);
            return false;
        }
        if range.present && (number < range.flo || number > range.fhi) {
            fail(state, .OUT_OF_RANGE, display, text, "value `%` for `%` is out of range (%)", text, display, range.text);
            return false;
        }
        dest.* = cast(V) number;
        return true;
    } else {
        #assert false "Args: unsupported value type";
    }
}

looks_like_integer :: (text: string) -> bool {
    start := 0;
    if text.count && (text[0] == #char "-" || text[0] == #char "+") start = 1;
    if start >= text.count return false;
    for start..text.count - 1 if text[it] < #char "0" || text[it] > #char "9" return false;
    return true;
}

// The error for a value that is not one of an enum's choices.
bad_choice :: (state: *Parse_State, display: string, text: string, choices: [] string) {
    list := join(..choices, ", ");
    suggestion := closest(text, choices);
    if suggestion.count {
        fail(state, .INVALID_VALUE, display, text, "invalid value `%` for `%` (possible values: %; did you mean `%`?)", text, display, list, suggestion);
    } else {
        fail(state, .INVALID_VALUE, display, text, "invalid value `%` for `%` (possible values: %)", text, display, list);
    }
}

// Edit-distance suggestion: the candidate nearest to `word`, or "" when none is close.
closest :: (word: string, candidates: [] string) -> string {
    best := "";
    best_distance := 1000;
    for candidates {
        distance := edit_distance(word, it);
        if distance < best_distance {
            best_distance = distance;
            best = it;
        }
    }
    limit := max(1, word.count / 3);
    if word.count > 3 limit = max(limit, 2);
    if best_distance > limit return "";
    return best;
}

edit_distance :: (a: string, b: string) -> s64 {
    if !a.count return b.count;
    if !b.count return a.count;
    previous := NewArray(b.count + 1, s64,, temp);
    current := NewArray(b.count + 1, s64,, temp);
    for 0..b.count previous[it] = it;
    for i: 1..a.count {
        current[0] = i;
        for j: 1..b.count {
            cost := ifx a[i - 1] == b[j - 1] then 0 else 1;
            best := previous[j] + 1;
            if current[j - 1] + 1 < best best = current[j - 1] + 1;
            if previous[j - 1] + cost < best best = previous[j - 1] + cost;
            current[j] = best;
        }
        swap := previous;
        previous = current;
        current = swap;
    }
    return previous[b.count];
}

// What `default: ...` shows in help for a field's initial value; "" when it is the zero value.
default_text :: (value: $V) -> string {
    #if V == bool {
        return "";
    } else #if V == string {
        return value;
    } else #if type_info(V).type == .ENUM {
        if cast(s64) value == 0 return "";
        return kebab(tprint("%", value));
    } else #if type_info(V).type == .INTEGER || type_info(V).type == .FLOAT {
        if value == 0 return "";
        return tprint("%", value);
    } else {
        return "";
    }
}

// FIRST_SECOND -> first-second, the spelling of enum values and of long option names.
kebab :: (name: string) -> string {
    result := copy_string(name,, temp);
    for 0..result.count - 1 {
        if result[it] == #char "_" result[it] = #char "-";
        else if result[it] >= #char "A" && result[it] <= #char "Z" result[it] += 32;
    }
    return result;
}

// The value of an environment variable for `@env` fallbacks; empty when unset.
environment_lookup :: (state: *Parse_State, name: string) -> string {
    for entry: state.environment {
        if entry.count > name.count && begins_with(entry, name) && entry[name.count] == #char "=" {
            return slice(entry, name.count + 1, entry.count - name.count - 1);
        }
    }
    return environment_value(name);
}
