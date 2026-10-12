// Picks a matcher for each search: the lazy DFAs find where a match ends and starts, then the
// one-pass table (patterns that have one), the backtracker (short matches) or the Pike VM fills
// in captures. The Pike VM also takes over
// when a DFA grows too big, and for Unicode \b on non-ASCII text, which the DFA can't see.

Engine :: struct {
    pattern: string;
    program: Program;
    reverse: Program;
    forward: Dfa;
    backward: Dfa;
    onepass: OnePass; // the one-pass capture table, if the pattern has one (onepass.ok)
    frozen: bool; // built at compile time: both DFAs are complete and the programs are constants
}

build_engine :: (pattern: string, options: Options) -> *Engine, bool, Error {
    parsed := parse_pattern(pattern, options);
    if !parsed.ok return null, false, .{ parsed.error, parsed.offset };
    engine := New(Engine);
    engine.pattern = pattern;
    ok: bool;
    message: string;
    engine.program, ok, message = compile_program(*parsed, false);
    if !ok {
        free(engine);
        return null, false, .{ message, 0 };
    }
    engine.reverse, ok, message = compile_program(*parsed, true);
    if !ok {
        free(engine);
        return null, false, .{ message, 0 };
    }
    dfa_init(*engine.forward, *engine.program, false);
    dfa_init(*engine.backward, *engine.reverse, true);
    engine.onepass = onepass_build(*engine.program, ONEPASS_STATE_LIMIT);
    return engine, true, .{};
}

// The leftmost-first match at or after `start`: its start and end, or end -1. `earliest` only
// promises to say whether there is one (the end may be early and the start is not computed).
find_span :: (engine: *Engine, text: string, start: int, earliest := false) -> start: int, end: int {
    if start > text.count return -1, -1;
    program := *engine.program;
    if program.anchored && start > 0 return -1, -1;
    end, gave_up := dfa_find_end(*engine.forward, text, start, program.anchored, earliest);
    if !gave_up {
        if end < 0 return -1, -1;
        if earliest return -1, end;
        s, gave_up_backward := dfa_find_start(*engine.backward, text, start, end);
        if !gave_up_backward return s, end;
    }
    slots: [2] int;
    if !pike_search(program, text, start, text.count, program.anchored, slots) return -1, -1;
    return slots[0], slots[1];
}

captures_of :: (engine: *Engine, text: string, s: int, e: int) -> Captures {
    caps: Captures;
    caps.text = text;
    caps.names = engine.program.names;
    caps.slots = NewArray(engine.program.slot_count, int);
    fill_slots(engine, text, s, e, caps.slots);
    return caps;
}

// Where each group of the match text[s .. e] is.
fill_slots :: (engine: *Engine, text: string, s: int, e: int, slots: [] int) {
    program := *engine.program;
    found: bool;
    if engine.onepass.ok found = onepass_search(*engine.onepass, text, s, e, slots);
    else if backtrack_fits(program, e - s) found = backtrack_search(program, text, s, e, slots);
    else found = pike_search(program, text, s, e, true, slots);
    assert(found && slots[0] == s && slots[1] == e, "Regex: the matchers disagree on `%` in `%`: %..% vs %", engine.pattern, text, s, e, slots);
}

// Runs `body` for each match with `s` and `e` set, following find_all's rules for empty matches.
for_matches :: (the_engine: *Engine, text: string, body: Code) #expand {
    `engine := the_engine;
    pos := 0;
    last_end := -1;
    while pos <= text.count {
        `s, `e := find_span(engine, text, pos);
        if e < 0 break;
        if s == e && (s == last_end || !is_boundary(engine, text, s)) {
            // Not allowed here: look again one character on.
            pos = s + next_char_size(text, s);
            continue;
        }
        #insert body;
        last_end = e;
        pos = e;
    }
}

is_boundary :: (engine: *Engine, text: string, pos: int) -> bool {
    if pos == 0 || pos >= text.count return true;
    return (text[pos] & 0xC0) != 0x80;
}

next_char_size :: (text: string, pos: int) -> int {
    if pos >= text.count return 1;
    _, size := decode_utf8(text, pos);
    return size;
}

is_meta :: (c: u8) -> bool {
    if c == {
        case #char "\\"; #through;
        case #char "."; #through;
        case #char "+"; #through;
        case #char "*"; #through;
        case #char "?"; #through;
        case #char "("; #through;
        case #char ")"; #through;
        case #char "|"; #through;
        case #char "["; #through;
        case #char "]"; #through;
        case #char "{"; #through;
        case #char "}"; #through;
        case #char "^"; #through;
        case #char "$"; #through;
        case #char "#"; #through;
        case #char "&"; #through;
        case #char "-"; #through;
        case #char "~";
            return true;
    }
    return false;
}

expand_replacement :: (builder: *String_Builder, caps: Captures, replacement: string) {
    i := 0;
    while i < replacement.count {
        c := replacement[i];
        if c != #char "$" || i + 1 >= replacement.count {
            append(builder, c);
            i += 1;
            continue;
        }
        next := replacement[i + 1];
        if next == #char "$" {
            append(builder, #char "$");
            i += 2;
            continue;
        }
        name: string;
        if next == #char "{" {
            close := find_index_from_left(replacement, #char "}", i + 2);
            if close < 0 {
                append(builder, c);
                i += 1;
                continue;
            }
            name = slice(replacement, i + 2, close - i - 2);
            i = close + 1;
        } else {
            end := i + 1;
            while end < replacement.count && (is_word_byte(replacement[end])) end += 1;
            if end == i + 1 {
                append(builder, c);
                i += 1;
                continue;
            }
            name = slice(replacement, i + 1, end - i - 1);
            i = end;
        }
        number, is_number := parse_group_number(name);
        text: string;
        if is_number text = group(caps, number);
        else text = group(caps, name);
        append(builder, text);
    }
}

parse_group_number :: (name: string) -> int, bool {
    if name.count == 0 || name.count > 9 return 0, false;
    value := 0;
    for name {
        c := it;
        if c < #char "0" || c > #char "9" return 0, false;
        value = value * 10 + cast(int) (c - #char "0");
    }
    return value, true;
}
