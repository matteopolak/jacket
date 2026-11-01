// Regular expressions with full Unicode support, whose patterns can be compiled while your
// program compiles. [compile] builds one at run time; [Static] builds one at compile time and
// types its named groups. [find], [captures], [replace] and [split] work on either.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// // Compiled while the program compiles: a bad pattern is a compile error, and named
// // groups become typed fields (an integer for a run of digits, an enum for a few words).
// Date :: Regex.Static("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})");
// date, ok := Regex.parse(Date, "2026-10-11");  // date.year is a u16, date.month a u8
//
// // Compiled at run time, for a pattern only known then.
// user_pattern := "[a-z]+";
// text := "one two";
// re, compiled, error := Regex.compile(user_pattern);
// for Regex.find_all(*re, text) print("% ", it.text);
// ```
//
// Matching takes time linear in the text for every pattern: there are no backreferences and no
// look-around, and the engines are automata, never an exponential backtracker.

// Pattern flags, also settable inside the pattern: (?i) (?m) (?s) (?U) (?x) (?R) (?-u).
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// options: Regex.Options;
// options.case_insensitive = true;
// re, ok := Regex.compile("hello", options);
// assert(ok && Regex.is_match(*re, "Say HELLO"));
// Regex.free_regex(*re);
// ```
Options :: struct {
    case_insensitive: bool; // i: letters match either case (Unicode simple case folding)
    multi_line: bool; // m: ^ and $ match at line breaks, not just the ends of the text
    dot_all: bool; // s: . also matches \n
    swap_greed: bool; // U: x* is lazy and x*? greedy
    ignore_space: bool; // x: whitespace is ignored and # starts a comment
    crlf: bool; // R: with m, \r\n also ends a line, and . does not match \r
    unicode := true; // u: classes and \b are Unicode-aware; off, they are ASCII and match bytes
}

// A compiled pattern from [compile]. Plain copies of the struct share the compiled program and the search cache, which is not
// safe to use from two threads at once: give each thread its own [clone].
Regex :: struct {
    pattern: string;
    group_count: int; // capture groups, not counting group 0 (the whole match)
    names: [] string; // names[i] is group i's name, or ""
    engine: *Engine;
}

// Why a pattern did not compile.
Error :: struct {
    message: string; // `unclosed group: expected `)``
    offset: int; // byte offset in the pattern where the problem is
}

// One match: its byte range and a view of the matched text.
Match :: struct {
    start: int;
    end: int;
    text: string; // the matched text, a view into the searched string
}

// Where each group matched. Group 0 is the whole match; a group that did not take part has
// start -1. Read it with [group] and [span].
Captures :: struct {
    text: string; // the whole searched string
    slots: [] int; // start and end of each group, in pairs
    names: [] string;
}

// Compiles `pattern`. The result says whether it was valid and, if not, the error says where it
// went wrong. Free the regex with [free_regex].
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, ok, error := Regex.compile("[a-z]+[0-9]");
// assert(ok);
// Regex.free_regex(*re);
//
// _, ok, error = Regex.compile("(unclosed");
// assert(!ok && error.message.count > 0);
// ```
//
// - pattern: the regular expression.
// - options: flags such as case-insensitive matching.
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

// A regex for another thread: it shares the compiled pattern (parse, programs and one-pass table) with `re` and has a search
// cache of its own, so it costs far less than a second [compile]. Free it with [free_regex], like any other regex.
//
// Clone before the threads start, or from the thread that owns `re`: `re` must not be freed while `clone` runs. After that,
// the regexes can be freed in any order and on any thread; the shared part goes with the last one. Memory for the clone and its
// cache comes from `context.allocator`, so free it with the same allocator.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("[0-9]+");
// copy := Regex.clone(*re);
// Regex.free_regex(*re);   // the clone keeps the pattern alive
// assert(Regex.is_match(*copy, "room 101"));
// Regex.free_regex(*copy);
// ```
clone :: (re: *Regex) -> Regex {
    result := re.*;
    if !re.engine return result;

    origin := re.engine;
    if origin.origin origin = origin.origin;

    assert(origin.refs > 0 && !origin.frozen, "Regex.clone: the regex was freed or is not from compile");
    atomic_increment(*origin.refs);

    engine := New(Engine);
    engine.pattern = origin.pattern;
    engine.program = origin.program;
    engine.reverse = origin.reverse;
    engine.onepass = origin.onepass;
    engine.fixed_length = origin.fixed_length;
    engine.origin = origin;

    dfa_init(*engine.forward, *engine.program, false);
    dfa_init(*engine.backward, *engine.reverse, true);

    result.engine = engine;

    return result;
}

// Releases everything [compile] allocated for the regex: the parsed pattern, both programs and the search tables. For a regex
// made by [clone], that is its own search cache, and the pattern goes when the last regex sharing it is freed.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, ok := Regex.compile("a+b");
// assert(ok && Regex.is_match(*re, "caab"));
// Regex.free_regex(*re);
// assert(re.engine == null);
// ```
free_regex :: (re: *Regex) {
    if !re.engine return;

    engine := re.engine;

    dfa_free(*engine.forward);
    dfa_free(*engine.backward);
    re.engine = null;

    origin := engine;

    if engine.origin {
        origin = engine.origin;
        free(engine);
    }

    // The shared part goes with the last regex that uses it.
    if !atomic_decrement(*origin.refs) return;

    onepass_free(*origin.onepass);

    // A compiled engine lives in its own pool, the engine itself included.
    if origin.pool {
        pool := origin.pool;

        Memory_Pool.release(pool);
        free(pool);
    }
} @FreesArgument

// Whether captures for this pattern are found in a single scan, which makes [captures] faster.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("(a)(b)");
// assert(Regex.is_one_pass(*re));
// Regex.free_regex(*re);
// ```
is_one_pass :: (re: *Regex) -> bool {
    return re.engine.onepass.ok;
}

// [is_one_pass] for a static pattern.
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Pair :: Regex.Static("(a)(b)");
//
// assert(Regex.is_one_pass(Pair));
// ```
is_one_pass :: ($R: Type) -> bool #modify {
    return is_static(R);
} {
    return static_engine_of(R.pattern, false).onepass.ok;
}

// Whether the pattern matches anywhere in `text`, starting at or after byte `start`.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("[0-9]+");
// assert(Regex.is_match(*re, "room 101"));
// assert(!Regex.is_match(*re, "no digits"));
// Regex.free_regex(*re);
// ```
is_match :: (re: *Regex, text: string, start := 0) -> bool {
    _, end := find_span(re.engine, text, start, earliest = true);
    return end >= 0;
}

// The leftmost match at or after byte `start`.
//
// - start: the byte offset to begin at. Anchors and `\b` still see the text before it.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("[0-9]+");
// match, found := Regex.find(*re, "room 101, floor 3");
// assert(found && match.text == "101" && match.start == 5);
// Regex.free_regex(*re);
// ```
find :: (re: *Regex, text: string, start := 0) -> Match, bool {
    s, e := find_span(re.engine, text, start);
    if e < 0 return .{}, false;
    return .{ s, e, slice(text, s, e - s) }, true;
}

// Every match, left to right, not overlapping. An empty match right where the previous match
// ended is skipped, and in Unicode mode empty matches never split a UTF-8 sequence.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("[0-9]+");
// matches := Regex.find_all(*re, "room 101, floor 3");
// assert(matches.count == 2 && matches[1].text == "3");
// array_free(matches);
// Regex.free_regex(*re);
// ```
find_all :: (re: *Regex, text: string) -> [..] Match {
    out: [..] Match;
    for_matches(re.engine, text, #code array_add(*out, .{ s, e, slice(text, s, e - s) }));
    return out;
}

// The leftmost match at or after `start`, with where each group matched. Read the groups with
// [group] and [span].
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("(?<key>[a-z]+)=(?<value>[0-9]+)");
// caps, found := Regex.captures(*re, "retries=3");
// assert(found);
// key, _ := Regex.group(caps, "key");
// assert(key == "retries");
// Regex.free_regex(*re);
// ```
captures :: (re: *Regex, text: string, start := 0) -> Captures, bool {
    s, e := find_span(re.engine, text, start);
    if e < 0 return .{}, false;
    return captures_of(re.engine, text, s, e), true;
}

// [captures] for every match, as [find_all] finds them.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("([a-z])=([0-9])");
// all := Regex.captures_all(*re, "a=1, b=2");
// assert(all.count == 2);
// value, _ := Regex.group(all[1], 2);
// assert(value == "2");
// Regex.free_regex(*re);
// ```
captures_all :: (re: *Regex, text: string) -> [..] Captures {
    out: [..] Captures;
    for_matches(re.engine, text, #code array_add(*out, captures_of(engine, text, s, e)));
    return out;
}

// Group `index`'s text, and whether it took part in the match. Group 0 is the whole match.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("(a)(x)?");
// caps, _ := Regex.captures(*re, "ab");
// first, took_part := Regex.group(caps, 1);
// assert(took_part && first == "a");
// _, took_part = Regex.group(caps, 2);
// assert(!took_part);
// Regex.free_regex(*re);
// ```
group :: (caps: Captures, index: int) -> string, bool {
    if index < 0 || 2 * index + 1 >= caps.slots.count return "", false;

    s := caps.slots[2 * index];
    e := caps.slots[2 * index + 1];

    if s < 0 return "", false;

    return slice(caps.text, s, e - s), true;
}

// The text of the group called `name`, and whether it took part in the match.
group :: (caps: Captures, name: string) -> string, bool {
    for caps.names if it.count && it == name return group(caps, it_index);
    return "", false;
}

// Group `index`'s byte range, or -1, -1.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("b+");
// caps, _ := Regex.captures(*re, "abbc");
// start, end := Regex.span(caps, 0);
// assert(start == 1 && end == 3);
// Regex.free_regex(*re);
// ```
span :: (caps: Captures, index: int) -> start: int, end: int {
    if index < 0 || 2 * index + 1 >= caps.slots.count return -1, -1;
    return caps.slots[2 * index], caps.slots[2 * index + 1];
}

// Replaces the first match. In `replacement`, `$1` / `${1}` is a group's text, `$name` /
// `${name}` a named group's, and `$$` a dollar sign; a group that did not match (or does not
// exist) gives "". `$name` takes the longest run of letters, digits and `_`, so write
// `${1}st` rather than `$1st`.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("(\\w+)@(\\w+)");
// assert(Regex.replace(*re, "ada@home, bob@work", "$2:$1") == "home:ada, bob@work");
// Regex.free_regex(*re);
// ```
replace :: (re: *Regex, text: string, replacement: string) -> string {
    return replace_n(re, text, replacement, 1);
}

// Replaces every match; `replacement` works as in [replace].
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("[0-9]+");
// assert(Regex.replace_all(*re, "a1 b22", "#") == "a# b#");
// Regex.free_regex(*re);
// ```
replace_all :: (re: *Regex, text: string, replacement: string) -> string {
    return replace_n(re, text, replacement, -1);
}

// Replaces the first `limit` matches (all of them when limit < 0).
//
// - replacement: the new text; `$1` or `${1}` stands for a group and `$name` or `${name}` for a named
//   one.
// - limit: how many matches to replace. Negative replaces all of them.
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("[0-9]+");
// assert(Regex.replace_n(*re, "1 2 3", "#", 2) == "# # 3");
// Regex.free_regex(*re);
// ```
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
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// re, _ := Regex.compile("\\s*,\\s*");
// parts := Regex.split(*re, "a , b,c");
// assert(parts.count == 3 && parts[1] == "b");
// array_free(parts);
// Regex.free_regex(*re);
// ```
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
//
// ```jai
// Regex :: #import "Extensions/Regex";
//
// assert(Regex.escape("1+1=2?") == "1\\+1=2\\?");
// ```
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
// ```jai
// Regex :: #import "Extensions/Regex";
// Date :: Regex.Static("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})");
//
// date, ok := Regex.parse(Date, "2026-10-11");  // a Date.Groups: date.year is a u16
// assert(ok && date.year == 2026 && date.month == 10 && date.day == 11);
// ```
//
// A bad pattern is a compile error, and the automata are built at compile time (big Unicode
// classes excepted: those are built on first use), so searching needs no setup. The procedures
// below take it in place of `*re`; [parse], [parse_into] and [captures_into] only take one of
// these. For a pattern known only at run time, use [compile].
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

// Whether the static pattern matches anywhere in `text`.
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Digits :: Regex.Static("[0-9]+");
//
// assert(Regex.is_match(Digits, "room 101"));
// ```
is_match :: ($R: Type, text: string, start := 0) -> bool #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return is_match(*re, text, start);
}

// The leftmost match of the static pattern at or after `start`.
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Digits :: Regex.Static("[0-9]+");
//
// match, found := Regex.find(Digits, "room 101");
// assert(found && match.text == "101");
// ```
find :: ($R: Type, text: string, start := 0) -> Match, bool #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    match, ok := find(*re, text, start);
    return match, ok;
}

// Every match of the static pattern, as [find_all] does for a [Regex].
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Digits :: Regex.Static("[0-9]+");
//
// assert(Regex.find_all(Digits, "1 22 333").count == 3);
// ```
find_all :: ($R: Type, text: string) -> [..] Match #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return find_all(*re, text);
}

// The leftmost match at or after `start`, with its named groups parsed. False if there is no
// match, or a number does not fit its field.
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Pair :: Regex.Static("(?<key>[a-z]+)=(?<value>[0-9]+)");
//
// pair, ok := Regex.captures(Pair, "  retries=3");
// assert(ok && pair.key == "retries" && pair.value == 3);
// ```
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

// [captures] for every match.
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Pair :: Regex.Static("(?<key>[a-z]+)=(?<value>[0-9]+)");
//
// pairs := Regex.captures_all(Pair, "a=1 b=2");
// assert(pairs.count == 2 && pairs[1].key == "b");
// ```
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
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Time :: Regex.Static("(?<hour>[0-9]{2}):(?<minute>[0-9]{2})");
//
// time, ok := Regex.parse(Time, "09:30");
// assert(ok && time.hour == 9 && time.minute == 30);
// _, ok = Regex.parse(Time, "at 09:30");
// assert(!ok);
// ```
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
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Time :: Regex.Static("(?<hour>[0-9]{2}):(?<minute>[0-9]{2})");
//
// Clock :: struct { hour: int; minute: int; }
//
// clock: Clock;
// assert(Regex.parse_into(Time, "09:30", *clock));
// assert(clock.hour == 9 && clock.minute == 30);
// ```
parse_into :: ($R: Type, text: string, out: *$T) -> bool #modify {
    return is_static(R);
} {
    engine := static_engine_of(R.pattern, true);
    s, e := find_span(engine, text, 0);
    if e < 0 return false;
    return fill_struct(R.pattern, engine, text, s, e, out);
}

// Like [captures], into your own struct (see [parse_into]).
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Time :: Regex.Static("(?<hour>[0-9]{2}):(?<minute>[0-9]{2})");
//
// Clock :: struct { hour: int; minute: int; }
//
// clock: Clock;
// assert(Regex.captures_into(Time, "meet at 09:30", *clock));
// assert(clock.minute == 30);
// ```
captures_into :: ($R: Type, text: string, out: *$T, start := 0) -> bool #modify {
    return is_static(R);
} {
    engine := static_engine_of(R.pattern, false);
    s, e := find_span(engine, text, start);
    if e < 0 return false;
    return fill_struct(R.pattern, engine, text, s, e, out);
}

// Replaces the first match of the static pattern; `replacement` works as for a [Regex].
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Digits :: Regex.Static("[0-9]+");
//
// assert(Regex.replace(Digits, "1 2", "#") == "# 2");
// ```
replace :: ($R: Type, text: string, replacement: string) -> string #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return replace_n(*re, text, replacement, 1);
}

// Replaces every match of the static pattern.
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Digits :: Regex.Static("[0-9]+");
//
// assert(Regex.replace_all(Digits, "1 2", "#") == "# #");
// ```
replace_all :: ($R: Type, text: string, replacement: string) -> string #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return replace_n(*re, text, replacement, -1);
}

// Replaces the first `limit` matches of the static pattern; negative replaces all.
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Digits :: Regex.Static("[0-9]+");
//
// assert(Regex.replace_n(Digits, "1 2 3", "#", 2) == "# # 3");
// ```
replace_n :: ($R: Type, text: string, replacement: string, limit: int) -> string #modify {
    return is_static(R);
} {
    re := static_regex(R.pattern);
    return replace_n(*re, text, replacement, limit);
}

// The pieces of `text` between matches of the static pattern.
//
// ```jai
// Regex :: #import "Extensions/Regex";
// Comma :: Regex.Static(",");
//
// assert(Regex.split(Comma, "a,b,c").count == 3);
// ```
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
    engine, _, _ = build_engine(key, .{});
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
#import "Atomics";
#import "String";
#import "Hash_Table";
#import "Compiler";
Byte_Search :: #import "Extensions/Byte_Search";
Memory_Pool :: #import "Pool";

#load "unicode.jai";
#load "syntax.jai";
#load "program.jai";
#load "nfa.jai";
#load "onepass.jai";
#load "dfa.jai";
#load "search.jai";
#load "generate.jai";
