// Patterns known at compile time: typed groups, parse and parse_into, and the static versions
// of the search procedures.
//
//     jaic run stdlib/Extensions/Regex/tests/typed.jai

#import "Basic";
Regex :: #import "Extensions/Regex";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Date :: Regex.Static("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})");
Pet_Regex :: Regex.Static("(?<kind>cat|dog|bird) (?<age>-?\\d+)(?: (?<weight>\\d+\\.\\d+))?(?: \"(?<name>[^\"]*)\")?");

Pet :: struct {
    kind: enum u16 {
        CAT;
        DOG;
        BIRD;
    };
    age: s8;
    weight: float32;
    name: string;
}

Setting :: struct {
    key: string;
    on: bool;
}

types :: () {
    date: Date.Groups;
    check(type_of(date.year) == u16 && type_of(date.month) == u8, "fixed digit counts give the smallest integer");
    pet: Pet_Regex.Groups;
    check(type_of(pet.age) == s64, "an unbounded signed run is an s64");
    check(type_of(pet.weight) == float64, "a decimal is a float64");
    check(type_of(pet.name) == string, "anything else is a string");
    check(type_of(pet.kind) == Pet_Regex.Groups.Kind, "a few words make an enum");
    check(type_of(pet.match) == Regex.Match, "the whole match");
    check(type_of(pet.has.weight) == bool && type_of(pet.has.name) == bool, "optional groups get flags");
}

parsing :: () {
    date, ok := Regex.parse(Date, "2026-10-11");
    check(ok && date.year == 2026 && date.month == 10 && date.day == 11, "parse");
    check(date.match.start == 0 && date.match.end == 10, "the match");
    _, ok = Regex.parse(Date, " 2026-10-11");
    check(!ok, "parse needs the whole text");
    _, ok = Regex.parse(Date, "2026-10-11\n");
    check(!ok, "parse needs the whole text, not up to a newline");

    pet: Pet_Regex.Groups;
    pet, ok = Regex.parse(Pet_Regex, "dog -3 4.5");
    check(ok && pet.kind == .DOG && pet.age == -3 && pet.weight == 4.5, "parse with an enum, a sign and a float");
    check(pet.has.weight && !pet.has.name, "which optional groups took part");
    pet, ok = Regex.captures(Pet_Regex, "a bird 2 \"Tweety\" ...");
    check(ok && pet.kind == .BIRD && pet.name == "Tweety" && pet.match.start == 2, "captures");

    small, fits := Regex.parse(Regex.Static("(?<n>\\d{2})"), "99");
    check(fits && small.n == 99, "u8");
    huge: Regex.Static("(?<n>\\d+)").Groups;
    huge, fits = Regex.parse(Regex.Static("(?<n>\\d+)"), "18446744073709551615");
    check(fits && huge.n == 0xFFFF_FFFF_FFFF_FFFF, "the largest u64");
    _, fits = Regex.parse(Regex.Static("(?<n>\\d+)"), "18446744073709551616");
    check(!fits, "a number too big for its field");
    negative, fits2 := Regex.parse(Regex.Static("(?<n>[+-]\\d{1,2})"), "-99");
    check(fits2 && negative.n == -99 && type_of(negative.n) == s8, "s8");
    arabic, fits3 := Regex.parse(Regex.Static("(?<n>\\d{3})"), "٣٤٥");
    check(fits3 && arabic.n == 345, "digits from another script");

    size, ok2 := Regex.parse(Regex.Static("(?<size>small|SMALL|bigOne|big_one|if)"), "bigOne");
    check(ok2 && size.size == .BIG_ONE, "words become upper snake case members");
    size, ok2 = Regex.parse(Regex.Static("(?<size>small|SMALL|bigOne|big_one|if)"), "small");
    check(ok2 && size.size == .SMALL && size.size != .IF, "words with the same member fold into one");
    shout, ok3 := Regex.parse(Regex.Static("(?i)(?<word>yes|no)"), "YeS");
    check(ok3 && shout.word == .YES, "a case-insensitive word");
    _, ok3 = Regex.parse(Regex.Static("(?i)(?<word>ok|no)"), "o\u212A");
    check(!ok3, "a word matched through a non-ASCII fold has no member");
    one: Regex.Static("(?<w>a|A)").Groups;
    check(type_of(one.w) == string, "one member is a string");

    dates := Regex.captures_all(Date, "1999-01-02, 2000-12-31");
    check(dates.count == 2 && dates[1].year == 2000 && dates[1].day == 31, "captures_all");
}

into :: () {
    pet: Pet;
    check(Regex.parse_into(Pet_Regex, "bird 7 1.25 \"Polly\"", *pet), "parse_into");
    check(pet.kind == .BIRD && pet.age == 7 && pet.weight == 1.25 && pet.name == "Polly", tprint("parse_into fields: %", pet));
    check(!Regex.parse_into(Pet_Regex, "cat 300", *pet), "a number too big for an s8 field");
    check(!Regex.parse_into(Pet_Regex, "cow 3", *pet), "no match");

    setting: Setting;
    check(Regex.captures_into(Regex.Static("(?<key>\\w+)=(?<on>\\w+)"), "x: debug=TRUE", *setting), "captures_into");
    check(setting.key == "debug" && setting.on, "a bool field");
    check(!Regex.captures_into(Regex.Static("(?<key>\\w+)=(?<on>\\w+)"), "debug=maybe", *setting), "not a bool");
}

searching :: () {
    check(Regex.is_match(Regex.Static("\\bfoo\\b"), "a foo b"), "is_match");
    check(!Regex.is_match(Regex.Static("\\bfoo\\b"), "afoo"), "is_match, no match");
    m, found := Regex.find(Regex.Static("\\p{Greek}+"), "abc αβγ");
    check(found && m.text == "αβγ", "find with a Unicode class");
    check(Regex.find_all(Regex.Static("\\d+"), "1 22 333").count == 3, "find_all");
    check(Regex.replace(Regex.Static("\\d"), "a1b2", "#") == "a#b2", "replace");
    check(Regex.replace_all(Regex.Static("(?<w>\\w+)@"), "a@ bb@ c", "<$w>") == "<a> <bb> c", "replace_all");
    parts := Regex.split(Regex.Static("\\s*,\\s*"), "a , b,c");
    check(parts.count == 3 && parts[1] == "b", "split");
    // Big enough that its DFAs are built at run time instead.
    m, found = Regex.find(Regex.Static("\\w+\\s+\\w+@\\w+"), "-- héllo wörld@ünï --");
    check(found && m.text == "héllo wörld@ünï", "a pattern compiled lazily at run time");
}

main :: () {
    types();
    parsing();
    into();
    searching();
    if failures {
        print("% typed regex checks failed\n", failures);
        exit(1);
    }
}
