// Regex: regular expressions with the syntax and guarantees of Rust's `regex` crate, full
// Unicode support, and patterns that can be compiled while your program compiles. A jaic
// extension, not official Jai: other Jai compilers have no such module, and it may change
// between jaic releases. See docs/extensions/regex.md.
//
//     Regex :: #import "Extensions/Regex";
//
//     // Compiled while the program compiles: a bad pattern is a compile error, and named
//     // groups become typed fields (an integer for a run of digits, an enum for a few words).
//     Date :: Regex.Static("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})");
//     date, ok := Regex.parse(Date, "2026-10-11");  // date.year is a u16, date.month a u8
//
//     // Compiled at run time, for a pattern only known then.
//     re, ok, error := Regex.compile(user_pattern);
//     for Regex.find_all(*re, text) print("% ", it.text);
//
// Matching takes time linear in the text for every pattern: there are no backreferences and no
// look-around, and the engines are automata, never an exponential backtracker.

// Pattern flags, also settable inside the pattern: (?i) (?m) (?s) (?U) (?x) (?R) (?-u).
Options :: struct {
    case_insensitive: bool; // i: letters match either case (Unicode simple case folding)
    multi_line: bool; // m: ^ and $ match at line breaks, not just the ends of the text
    dot_all: bool; // s: . also matches \n
    swap_greed: bool; // U: x* is lazy and x*? greedy
    ignore_space: bool; // x: whitespace is ignored and # starts a comment
    crlf: bool; // R: with m, \r\n also ends a line, and . does not match \r
    unicode := true; // u: classes and \b are Unicode-aware; off, they are ASCII and match bytes
}

// A compiled pattern. Copies share the compiled program and the search cache, which is not safe
// to use from two threads at once: give each thread its own compile.
Regex :: struct {
    pattern: string;
    group_count: int; // capture groups, not counting group 0 (the whole match)
    names: [] string; // names[i] is group i's name, or ""
    engine: *Engine;
}

Error :: struct {
    message: string; // `unclosed group: expected `)``
    offset: int; // byte offset in the pattern where the problem is
}

Match :: struct {
    start: int;
    end: int;
    text: string; // the matched text, a view into the searched string
}

// Where each group matched. Group 0 is the whole match; a group that did not take part has
// start -1.
Captures :: struct {
    text: string; // the whole searched string
    slots: [] int; // start and end of each group, in pairs
    names: [] string;
}

compile :: (pattern: string, options := Options.{}) -> Regex, ok: bool, Error {
    re: Regex;
    re.pattern = pattern;
    engine, ok, error := build_engine(pattern, options);
    if !ok return re, false, error;

    re.engine = engine;
    re.group_count = engine.program.slot_count / 2 - 1;
    re.names = engine.program.names;

    return re, true, .{};
}

free_regex :: (re: *Regex) {
    if !re.engine return;

    dfa_free(*re.engine.forward);
    dfa_free(*re.engine.backward);
    onepass_free(*re.engine.onepass);
    free(re.engine);
    re.engine = null;
}

// Whether captures for this pattern are found in a single scan (see docs/extensions/regex.md).
is_one_pass :: (re: *Regex) -> bool {
    return re.engine.onepass.ok;
}

is_one_pass :: ($R: Type) -> bool #modify {
    return is_static(R);
} {
    return static_engine_of(R.pattern, false).onepass.ok;
}

is_match :: (re: *Regex, text: string, start := 0) -> bool {
    _, end := find_span(re.engine, text, start, earliest = true);
    return end >= 0;
}

// The leftmost match at or after byte `start`.
find :: (re: *Regex, text: string, start := 0) -> Match, bool {
    s, e := find_span(re.engine, text, start);
    if e < 0 return .{}, false;
    return .{ s, e, slice(text, s, e - s) }, true;
}

// Every match, left to right, not overlapping. An empty match right where the previous match
// ended is skipped, and in Unicode mode empty matches never split a UTF-8 sequence.
find_all :: (re: *Regex, text: string) -> [..] Match {
    out: [..] Match;
    for_matches(re.engine, text, #code array_add(*out, .{ s, e, slice(text, s, e - s) }));
    return out;
}

captures :: (re: *Regex, text: string, start := 0) -> Captures, bool {
    s, e := find_span(re.engine, text, start);
    if e < 0 return .{}, false;
    return captures_of(re.engine, text, s, e), true;
}

captures_all :: (re: *Regex, text: string) -> [..] Captures {
    out: [..] Captures;
    for_matches(re.engine, text, #code array_add(*out, captures_of(engine, text, s, e)));
    return out;
}

// Group `index`'s text, and whether it took part in the match.
group :: (caps: Captures, index: int) -> string, bool {
    if index < 0 || 2 * index + 1 >= caps.slots.count return "", false;

    s := caps.slots[2 * index];
    e := caps.slots[2 * index + 1];

    if s < 0 return "", false;

    return slice(caps.text, s, e - s), true;
}

group :: (caps: Captures, name: string) -> string, bool {
    for caps.names if it.count && it == name return group(caps, it_index);
    return "", false;
}

// Group `index`'s byte range, or -1, -1.
span :: (caps: Captures, index: int) -> start: int, end: int {
    if index < 0 || 2 * index + 1 >= caps.slots.count return -1, -1;
    return caps.slots[2 * index], caps.slots[2 * index + 1];
}

// Replaces the first match. In `replacement`, `$1` / `${1}` is a group's text, `$name` /
// `${name}` a named group's, and `$$` a dollar sign; a group that did not match (or does not
// exist) gives "". `$name` takes the longest run of letters, digits and `_`, so write
// `${1}st` rather than `$1st`.
replace :: (re: *Regex, text: string, replacement: string) -> string {
    return replace_n(re, text, replacement, 1);
}

replace_all :: (re: *Regex, text: string, replacement: string) -> string {
    return replace_n(re, text, replacement, -1);
}

// Replaces the first `limit` matches (all of them when limit < 0).
replace_n :: (re: *Regex, text: string, replacement: string, limit: int) -> string {
    builder: String_Builder;
    last := 0;
    count := 0;
    literal := find_index_from_left(replacement, #char "$") < 0;

    for_matches(re.engine, text, #code {
        if limit >= 0 && count >= limit break;

        append(*builder, slice(text, last, s - last));

        if literal {
            append(*builder, replacement);
        } else {
            caps := captures_of(engine, text, s, e);
            expand_replacement(*builder, caps, replacement);
        }

        last = e;
        count += 1;
    });
    append(*builder, slice(text, last, text.count - last));

    return builder_to_string(*builder);
}

// The pieces of `text` between matches.
split :: (re: *Regex, text: string) -> [..] string {
    out: [..] string;
    last := 0;

    for_matches(re.engine, text, #code {
        array_add(*out, slice(text, last, s - last));
        last = e;
    });
    array_add(*out, slice(text, last, text.count - last));

    return out;
}

// `text` with every regex metacharacter escaped, so it matches literally.
escape :: (text: string) -> string {
    builder: String_Builder;

    for text {
        c := it;
        if is_meta(c) append(*builder, #char "\\");
        append(*builder, c);
    }

    return builder_to_string(*builder);
}

// A regex compiled while your program compiles: a type, made from a constant pattern.
//
//     Date :: Regex.Static("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})");
//     date, ok := Regex.parse(Date, "2026-10-11");  // a Date.Groups: date.year is a u16
//
// A bad pattern is a compile error, and the automata are built at compile time (big Unicode
// classes excepted: those are built on first use), so searching needs no setup. The same
// procedures as for a Regex take it in place of `*re`, and parse, parse_into and captures_into
// only take one of these. For a pattern known only at run time, use compile.
Static :: struct(pattern: string) {

    // The named groups, as typed fields: `[0-9]{4}` is a u16, `-?\d+` an s64, `\d+\.\d+` a
    // float64, `cat|dog` an enum (Groups.Kind for a group named kind) with members CAT and DOG,
    // and anything else a string. Groups that might not take part in a match get a flag in `has`.
    Groups :: struct {
        match: Match;
        #insert -> string {
            return groups_members(pattern);
        }
    }

    #insert -> string {
        return check_pattern(pattern);
    }
}

is_match :: ($R: Type, text: string, start := 0) -> bool #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return is_match(*re, text, start);
}

find :: ($R: Type, text: string, start := 0) -> Match, bool #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    match, ok := find(*re, text, start);
    return match, ok;
}

find_all :: ($R: Type, text: string) -> [..] Match #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return find_all(*re, text);
}

// The leftmost match at or after `start`, with its named groups parsed. False if there is no
// match, or a number does not fit its field.
captures :: ($R: Type, text: string, start := 0) -> R.Groups, bool #modify {
    return is_static(R);
} {
    out: R.Groups;
    engine := static_engine_of(R.pattern, false);
    s, e := find_span(engine, text, start);

    if e < 0 return out, false;

    ok := fill_groups(R, engine, text, s, e, *out);

    return out, ok;
}

captures_all :: ($R: Type, text: string) -> [..] R.Groups #modify {
    return is_static(R);
} {
    out: [..] R.Groups;

    for_matches(static_engine_of(R.pattern, false), text, #code {
        groups: R.Groups;
        if fill_groups(R, engine, text, s, e, *groups) array_add(*out, groups);
    });

    return out;
}

// The whole of `text` against the pattern, with its named groups parsed.
parse :: ($R: Type, text: string) -> R.Groups, bool #modify {
    return is_static(R);
} {
    out: R.Groups;
    engine := static_engine_of(R.pattern, true);
    s, e := find_span(engine, text, 0);

    if e < 0 return out, false;

    ok := fill_groups(R, engine, text, s, e, *out);

    return out, ok;
}

// Like parse, into your own struct: each field takes the named group of the same name, and is a
// string, integer, float, bool (`true` / `false`) or enum (a member's name, any case). A field
// with no group or a group with no field is a compile error. False if the text does not match
// or a group's text does not fit its field.
parse_into :: ($R: Type, text: string, out: *$T) -> bool #modify {
    return is_static(R);
} {
    engine := static_engine_of(R.pattern, true);
    s, e := find_span(engine, text, 0);
    if e < 0 return false;
    return fill_struct(R.pattern, engine, text, s, e, out);
}

// Like captures, into your own struct (see parse_into).
captures_into :: ($R: Type, text: string, out: *$T, start := 0) -> bool #modify {
    return is_static(R);
} {
    engine := static_engine_of(R.pattern, false);
    s, e := find_span(engine, text, start);
    if e < 0 return false;
    return fill_struct(R.pattern, engine, text, s, e, out);
}

replace :: ($R: Type, text: string, replacement: string) -> string #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return replace_n(*re, text, replacement, 1);
}

replace_all :: ($R: Type, text: string, replacement: string) -> string #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return replace_n(*re, text, replacement, -1);
}

replace_n :: ($R: Type, text: string, replacement: string, limit: int) -> string #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return replace_n(*re, text, replacement, limit);
}

split :: ($R: Type, text: string) -> [..] string #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return split(*re, text);
}

#scope_module

// The engine built at compile time, in the program's constant data.
static_engine_of :: ($pattern: string, $full: bool) -> *Engine {
    ENGINE :: #run static_engine(pattern, full);
    #if ENGINE.frozen {
        return *ENGINE;
    } else {
        // Too many states to store: build the DFAs lazily, once per run.
        return cached_engine(pattern, full);
    }
}

static_regex :: ($pattern: string) -> Regex {
    re: Regex;
    re.pattern = pattern;
    re.engine = static_engine_of(pattern, false);
    re.group_count = re.engine.program.slot_count / 2 - 1;
    re.names = re.engine.program.names;

    return re;
}

cached_engine :: (pattern: string, full: bool) -> *Engine {
    key := ifx full then tprint("\\A(?:%)\\z", pattern) else pattern;
    found, engine := table_find(*engine_cache, key);

    if found return engine;

    key = copy_string(key);
    engine = build_engine(key, .{});
    table_add(*engine_cache, key, engine);

    return engine;
}

engine_cache: Table(string, *Engine);

fill_groups :: ($R: Type, engine: *Engine, text: string, s: int, e: int, out: *R.Groups) -> bool {
    out.match = .{ s, e, slice(text, s, e - s) };
    slots := NewArray(engine.program.slot_count, int,, temp);

    fill_slots(engine, text, s, e, slots);

    #insert -> string {
        return groups_fill(R.pattern);
    }

    return true;
}

fill_struct :: ($pattern: string, engine: *Engine, text: string, s: int, e: int, out: *$T) -> bool {
    slots := NewArray(engine.program.slot_count, int,, temp);

    fill_slots(engine, text, s, e, slots);

    #insert -> string {
        return into_fill(pattern, T);
    }

    return true;
}

#import "Basic";
#import "String";
#import "Hash_Table";
#import "Compiler";
Byte_Search :: #import "Extensions/Byte_Search";

#load "unicode.jai";
#load "syntax.jai";
#load "program.jai";
#load "nfa.jai";
#load "onepass.jai";
#load "dfa.jai";
#load "search.jai";
#load "generate.jai";
