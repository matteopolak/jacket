// A DFA over a Program, built lazily while searching (or all at once, at compile time). Each
// state is an ordered list of the program's waiting instructions; the order is the pattern's
// preference, so the DFA finds the same leftmost-first match the Pike VM does, without
// captures. A forward DFA finds where a match ends; a reverse DFA, run back from there, finds
// where it starts.
//
// Looks need to see the bytes on both sides of a position, so they are resolved one byte late:
// a state keeps unresolved looks together with the kind of byte that led to it, and the next
// transition, which knows the following byte, settles them. For the same reason a transition
// says whether a match ended just before its byte.

Dfa :: struct {
    program: *Program; // null for a DFA that was built completely at compile time
    reverse: bool;
    match_all: bool; // keep going after a match (the reverse DFA wants the leftmost start)
    stride: int; // byte classes + 1; the last column is the end of the text
    byte_class: [256] u8;
    trans: [] u32; // [state * stride + class]; states are stored premultiplied by stride
    starts: [START_COUNT] u32; // by START_INDEX(side, anchored)

    // Only while building:
    rows: [..] u32;
    lists: [..] u32; // every state's instructions, back to back
    list_start: [..] u32; // per state
    list_count: [..] u32;
    left: [..] u8; // per state: the kind of byte before it (for unresolved looks)
    known: Table(string, u32);
    resolved: [..] u32;
    next: [..] u32;
    stack: [..] u32;
    mark_a: [] u32;
    mark_b: [] u32;
    generation: u32;
    state_limit: int;
    gave_up: bool;
    quit_non_ascii: bool; // the program has a Unicode \b, which needs to see whole characters
}

DEAD :: 0;
UNKNOWN :: 0xFFFF_FFFF;
MATCH_BIT :: 0x8000_0000;
// A byte the DFA can't handle (non-ASCII next to a Unicode \b): this search needs another matcher.
QUIT :: 0xFFFF_FFFE;

// What a byte looks like to a look: the edge of the text, \n, \r, a word byte, anything else.
CONTEXT_EDGE :: 0;
CONTEXT_LF :: 1;
CONTEXT_CR :: 2;
CONTEXT_WORD :: 3;
CONTEXT_OTHER :: 4;
START_COUNT :: 10;

START_INDEX :: inline (side: int, anchored: bool) -> int {
    return side * 2 + cast(int) anchored;
}

DEFAULT_STATE_LIMIT :: 10_000;

dfa_init :: (d: *Dfa, program: *Program, reverse: bool, state_limit := DEFAULT_STATE_LIMIT) {
    d.program = program;
    d.reverse = reverse;
    d.match_all = reverse;
    d.quit_non_ascii = program.has_unicode_word;
    d.stride = program.class_count + 1;
    d.byte_class = program.byte_class;
    d.state_limit = state_limit;
    for *d.starts it.* = UNKNOWN;
    init(*d.known, 64);
    d.mark_a = NewArray(program.insts.count, u32);
    d.mark_b = NewArray(program.insts.count, u32);
    // State 0 is dead: no instructions, every transition back to itself.
    add_state(d, 0, .[]);
    for 0..d.stride - 1 d.rows[it] = DEAD;
    d.trans = d.rows;
}

dfa_free :: (d: *Dfa) {
    if !d.program return;
    array_free(d.rows);
    array_free(d.lists);
    array_free(d.list_start);
    array_free(d.list_count);
    array_free(d.left);
    for d.known free(it_index);
    deinit(*d.known);
    array_free(d.resolved);
    array_free(d.next);
    array_free(d.stack);
    array_free(d.mark_a);
    array_free(d.mark_b);
}

// Builds every state, so that no search ever needs the program. False if there are more than
// the limit.
dfa_build_all :: (d: *Dfa) -> bool {
    for side: 0..4 {
        dfa_start(d, side, false);
        dfa_start(d, side, true);
    }
    i := 0;
    while i < d.list_start.count {
        for cls: 0..d.stride - 1 {
            at := i * d.stride + cls;
            if d.rows[at] == UNKNOWN dfa_compute(d, cast(u32) (i * d.stride), cls);
            if d.gave_up return false;
        }
        i += 1;
    }
    return !d.gave_up;
}

// A copy holding only the tables (for a DFA built completely).
dfa_freeze :: (d: *Dfa) -> Dfa {
    out: Dfa;
    out.reverse = d.reverse;
    out.match_all = d.match_all;
    out.quit_non_ascii = d.quit_non_ascii;
    out.stride = d.stride;
    out.byte_class = d.byte_class;
    trans := NewArray(d.rows.count, u32);
    for d.rows trans[it_index] = it;
    out.trans = trans;
    out.starts = d.starts;
    return out;
}

dfa_state_count :: (d: *Dfa) -> int {
    return d.trans.count / d.stride;
}

context_of :: inline (b: u8) -> int {
    if b == #char "\n" return CONTEXT_LF;
    if b == #char "\r" return CONTEXT_CR;
    if is_word_byte(b) return CONTEXT_WORD;
    return CONTEXT_OTHER;
}

dfa_start :: (d: *Dfa, side: int, anchored: bool) -> u32 {
    index := START_INDEX(side, anchored);
    s := d.starts[index];
    if s != UNKNOWN || !d.program return s;
    d.next.count = 0;
    array_add(*d.next, ifx anchored then d.program.start else d.program.start_unanchored);
    s = state_for(d, side);
    if d.gave_up return UNKNOWN;
    d.starts[index] = s;
    return s;
}

// The end of the leftmost-first match that starts at or after `start` (and exactly at `start`
// if `anchored`), or -1. With `earliest`, stops at the first position where any match ends,
// which answers "is there a match" sooner. `gave_up` means the DFA grew past its limit and the
// caller has to use another matcher.
dfa_find_end :: (d: *Dfa, text: string, start: int, anchored: bool, earliest: bool) -> end: int, gave_up: bool {
    if d.quit_non_ascii && start > 0 && text[start - 1] >= 0x80 return -1, true;
    side := ifx start > 0 then context_of(text[start - 1]) else CONTEXT_EDGE;
    s := dfa_start(d, side, anchored);
    if s == UNKNOWN return -1, true;
    last := -1;
    i := start;
    count := text.count;
    while i < count {
        n := d.trans[s + d.byte_class[text[i]]];
        if n == DEAD return last, false;
        if n >= MATCH_BIT {
            if n == UNKNOWN {
                n = dfa_compute(d, s, d.byte_class[text[i]]);
                if d.gave_up return -1, true;
            }
            if n == QUIT return -1, true;
            if n & MATCH_BIT {
                last = i;
                if earliest return last, false;
                n &= ~cast(u32) MATCH_BIT;
            }
            if n == DEAD return last, false;
        }
        s = n;
        i += 1;
    }
    n := d.trans[s + cast(u32) d.stride - 1];
    if n == UNKNOWN {
        n = dfa_compute(d, s, d.stride - 1);
        if d.gave_up return -1, true;
    }
    if n & MATCH_BIT last = count;
    return last, false;
}

// Run back from `end` (a reverse DFA): the leftmost position >= `bound` where a match ending at
// `end` can start, or -1.
dfa_find_start :: (d: *Dfa, text: string, bound: int, end: int) -> start: int, gave_up: bool {
    if d.quit_non_ascii && end < text.count && text[end] >= 0x80 return -1, true;
    side := ifx end < text.count then context_of(text[end]) else CONTEXT_EDGE;
    s := dfa_start(d, side, true);
    if s == UNKNOWN return -1, true;
    last := -1;
    i := end;
    while true {
        // At `bound` the step only settles looks and reports a match; past the start of the text
        // that is the end-of-text column.
        cls: int = ifx i > 0 then d.byte_class[text[i - 1]] else d.stride - 1;
        n := d.trans[s + cast(u32) cls];
        if n == UNKNOWN {
            n = dfa_compute(d, s, cls);
            if d.gave_up return -1, true;
        }
        if n == QUIT return -1, true;
        if n & MATCH_BIT last = i;
        n &= ~cast(u32) MATCH_BIT;
        if n == DEAD || i <= bound break;
        s = n;
        i -= 1;
    }
    return last, false;
}

#scope_file

look_holds_between :: (look: Look, left: int, right: int) -> bool {
    if look == {
        case .START_TEXT; return left == CONTEXT_EDGE;
        case .END_TEXT; return right == CONTEXT_EDGE;
        case .START_LINE; return left == CONTEXT_EDGE || left == CONTEXT_LF;
        case .END_LINE; return right == CONTEXT_EDGE || right == CONTEXT_LF;
        case .START_LINE_CRLF; return left == CONTEXT_EDGE || left == CONTEXT_LF || (left == CONTEXT_CR && right != CONTEXT_LF);
        case .END_LINE_CRLF; return right == CONTEXT_EDGE || right == CONTEXT_CR || (right == CONTEXT_LF && left != CONTEXT_CR);
    }
    // Unicode word looks are only used on ASCII text by the DFA, where they agree with these.
    before := left == CONTEXT_WORD;
    after := right == CONTEXT_WORD;
    if look == {
        case .WORD; #through;
        case .WORD_ASCII; return before != after;
        case .NOT_WORD; #through;
        case .NOT_WORD_ASCII; return before == after;
        case .WORD_START; #through;
        case .WORD_START_ASCII; return !before && after;
        case .WORD_END; #through;
        case .WORD_END_ASCII; return before && !after;
        case .WORD_START_HALF; #through;
        case .WORD_START_HALF_ASCII; return !before;
        case .WORD_END_HALF; #through;
        case .WORD_END_HALF_ASCII; return !after;
    }
    return false;
}

add_state :: (d: *Dfa, side: u8, list: [] u32) -> u32 {
    array_add(*d.list_start, cast(u32) d.lists.count);
    array_add(*d.list_count, cast(u32) list.count);
    array_add(*d.left, side);
    for list array_add(*d.lists, it);
    old := d.rows.count;
    array_resize(*d.rows, old + d.stride, initialize = false);
    for old..d.rows.count - 1 d.rows[it] = UNKNOWN;
    d.trans = d.rows;
    return cast(u32) old;
}

// The state whose threads continue at the instructions in d.next, reached through a byte of
// kind `side`.
state_for :: (d: *Dfa, side: int) -> u32 {
    if d.next.count == 0 return DEAD;
    // Without looks, the byte before does not matter.
    if !d.program.has_looks side = 0;
    key := alloc_string(1 + d.next.count * 4);
    key[0] = cast(u8) side;
    memcpy(key.data + 1, d.next.data, d.next.count * 4);
    found, s := table_find(*d.known, key);
    if found {
        free(key);
        return s;
    }
    if d.list_start.count >= d.state_limit {
        free(key);
        d.gave_up = true;
        return DEAD;
    }
    s = add_state(d, cast(u8) side, d.next);
    table_add(*d.known, key, s);
    return s;
}

dfa_compute :: (d: *Dfa, s: u32, cls: int) -> u32 {
    program := d.program;
    insts := program.insts;
    state := s / cast(u32) d.stride;
    end_of_text := cls == d.stride - 1;
    b := program.class_byte[cls];
    here := ifx end_of_text then CONTEXT_EDGE else context_of(b);
    behind := cast(int) d.left[state];
    left, right := behind, here;
    if d.reverse left, right = here, behind;
    if d.quit_non_ascii && !end_of_text && b >= 0x80 {
        d.rows[s + cast(u32) cls] = QUIT;
        return QUIT;
    }

    // A state holds where its threads continue; follow them now that the bytes on both sides
    // are known, in priority order and visiting each instruction once, as the Pike VM does.
    d.generation += 1;
    d.resolved.count = 0;
    first := d.list_start[state];
    for k: 0..d.list_count[state] - 1 {
        array_add(*d.stack, d.lists[first + k]);
        while d.stack.count {
            at := pop(*d.stack);
            if d.mark_a[at] == d.generation continue;
            d.mark_a[at] = d.generation;
            inst := *insts[at];
            if inst.kind == {
                case .MATCH; #through;
                case .RANGES;
                    array_add(*d.resolved, at);
                case .LOOK;
                    if look_holds_between(inst.look, left, right) array_add(*d.stack, inst.next);
                case .SPLIT;
                    array_add(*d.stack, inst.alt);
                    array_add(*d.stack, inst.next);
                case .SAVE;
                    array_add(*d.stack, inst.next);
            }
        }
    }

    // Step over the byte.
    matched := false;
    d.next.count = 0;
    for d.resolved {
        inst := *insts[it];
        if inst.kind == .MATCH {
            matched = true;
            if !d.match_all break;
            continue;
        }
        if end_of_text continue;
        for t: 0..inst.alt - 1 {
            tr := *program.transitions[inst.next + t];
            if b < tr.lo || b > tr.hi continue;
            if d.mark_b[tr.next] == d.generation continue;
            d.mark_b[tr.next] = d.generation;
            array_add(*d.next, tr.next);
        }
    }
    target: u32 = DEAD;
    if !end_of_text target = state_for(d, here);
    if d.gave_up return UNKNOWN;
    if matched target |= MATCH_BIT;
    d.rows[s + cast(u32) cls] = target;
    d.trans = d.rows;
    return target;
}
