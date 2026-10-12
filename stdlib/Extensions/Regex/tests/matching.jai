// The run-time API: compiling, errors, searching, captures, replacing, splitting and options.
//
//     jaic run stdlib/Extensions/Regex/tests/matching.jai

#import "Basic";
Regex :: #import "Extensions/Regex";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

compiled :: (pattern: string, options := Regex.Options.{}) -> Regex.Regex {
    re, ok, error := Regex.compile(pattern, options);
    check(ok, tprint("% compiles (%)", pattern, error.message));
    return re;
}

errors :: () {
    Bad :: struct {
        pattern: string;
        offset: int;
    }
    BAD :: Bad.[
        .{ "ab(c", 2 }, .{ "a)", 1 }, .{ "[a", 0 }, .{ "a{3,2}", 1 }, .{ "*a", 0 }, .{ "(?<1>a)", 3 },
        .{ "\\p{Nope}", 0 }, .{ "a\\1", 1 }, .{ "(?=a)", 0 }, .{ "(?<!a)", 0 }, .{ "[b-a]", 1 },
        .{ "(?P<x>a)(?P<x>b)", 12 }, .{ "a{,3}", 1 }, .{ "\\Z", 0 }, .{ "(?z)", 2 },
    ];
    for BAD {
        _, ok, error := Regex.compile(it.pattern);
        check(!ok, tprint("% does not compile", it.pattern));
        check(error.message.count > 0, tprint("% has a message", it.pattern));
        check(error.offset == it.offset, tprint("% fails at % (said %: %)", it.pattern, it.offset, error.offset, error.message));
    }
    // Deep nesting is refused rather than overflowing the stack.
    deep: String_Builder;
    for 1..300 append(*deep, "(");
    append(*deep, "a");
    for 1..300 append(*deep, ")");
    _, ok := Regex.compile(builder_to_string(*deep));
    check(!ok, "300 nested groups");
}

searching :: () {
    re := compiled("[0-9]+");
    m, found := Regex.find(*re, "ab 12 345");
    check(found && m.start == 3 && m.end == 5 && m.text == "12", "find");
    m, found = Regex.find(*re, "ab 12 345", 5);
    check(found && m.text == "345", "find from an offset");
    _, found = Regex.find(*re, "none");
    check(!found, "no match");
    check(Regex.is_match(*re, "x9"), "is_match");
    all := Regex.find_all(*re, "1 22 333");
    check(all.count == 3 && all[2].text == "333", "find_all");

    // Leftmost-first, not leftmost-longest.
    alt := compiled("a|ab");
    m = Regex.find(*alt, "ab");
    check(m.text == "a", "alternation prefers the first branch");
    lazy := compiled("a+?");
    m = Regex.find(*lazy, "aaa");
    check(m.text == "a", "lazy repetition");

    // Empty matches: never right after the previous match, never inside a UTF-8 sequence.
    empty := compiled("a*");
    spans := Regex.find_all(*empty, "baaé");
    check(spans.count == 3, tprint("empty matches in baaé: %", spans));
    if spans.count == 3 check(spans[0].end == 0 && spans[1].start == 1 && spans[1].end == 3 && spans[2].start == 5, "empty match positions");

    // \b sees the text before the search start.
    word := compiled("\\bb");
    _, found = Regex.find(*word, "ab", 1);
    check(!found, "\\b looks behind the start");
    m, found = Regex.find(*word, "é b");
    check(found && m.start == 3, "Unicode word boundary");
    unicode_word := compiled("\\w+");
    m = Regex.find(*unicode_word, "--héllo--");
    check(m.text == "héllo", "\\w is Unicode");
}

capturing :: () {
    re := compiled("(?<key>\\w+)=(?<value>\\w*)(;)?");
    caps, found := Regex.captures(*re, "a=1; b=");
    check(found, "captures");
    check(re.group_count == 3 && re.names[1] == "key", "group count and names");
    key, ok := Regex.group(caps, "key");
    check(ok && key == "a", "named group");
    value := Regex.group(caps, 2);
    check(value == "1", "numbered group");
    s, e := Regex.span(caps, 3);
    check(s == 3 && e == 4, "span");
    _, ok = Regex.group(caps, "missing");
    check(!ok, "unknown group name");
    all := Regex.captures_all(*re, "a=1; b=");
    check(all.count == 2, "captures_all");
    if all.count == 2 {
        _, ok = Regex.group(all[1], 3);
        check(!ok, "a group that did not take part");
        check(Regex.group(all[1], "value") == "" && Regex.span(all[1], 2) == 7, "an empty group");
    }
    // A repeated group keeps its last iteration.
    rep := compiled("(?:(\\d)x)+");
    caps = Regex.captures(*rep, "1x2x3x");
    check(Regex.group(caps, 1) == "3", "last iteration");
}

replacing :: () {
    re := compiled("(?<first>\\w+) (?<second>\\w+)");
    check(Regex.replace(*re, "hello world, big cat", "$second $first") == "world hello, big cat", "replace");
    check(Regex.replace_all(*re, "hello world, big cat", "${second}_$1") == "world_hello, cat_big", "replace_all");
    check(Regex.replace_all(*re, "a b", "$$1") == "$1", "a literal dollar");
    check(Regex.replace_all(*re, "a b", "$3|$nope|") == "||", "missing groups give nothing");
    digits := compiled("\\d");
    check(Regex.replace_n(*digits, "1a2b3", "#", 2) == "#a#b3", "replace_n");
    check(Regex.replace_all(*digits, "none", "#") == "none", "no matches");
    empty := compiled("");
    check(Regex.replace_all(*empty, "aé", "-") == "-a-é-", "empty matches between characters");

    comma := compiled("\\s*,\\s*");
    parts := Regex.split(*comma, "a , b,,c");
    check(parts.count == 4 && parts[1] == "b" && parts[2] == "" && parts[3] == "c", tprint("split: %", parts));
    check(Regex.split(*comma, "").count == 1, "split of nothing");

    escaped := Regex.escape("1+1=2? (a.b)");
    check(escaped == "1\\+1=2\\? \\(a\\.b\\)", tprint("escape: %", escaped));
    literal := compiled(escaped);
    check(Regex.is_match(*literal, "so 1+1=2? (a.b)!"), "escaped text matches itself");
}

options :: () {
    folded := compiled("straße", .{ case_insensitive = true });
    check(Regex.is_match(*folded, "STRAßE"), "case-insensitive");
    kelvin := compiled("(?i)k");
    check(Regex.is_match(*kelvin, "K"), "the Kelvin sign folds to k");
    lines := compiled("^\\w+$", .{ multi_line = true });
    check(Regex.find_all(*lines, "ab\ncd\n").count == 2, "multi-line");
    crlf := compiled("(?mR)^\\w+$");
    check(Regex.find_all(*crlf, "ab\r\ncd").count == 2, "CRLF lines");
    dot := compiled(".", .{ dot_all = true });
    check(Regex.is_match(*dot, "\n"), "dot_all");
    plain_dot := compiled(".");
    check(!Regex.is_match(*plain_dot, "\n"), ". skips \\n");
    greedy := compiled("a+", .{ swap_greed = true });
    check(Regex.find(*greedy, "aaa").text == "a", "swap_greed");
    spaced := compiled("a b # comment\nc", .{ ignore_space = true });
    check(Regex.is_match(*spaced, "abc"), "ignore_space");
    bytes := compiled("\\w+", .{ unicode = false });
    check(Regex.find(*bytes, "éab").text == "ab", "ASCII \\w");
    greek := compiled("\\p{Greek}+");
    check(Regex.find(*greek, "abc αβγ").text == "αβγ", "script property");
    class := compiled("[\\p{L}&&[^a-z]]+");
    check(Regex.find(*class, "abcDÉF").text == "DÉF", "class intersection");
}

linear :: () {
    // A pattern that makes backtracking engines take exponential time.
    re := compiled("(a*)*b");
    text: String_Builder;
    for 1..5000 append(*text, "a");
    check(!Regex.is_match(*re, builder_to_string(*text)), "(a*)*b on a long run of a");
    caps, found := Regex.captures(*re, "aaab");
    check(found && Regex.group(caps, 1) == "aaa", "(a*)*b captures");
}

main :: () {
    errors();
    searching();
    capturing();
    replacing();
    options();
    linear();
    if failures {
        print("% regex checks failed\n", failures);
        exit(1);
    }
}
