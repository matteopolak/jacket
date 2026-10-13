// Checks of the Unicode tables behind the regex engines: property names, the Perl classes,
// range helpers and simple case folding.
//
//     jaic stdlib/Extensions/Regex/tests/unicode.jai -interp

#import "Basic";

#load "../unicode.jai";

failures := 0;

fail :: (format: string, args: ..Any) {
    print("FAIL: ");
    print(format, ..args);
    print("\n");
    failures += 1;
}

expect :: (condition: bool, what: string) {
    if !condition fail("%", what);
}

contains :: (ranges: [] Char_Range, c: u32) -> bool {
    for ranges if c >= it.lo && c <= it.hi return true;
    return false;
}

is_normalized :: (ranges: [] Char_Range) -> bool {
    for ranges {
        if it.lo > it.hi return false;
        if it_index > 0 && cast(s64) it.lo <= cast(s64) ranges[it_index - 1].hi + 1 return false;
    }

    return true;
}

same_ranges :: (a: [] Char_Range, b: [] Char_Range) -> bool {
    if a.count != b.count return false;
    for a if it.lo != b[it_index].lo || it.hi != b[it_index].hi return false;
    return true;
}

size_of_ranges :: (ranges: [] Char_Range) -> s64 {
    total: s64 = 0;
    for ranges total += cast(s64) it.hi - cast(s64) it.lo + 1;
    return total;
}

// Looks a property up and checks the result is well formed; empty ranges when it fails.
lookup :: (spec: string) -> [..] Char_Range {
    ranges, ok, error := unicode_property(spec);
    if !ok {
        fail("`%` did not resolve: %", spec, error);
        return ranges;
    }

    expect(error == "", tprint("`%` succeeded but has an error text", spec));
    expect(is_normalized(ranges), tprint("`%` is not sorted and merged", spec));

    return ranges;
}

has :: (spec: string, c: u32) -> bool {
    return contains(lookup(spec), c);
}

unknown :: (spec: string, message: string) {
    _, ok, error := unicode_property(spec);
    if ok fail("`%` should not resolve", spec);
    else if error != message fail("`%`: error was `%`, expected `%`", spec, error, message);
}

same_as :: (spec: string, other: string) {
    expect(same_ranges(lookup(spec), lookup(other)), tprint("`%` should be the same set as `%`", spec, other));
}

test_general_categories :: () {
    expect(has("Lu", #char "A") && !has("Lu", #char "a"), "Lu holds A, not a");
    expect(has("Ll", #char "a") && !has("Ll", #char "A"), "Ll holds a, not A");
    expect(has("Nd", #char "7") && has("Nd", 0x0663), "Nd holds ASCII and Arabic-Indic digits");
    expect(has("Zs", 0x00A0) && has("Zs", 0x3000), "Zs holds no-break and ideographic space");
    expect(has("Cn", 0x0378) && !has("Cn", #char "a"), "Cn holds unassigned code points");
    expect(has("Cs", 0xD800) && has("Co", 0xE000), "Cs and Co");
    expect(has("L", 0x4E2D) && has("L", #char "a") && !has("L", #char "1"), "L");
    expect(has("LC", #char "a") && has("LC", 0x01C5) && !has("LC", 0x4E2D), "LC is Lu, Ll and Lt only");
    expect(has("C", 0x0378) && has("C", 0x0007) && has("C", 0xE000), "C includes Cn, Cc and Co");
    expect(has("M", 0x0301) && has("N", 0x00BD) && has("P", #char "!") && has("S", #char "+"), "M, N, P and S");
    expect(has("Z", #char " ") && has("Z", 0x2028), "Z");
    expect(has("Math_Symbol", #char "+") && has("Uppercase_Letter", #char "Q") && has("letter", #char "q"), "long names");
    expect(has("gc=Lu", #char "A") && has("General_Category=Uppercase_Letter", #char "A") && has("gc:Lu", #char "A"), "gc= and gc:");

    // Every code point is in exactly one two-letter category.
    total: s64 = 0;

    for string.["Lu", "Ll", "Lt", "Lm", "Lo", "Mn", "Mc", "Me", "Nd", "Nl", "No", "Pc", "Pd", "Ps", "Pe", "Pi", "Pf", "Po", "Sm", "Sc", "Sk", "So", "Zs", "Zl", "Zp", "Cc", "Cf", "Cs", "Co", "Cn"] {
        total += size_of_ranges(lookup(it));
    }

    expect(total == 0x110000, "the 30 categories partition the code space");
}

test_scripts :: () {
    expect(has("Greek", 0x03B1) && !has("Greek", #char "a"), "Greek holds alpha, not a");
    expect(has("sc=Grek", 0x03B1) && has("Script=Greek", 0x03B1) && has("script:greek", 0x03B1), "sc spellings");
    expect(has("Latin", #char "a") && !has("Latin", 0x03B1), "Latin");
    expect(has("Han", 0x4E2D) && has("Hiragana", 0x3042) && has("Cyrillic", 0x0416), "Han, Hiragana, Cyrillic");
    expect(has("Common", #char "1") && has("Inherited", 0x0301), "Common and Inherited");
    expect(has("Unknown", 0x0378) && !has("Unknown", #char "a"), "Unknown");
    same_as("sc=Qaai", "Inherited");

    // U+0342 is Inherited by script but used with Greek.
    expect(!has("sc=Grek", 0x0342) && has("scx=Grek", 0x0342), "scx adds extensions");
    expect(has("scx=Greek", 0x03B1) && !has("scx=Greek", #char "a"), "scx includes the script itself");
    expect(!has("Script_Extensions=Latn", 0x0342) && has("scx=Latn", #char "a"), "scx=Latn");
    // The middle dot is Common but extended to Greek, Latin and others.
    expect(has("scx=Latn", 0x00B7) && has("scx=Grek", 0x00B7) && has("sc=Common", 0x00B7), "scx of U+00B7");
    expect(!has("scx=Common", 0x00B7), "U+00B7 left scx=Common");
}

test_binary_properties :: () {
    expect(has("White_Space", #char " ") && has("White_Space", 0x00A0) && has("White_Space", 0x2028), "White_Space");
    same_as("wspace", "White_Space");
    same_as("space", "White_Space");
    expect(has("Alphabetic", #char "a") && has("Alphabetic", 0x00E9) && !has("Alphabetic", #char "1"), "Alphabetic");
    expect(has("Emoji", 0x1F600) && has("Emoji", #char "#") && !has("Emoji", #char "a"), "Emoji");
    expect(has("Emoji_Presentation", 0x1F600) && !has("Emoji_Presentation", #char "#"), "Emoji_Presentation");
    expect(has("Emoji_Modifier", 0x1F3FB) && has("Emoji_Modifier_Base", 0x1F44D), "emoji modifiers");
    expect(has("Extended_Pictographic", 0x1F600) && has("Emoji_Component", 0x200D), "pictographic and component");
    expect(has("Uppercase", #char "A") && has("Lowercase", #char "a") && has("Cased", #char "a"), "case properties");
    expect(has("ASCII_Hex_Digit", #char "f") && !has("ASCII_Hex_Digit", #char "g"), "ASCII_Hex_Digit");
    expect(has("XID_Start", #char "a") && has("ID_Continue", #char "_") && !has("ID_Start", #char "_"), "identifiers");
    expect(has("Join_Control", 0x200D) && has("Bidi_Mirrored", #char "(") && has("Dash", #char "-"), "misc binary properties");
    expect(has("Math", #char "+") && has("Default_Ignorable_Code_Point", 0x200B), "Math and default ignorable");
    expect(has("Pattern_White_Space", #char " ") && has("Noncharacter_Code_Point", 0xFFFE), "pattern white space and noncharacters");
    expect(has("Changes_When_Uppercased", #char "a") && !has("Changes_When_Uppercased", #char "A"), "Changes_When_Uppercased");
    expect(has("Full_Composition_Exclusion", 0x0958), "Full_Composition_Exclusion");

    same_as("White_Space=yes", "White_Space");
    same_as("Alphabetic=Y", "alpha");
    same_as("Alphabetic=true", "Alphabetic");
    same_as("Alphabetic:t", "Alphabetic");
    negated := ranges_negate(lookup("Alphabetic"));

    expect(same_ranges(negated, lookup("Alphabetic=no")), "=no is the complement");
    expect(same_ranges(negated, lookup("alpha=N")) && same_ranges(negated, lookup("alpha=false")) && same_ranges(negated, lookup("alpha:f")), "other spellings of no");
    expect(has("Alphabetic=No", #char "1") && !has("Alphabetic=No", #char "a"), "=No holds the digit");
}

test_special_names :: () {
    expect(size_of_ranges(lookup("Any")) == 0x110000, "Any");
    ascii := lookup("ASCII");

    expect(ascii.count == 1 && ascii[0].lo == 0 && ascii[0].hi == 0x7F, "ASCII");
    assigned := lookup("Assigned");

    expect(contains(assigned, #char "a") && !contains(assigned, 0x0378), "Assigned");
    expect(size_of_ranges(assigned) + size_of_ranges(lookup("Cn")) == 0x110000, "Assigned and Cn are complements");
    same_as("any", "ANY");
}

test_loose_matching :: () {
    same_as("Uppercase_Letter", "Lu");
    same_as("uppercaseletter", "Lu");
    same_as("UPPERCASE LETTER", "lu");
    same_as("uppercase-letter", "Lu");
    same_as("isLu", "Lu");
    same_as("IsGreek", "Greek");
    same_as("greek", "Grek");
    same_as("Letter", "L");
    same_as("letter", "L");
    same_as("Mark", "M");
    same_as("Combining_Mark", "M");
    same_as("Cased_Letter", "LC");
    same_as("Old_Italic", "Ital");
    same_as("Old Italic", "old-italic");
    same_as("sc = Greek", "sc=Greek");
    same_as("SCRIPT=GREK", "Greek");
    same_as("gc=letter", "L");
    same_as("scx=Greek", "scx=Grek");
    same_as("ASCII_Hex_Digit", "AHex");
    same_as("IsAlphabetic", "Alphabetic");

    // A bare name is a general category before a script before a binary property.
    expect(!has("Cased", 0x4E2D) && has("Cased", #char "a"), "Cased is the binary property");
}

test_errors :: () {
    unknown("Foo", "unknown Unicode property `Foo`");
    unknown("", "unknown Unicode property ``");
    unknown("Greekk", "unknown Unicode property `Greekk`");
    unknown("is", "unknown Unicode property `is`");
    unknown("sc=Nope", "unknown value `Nope` for Unicode property `sc`");
    unknown("gc=Greek", "unknown value `Greek` for Unicode property `gc`");
    unknown("Bogus=Greek", "unknown Unicode property `Bogus`");
    unknown("Alphabetic=maybe", "unknown value `maybe` for Unicode property `Alphabetic`");
    unknown("Alphabetic=", "unknown value `` for Unicode property `Alphabetic`");
    unknown("=Greek", "unknown Unicode property ``");
    unknown("Script", "unknown Unicode property `Script`");
    unknown("Lu=yes", "unknown Unicode property `Lu`");
}

test_perl_classes :: () {
    word := unicode_perl_word();
    digit := unicode_perl_digit();
    space := unicode_perl_space();

    expect(is_normalized(word) && is_normalized(digit) && is_normalized(space), "perl classes are normalized");

    expect(contains(word, #char "_") && contains(word, 0x00E9) && contains(word, 0x0663) && contains(word, 0x200D), "\\w holds _, e-acute, Arabic-Indic digit and ZWJ");
    expect(contains(word, #char "a") && contains(word, #char "Z") && contains(word, #char "0") && contains(word, 0x0301), "\\w holds ASCII word characters and marks");
    expect(!contains(word, #char "-") && !contains(word, #char " ") && !contains(word, 0x00D7) && !contains(word, 0x2014), "\\w excludes punctuation and symbols");

    expect(contains(digit, #char "5") && contains(digit, 0x0663) && !contains(digit, #char "a") && !contains(digit, 0x00B2), "\\d is Nd");
    expect(same_ranges(digit, lookup("Nd")), "\\d equals Nd");

    expect(contains(space, #char " ") && contains(space, #char "\t") && contains(space, 0x00A0) && contains(space, 0x2028) && contains(space, 0x3000), "\\s holds white space");
    expect(!contains(space, #char "a") && !contains(space, 0x200B), "\\s excludes letters and zero width space");
    expect(same_ranges(space, lookup("White_Space")), "\\s equals White_Space");
}

test_range_helpers :: () {
    ranges: [..] Char_Range;

    array_add(*ranges, .{ 10, 20 }, .{ 1, 3 }, .{ 4, 5 }, .{ 15, 30 }, .{ 40, 40 }, .{ 41, 41 }, .{ 7, 7 });
    ranges_normalize(*ranges);
    expected := Char_Range.[.{ 1, 5 }, .{ 7, 7 }, .{ 10, 30 }, .{ 40, 41 }];

    expect(same_ranges(ranges, expected), "normalize sorts and merges");

    empty: [..] Char_Range;

    ranges_normalize(*empty);
    expect(empty.count == 0, "normalize of nothing");
    expect(size_of_ranges(ranges_negate(empty)) == 0x110000, "negation of nothing is everything");

    negated := ranges_negate(ranges);

    expect(is_normalized(negated), "negation is normalized");
    expect(negated[0].lo == 0 && negated[0].hi == 0, "negation starts at zero");
    expect(negated[negated.count - 1].hi == 0x10FFFF && negated[negated.count - 1].lo == 42, "negation ends at the top");
    expect(size_of_ranges(negated) + size_of_ranges(ranges) == 0x110000, "negation covers the rest");
    again := ranges_negate(negated);

    expect(same_ranges(again, ranges), "negating twice round-trips");

    whole := Char_Range.[.{ 0, 0x10FFFF }];

    expect(ranges_negate(whole).count == 0, "negation of everything is nothing");

    letters := lookup("L");

    expect(same_ranges(ranges_negate(ranges_negate(letters)), letters), "negation round-trips on \\p{L}");
}

includes :: (items: [] u32, c: u32) -> bool {
    for items if it == c return true;
    return false;
}

test_case_folding :: () {
    k := unicode_simple_fold(#char "k");

    expect(k.count == 2 && includes(k, #char "K") && includes(k, 0x212A), "k folds with K and the Kelvin sign");
    kelvin := unicode_simple_fold(0x212A);

    expect(kelvin.count == 2 && includes(kelvin, #char "k") && includes(kelvin, #char "K"), "Kelvin sign orbit");

    sigma := unicode_simple_fold(0x03C3);

    expect(sigma.count == 2 && includes(sigma, 0x03A3) && includes(sigma, 0x03C2), "sigma folds with Sigma and final sigma");
    final := unicode_simple_fold(0x03C2);

    expect(final.count == 2 && includes(final, 0x03C3) && includes(final, 0x03A3), "final sigma orbit");

    s := unicode_simple_fold(#char "s");

    expect(s.count == 2 && includes(s, #char "S") && includes(s, 0x017F), "s folds with S and long s");
    expect(includes(unicode_simple_fold(0x1E9E), 0x00DF), "capital sharp s folds with sharp s");
    expect(includes(unicode_simple_fold(0x00E5), 0x00C5) && includes(unicode_simple_fold(0x00E5), 0x212B), "a-ring folds with A-ring and the Angstrom sign");

    expect(unicode_simple_fold(#char "1").count == 0, "digits have no orbit");
    expect(unicode_simple_fold(0x4E2D).count == 0, "ideographs have no orbit");
    expect(unicode_simple_fold(0x0130).count == 0, "dotted capital I has no simple orbit");
    expect(unicode_simple_fold(0x10FFFF).count == 0, "last code point");
    expect(unicode_simple_fold(0).count == 0, "NUL");

    // Orbits are symmetric: if b is in a's orbit then a is in b's.
    for c: u32.[#char "a", #char "z", #char "A", 0xE9, 0x3C3, 0x212A, 0x1E9E] {
        for other: unicode_simple_fold(c) {
            expect(includes(unicode_simple_fold(other), c), tprint("orbit of U+% is not symmetric", c));
        }
    }
}

test_case_fold_ranges :: () {
    lower := Char_Range.[.{ #char "a", #char "z" }];
    folded := unicode_case_fold_ranges(lower);

    expect(is_normalized(folded), "folded ranges are normalized");
    expect(contains(folded, #char "A") && contains(folded, #char "Z") && contains(folded, #char "m"), "[a-z] gains A-Z");
    expect(contains(folded, 0x017F) && contains(folded, 0x212A), "[a-z] gains long s and the Kelvin sign");
    expect(!contains(folded, #char "0") && !contains(folded, 0x00E9), "[a-z] gains nothing unrelated");

    single := Char_Range.[.{ 0x03C3, 0x03C3 }];
    sigma := unicode_case_fold_ranges(single);

    expect(contains(sigma, 0x03A3) && contains(sigma, 0x03C2) && contains(sigma, 0x03C3), "sigma range");
    expect(sigma.count == 3 || sigma.count == 2, "sigma range size");

    // Unsorted and overlapping input is accepted.
    messy := Char_Range.[.{ #char "z", #char "z" }, .{ #char "a", #char "c" }, .{ #char "b", #char "d" }];
    result := unicode_case_fold_ranges(messy);

    expect(contains(result, #char "A") && contains(result, #char "D") && contains(result, #char "Z") && !contains(result, #char "E"), "messy input");

    nothing: [..] Char_Range;

    expect(unicode_case_fold_ranges(nothing).count == 0, "folding no ranges");
    digits := Char_Range.[.{ #char "0", #char "9" }];

    expect(same_ranges(unicode_case_fold_ranges(digits), digits), "digits are their own fold");

    // Folding a whole category is closed under folding and stays within bounds.
    letters := lookup("L");
    all_letters := unicode_case_fold_ranges(letters);

    expect(is_normalized(all_letters), "folded \\p{L} is normalized");
    expect(size_of_ranges(all_letters) >= size_of_ranges(letters), "folding only adds");
    expect(same_ranges(unicode_case_fold_ranges(all_letters), all_letters), "folding is idempotent");

    upper := lookup("Lu");
    folded_upper := unicode_case_fold_ranges(upper);

    expect(contains(folded_upper, #char "a") && contains(folded_upper, 0x03C2), "folded Lu holds lower case");
}

main :: () {
    test_general_categories();
    test_scripts();
    test_binary_properties();
    test_special_names();
    test_loose_matching();
    test_errors();
    test_perl_classes();
    test_range_helpers();
    test_case_folding();
    test_case_fold_ranges();

    if failures {
        print("% unicode checks failed\n", failures);
        exit(1);
    }
}
