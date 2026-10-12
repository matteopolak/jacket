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
    skips: [START_COUNT] Skip; // per unanchored start: how to jump over bytes that stay in it

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
    // The program has a Unicode \b, which needs to see whole characters: a non-ASCII byte's
    // column holds QUIT, and the search decodes the text there and steps through one of the
    // CHARACTER_COLUMNS copies of the byte's column instead, which says whether a word
    // character starts at the byte and whether one ends with it.
    unicode_word: bool;
    classes: int;
}

CHARACTER_COLUMNS :: 4;

DEAD :: 0;
UNKNOWN :: 0xFFFF_FFFF;
MATCH_BIT :: 0x8000_0000;
// A non-ASCII byte in a DFA with a Unicode \b: look at the characters around it (see unicode_word).
QUIT :: 0xFFFF_FFFE;

// What a byte looks like to a look: the edge of the text, \n, \r, a word byte, anything else.
CONTEXT_EDGE :: 0;
CONTEXT_LF :: 1;
CONTEXT_CR :: 2;
CONTEXT_WORD :: 3;
CONTEXT_OTHER :: 4;
CONTEXT_CHARACTER_WORD :: 5; // a non-ASCII word character: a word for Unicode looks only
START_COUNT :: 12;

START_INDEX :: inline (side: int, anchored: bool) -> int {
    return side * 2 + cast(int) anchored;
}

DEFAULT_STATE_LIMIT :: 10_000;

// An unanchored start state usually goes back to itself on most bytes: the search is waiting
// for the first byte of a match. A vector search jumps to the next place a match could start.
// Without looks, any position will do as a restart (the start state does not depend on the byte
// before), so the search can use bytes from further into the match, at a fixed distance from its
// start: two of them, if no single one is rare. When every byte near the start is common, a rare
// byte that every match must contain will do: find it, then walk back over the bytes a match can
// have before it to where the match could start. With looks, it can only jump over bytes that keep
// the DFA in its start state.
Skip :: struct {
    computed: bool;
    state: u32 = UNKNOWN; // the start state, or UNKNOWN when nothing is skipped
    kind: Skip_Kind;
    offset: int; // from a match's start to the byte searched for (the first of a pair)
    gap: int; // PAIR and TRIPLE: from the first byte to the second
    gap_3: int; // TRIPLE: from the first byte to the third
    count: int; // BYTES: how many of `bytes`
    bytes: [3] u8;
    set: Byte_Search.Byte_Set;
    second: Byte_Search.Byte_Set; // PAIR, TRIPLE: the second byte; REQUIRED: the bytes a match can have before `set`
    third: Byte_Search.Byte_Set; // TRIPLE: the third byte
    // BYTES, SET and PAIR without looks: the bytes a match can have at each of its first
    // `checks` positions, as bit sets, checked before the DFA is given a candidate.
    checks: int;
    check: [FINGERPRINT_DEPTH][32] u8;
    // Every match is exactly `checks` bytes, any of the checked bytes at each position: a
    // candidate that fits the checks is the match, and the DFA need not run.
    exact: bool;
}

Skip_Kind :: enum u8 {
    BYTES;
    SET;
    PAIR;
    TRIPLE;
    REQUIRED;
}

// A byte rarer than this is searched for alone.
SINGLE_SHARE :: 0.004;
// A search whose bytes make up more than this share of typical text stops too often to pay.
SKIP_SHARE_LIMIT :: 0.25;
// Positions into a match that the start analysis looks at.
FINGERPRINT_DEPTH :: 8;

dfa_init :: (d: *Dfa, program: *Program, reverse: bool, state_limit := DEFAULT_STATE_LIMIT) {
    d.program = program;
    d.reverse = reverse;
    d.match_all = reverse;
    d.unicode_word = program.has_unicode_word;
    d.classes = program.class_count;
    d.stride = program.class_count + 1;
    if d.unicode_word d.stride += program.class_count * CHARACTER_COLUMNS;
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
    for side: 0..5 {
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
    if d.gave_up return false;
    for side: 0..5 dfa_skip(d, START_INDEX(side, false));
    return true;
}

// A copy holding only the tables (for a DFA built completely).
dfa_freeze :: (d: *Dfa) -> Dfa {
    out: Dfa;
    out.reverse = d.reverse;
    out.match_all = d.match_all;
    out.unicode_word = d.unicode_word;
    out.classes = d.classes;
    out.stride = d.stride;
    out.byte_class = d.byte_class;
    trans := NewArray(d.rows.count, u32);
    for d.rows trans[it_index] = it;
    out.trans = trans;
    out.starts = d.starts;
    out.skips = d.skips;
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
    side := CONTEXT_EDGE;
    if start > 0 {
        side = context_of(text[start - 1]);
        if d.unicode_word && text[start - 1] >= 0x80 side = ifx word_char_before(text, start) then CONTEXT_CHARACTER_WORD else CONTEXT_OTHER;
    }
    s := dfa_start(d, side, anchored);
    if s == UNKNOWN return -1, true;
    skip: *Skip;
    skip_state := UNKNOWN;
    if !anchored {
        skip = dfa_skip(d, START_INDEX(side, false));
        skip_state = skip.state;
    }
    last := -1;
    i := start;
    count := text.count;
    fence := -1; // REQUIRED skips: the last required byte walked back from
    while i < count {
        if s == skip_state {
            i = skip_to(skip, text, i, *fence);
            if i < 0 {
                i = count;
                break;
            }
            if skip.exact return i + skip.checks, false;
        }
        n := d.trans[s + d.byte_class[text[i]]];
        if n == DEAD return last, false;
        if n >= MATCH_BIT {
            if n == UNKNOWN {
                n = dfa_compute(d, s, d.byte_class[text[i]]);
                if d.gave_up return -1, true;
            }
            if n == QUIT {
                n = dfa_step_character(d, s, text, i);
                if d.gave_up return -1, true;
            }
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
    side := CONTEXT_EDGE;
    if end < text.count {
        side = context_of(text[end]);
        if d.unicode_word && text[end] >= 0x80 side = ifx word_char_after(text, end) then CONTEXT_CHARACTER_WORD else CONTEXT_OTHER;
    }
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
        if n == QUIT {
            n = dfa_step_character(d, s, text, i - 1);
            if d.gave_up return -1, true;
        }
        if n & MATCH_BIT last = i;
        n &= ~cast(u32) MATCH_BIT;
        if n == DEAD || i <= bound break;
        s = n;
        i -= 1;
    }
    return last, false;
}

// The step over non-ASCII byte text[at] in a DFA with a Unicode \b, through the column that
// knows whether a word character starts at `at` and whether one ends with it.
dfa_step_character :: (d: *Dfa, s: u32, text: string, at: int) -> u32 {
    starts := word_char_after(text, at);
    ends := word_char_before(text, at + 1);
    column := d.classes * (1 + cast(int) starts + 2 * cast(int) ends) + d.byte_class[text[at]];
    n := d.trans[s + cast(u32) column];
    if n == UNKNOWN n = dfa_compute(d, s, column);
    return n;
}

// The skip for an unanchored start (computing it the first time, in a lazy DFA).
dfa_skip :: (d: *Dfa, index: int) -> *Skip {
    skip := *d.skips[index];
    if skip.computed || !d.program return skip;
    skip.computed = true;
    s := d.starts[index];
    if s == UNKNOWN || s == DEAD || d.reverse return skip;
    for cls: 0..d.stride - 2 {
        if d.rows[s + cast(u32) cls] == UNKNOWN dfa_compute(d, s, cls);
        if d.gave_up return skip;
    }
    // A start state that matches, or leaves on every byte, has nothing to skip.
    stays := false;
    for b: 0..255 if d.rows[s + d.byte_class[b]] == s stays = true;
    if !stays return skip;

    sets: [..][256] bool;
    sets.allocator = temp;
    if d.program.has_looks {
        leaving: [256] bool;
        for b: 0..255 leaving[b] = d.rows[s + d.byte_class[b]] != s;
        array_add(*sets, leaving);
    } else {
        sets, skip.exact = fingerprint(d.program);
    }
    if !sets.count return skip;

    // The rarest offset, and the rarest pair of offsets.
    best := -1;
    best_share: float64 = 2;
    for sets {
        share := byte_share(it);
        if share < best_share best, best_share = it_index, share;
    }
    pair_a, pair_b := -1, -1;
    pair_share: float64 = 2;
    for a: 0..sets.count - 1 for b : a + 1..sets.count - 1 {
        share := byte_share(sets[a]) * byte_share(sets[b]);
        if share < pair_share pair_a, pair_b, pair_share = a, b, share;
    }
    triple_a, triple_b, triple_c := -1, -1, -1;
    triple_share: float64 = 2;
    for a: 0..sets.count - 1 for b : a + 1..sets.count - 1 for c : b + 1..sets.count - 1 {
        share := byte_share(sets[a]) * byte_share(sets[b]) * byte_share(sets[c]);
        if share < triple_share triple_a, triple_b, triple_c, triple_share = a, b, c, share;
    }
    // One rare byte is the fastest search; failing that, more bytes stop less often.
    use_triple := triple_a >= 0 && best_share > SINGLE_SHARE && triple_share < pair_share * 0.5;
    use_pair := !use_triple && pair_a >= 0 && best_share > SINGLE_SHARE && pair_share < best_share * 0.25;
    if !use_pair && !use_triple && best_share > SKIP_SHARE_LIMIT {
        skip.exact = false;
        if d.program.has_looks return skip;
        required, before, ok := required_bytes(d.program);
        if !ok return skip;
        skip.state = s;
        skip.kind = .REQUIRED;
        skip.set = make_set(required);
        skip.second = make_set(before);
        return skip;
    }

    skip.state = s;
    if !d.program.has_looks {
        skip.checks = sets.count;
        for set, k: sets for set if it skip.check[k][it_index >> 3] |= cast(u8) (1 << (it_index & 7));
    }
    if use_triple {
        skip.kind = .TRIPLE;
        skip.offset = triple_a;
        skip.gap = triple_b - triple_a;
        skip.gap_3 = triple_c - triple_a;
        skip.set = make_set(sets[triple_a]);
        skip.second = make_set(sets[triple_b]);
        skip.third = make_set(sets[triple_c]);
        return skip;
    }
    if use_pair {
        skip.kind = .PAIR;
        skip.offset = pair_a;
        skip.gap = pair_b - pair_a;
        skip.set = make_set(sets[pair_a]);
        skip.second = make_set(sets[pair_b]);
        return skip;
    }
    skip.offset = best;
    members: [..] u8;
    members.allocator = temp;
    for sets[best] if it array_add(*members, cast(u8) it_index);
    if members.count <= 3 {
        skip.kind = .BYTES;
        skip.count = members.count;
        for members skip.bytes[it_index] = it;
    } else {
        skip.kind = .SET;
        skip.set = Byte_Search.make_byte_set(members);
    }
    return skip;
}

// The next position at or after `i` where a match could start, or -1.
skip_to :: inline (skip: *Skip, text: string, i: int, fence: *int) -> int {
    if skip.kind == .REQUIRED {
        found := Byte_Search.find(*skip.set, text, i);
        if found < 0 return -1;
        // Walk back once per required byte: a second visit would go over the same bytes again.
        if found == fence.* return i;
        fence.* = found;
        at := found;
        while at > i && Byte_Search.contains(*skip.second, text[at - 1]) at -= 1;
        return at;
    }
    at := i + skip.offset;
    while true {
        found: int = ---;
        if skip.kind == {
            case .BYTES;
                if skip.count == 1 found = Byte_Search.find(text, skip.bytes[0], at);
                else if skip.count == 2 found = Byte_Search.find_any(text, skip.bytes[0], skip.bytes[1], at);
                else found = Byte_Search.find_any(text, skip.bytes[0], skip.bytes[1], skip.bytes[2], at);
            case .SET;
                found = Byte_Search.find(*skip.set, text, at);
            case .PAIR;
                found = Byte_Search.find_pair(*skip.set, *skip.second, skip.gap, text, at);
            case;
                found = Byte_Search.find_triple(*skip.set, *skip.second, *skip.third, skip.gap, skip.gap_3, text, at);
        }
        if found < 0 return -1;
        candidate := found - skip.offset;
        if fits_checks(skip, text, candidate) return candidate;
        at = found + 1;
    }
    return -1;
}

// Whether the bytes from `at` can begin a match, as far as the skip's checks tell.
fits_checks :: inline (skip: *Skip, text: string, at: int) -> bool {
    if !skip.checks return true;
    // Every match is at least `checks` bytes long; none fits past here.
    if at + skip.checks > text.count return false;
    for k: 0..skip.checks - 1 {
        b := text[at + k];
        if !(skip.check[k][b >> 3] & (1 << (b & 7))) return false;
    }
    return true;
}

// For each of the first few positions of a match, the bytes it can have there, up to the
// shortest match's length (looks are taken to hold, which only adds bytes).
fingerprint :: (program: *Program) -> sets: [..][256] bool, exact: bool {
    sets: [..][256] bool;
    // Exact while the program is a single chain: one byte-reading instruction per position,
    // all of whose bytes lead to the same place, ending in a match.
    exact := true;
    sets.allocator = temp;
    insts := program.insts;
    mark := NewArray(insts.count, u32,, temp);
    generation: u32 = 0;
    current: [..] u32;
    current.allocator = temp;
    next: [..] u32;
    next.allocator = temp;
    stack: [..] u32;
    stack.allocator = temp;
    array_add(*current, program.start);
    for depth: 0..FINGERPRINT_DEPTH - 1 {
        generation += 1;
        set: [256] bool;
        next.count = 0;
        // Everything reachable without a byte; a reachable match ends the fingerprint.
        for current array_add(*stack, it);
        consumers: [..] u32;
        consumers.allocator = temp;
        while stack.count {
            at := pop(*stack);
            if mark[at] == generation continue;
            mark[at] = generation;
            inst := *insts[at];
            if inst.kind == {
                case .MATCH;
                    return sets, exact && consumers.count == 0 && stack.count == 0;
                case .RANGES; array_add(*consumers, at);
                case .SPLIT;
                    array_add(*stack, inst.alt);
                    array_add(*stack, inst.next);
                case .SAVE; #through;
                case .LOOK;
                    array_add(*stack, inst.next);
            }
        }
        generation += 1;
        if consumers.count != 1 exact = false;
        for consumers {
            inst := *insts[it];
            for t: 0..inst.alt - 1 {
                tr := *program.transitions[inst.next + t];
                for b: cast(int) tr.lo..cast(int) tr.hi set[b] = true;
                if mark[tr.next] != generation {
                    mark[tr.next] = generation;
                    array_add(*next, tr.next);
                }
            }
        }
        if !next.count return sets, false;
        if next.count != 1 exact = false;
        array_add(*sets, set);
        current.count = 0;
        for next array_add(*current, it);
    }
    return sets, false;
}

// A rare set of bytes that every match contains, and the bytes a match can have before the
// first of them. Tries each ASCII byte together with its other case.
required_bytes :: (program: *Program) -> required: [256] bool, before: [256] bool, ok: bool {
    best: [256] bool;
    best_before: [256] bool;
    best_share: float64 = REQUIRED_SHARE_LIMIT;
    found := false;
    for b: 0..127 {
        required: [256] bool;
        required[b] = true;
        if b >= #char "a" && b <= #char "z" required[b - 32] = true;
        if b >= #char "A" && b <= #char "Z" continue; // tried with its lower case
        share := byte_share(required);
        if share >= best_share continue;
        before, ok := bytes_before(program, required);
        if !ok continue;
        best, best_before, best_share, found = required, before, share, true;
    }
    return best, best_before, found;
}

// A required byte rarer than this pays for walking back over the bytes before it.
REQUIRED_SHARE_LIMIT :: 0.03;

// The bytes a match can consume before its first byte from `required`; not ok if a match can
// do without one.
bytes_before :: (program: *Program, required: [256] bool) -> [256] bool, ok: bool {
    before: [256] bool;
    insts := program.insts;
    seen := NewArray(insts.count, bool,, temp);
    stack: [..] u32;
    stack.allocator = temp;
    array_add(*stack, program.start);
    while stack.count {
        at := pop(*stack);
        if seen[at] continue;
        seen[at] = true;
        inst := *insts[at];
        if inst.kind == {
            case .MATCH; return before, false;
            case .RANGES;
                for t: 0..inst.alt - 1 {
                    tr := *program.transitions[inst.next + t];
                    other := false;
                    for c: cast(int) tr.lo..cast(int) tr.hi if !required[c] {
                        before[c] = true;
                        other = true;
                    }
                    if other array_add(*stack, tr.next);
                }
            case .SPLIT;
                array_add(*stack, inst.alt);
                array_add(*stack, inst.next);
            case .SAVE; #through;
            case .LOOK;
                array_add(*stack, inst.next);
        }
    }
    return before, true;
}

make_set :: (members: [256] bool) -> Byte_Search.Byte_Set {
    bytes: [..] u8;
    bytes.allocator = temp;
    for members if it array_add(*bytes, cast(u8) it_index);
    return Byte_Search.make_byte_set(bytes);
}

// A rough share of typical text (mostly English, some UTF-8) that falls in the set.
byte_share :: (members: [256] bool) -> float64 {
    total: float64 = 0;
    for members if it total += BYTE_SHARE[it_index];
    return total;
}

BYTE_SHARE :: #run -> [256] float64 {
    share: [256] float64;
    for b: 0..255 {
        weight: float64 = 0.0005;
        if b >= 0x80 weight = 0.004;
        if b >= #char "A" && b <= #char "Z" weight = 0.003;
        if b >= #char "0" && b <= #char "9" weight = 0.003;
        if b == #char "," || b == #char "." || b == #char "\n" weight = 0.01;
        if b == #char " " weight = 0.15;
        share[b] = weight;
    }
    // Lower-case letters, most common first, with rough shares in percent.
    letters :: "etaoinshrdlcumwfgypbvkjxqz";
    percent :: float64.[10, 7, 6.5, 6, 5.7, 5.6, 5, 5, 4.8, 3.5, 3.2, 2.3, 2.3, 2, 1.9, 1.8, 1.6, 1.6, 1.5, 1.2, 0.8, 0.6, 0.15, 0.15, 0.1, 0.07];
    for letters share[it] = percent[it_index] / 100;
    return share;
};

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
    // Next to non-ASCII bytes, the search passes the word-ness of whole characters, which only
    // Unicode looks count.
    unicode := look >= .WORD && look <= .WORD_END_HALF;
    before := left == CONTEXT_WORD || (unicode && left == CONTEXT_CHARACTER_WORD);
    after := right == CONTEXT_WORD || (unicode && right == CONTEXT_CHARACTER_WORD);
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
    b := program.class_byte[cls % d.classes];
    here := ifx end_of_text then CONTEXT_EDGE else context_of(b);
    // What the byte looks like to the position before it, and to the one after it: the same,
    // except for a non-ASCII byte seen through a character column.
    before_it, after_it := here, here;
    if d.unicode_word && !end_of_text && b >= 0x80 {
        if cls < d.classes {
            d.rows[s + cast(u32) cls] = QUIT;
            return QUIT;
        }
        variant := cls / d.classes - 1;
        before_it = ifx variant & 1 then CONTEXT_CHARACTER_WORD else CONTEXT_OTHER; // a word character starts here
        after_it = ifx variant & 2 then CONTEXT_CHARACTER_WORD else CONTEXT_OTHER; // a word character ends here
    }
    behind := cast(int) d.left[state];
    left, right := behind, before_it;
    if d.reverse left, right = after_it, behind;
    here = ifx d.reverse then before_it else after_it;

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
