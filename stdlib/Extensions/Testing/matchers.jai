// Matchers: a check written as the value and a description of what it should be.
//
// ```jai
// assert_that(7, is(7));
// assert_that("hello world", contains("lo w"));
// assert_that(s32.[1, 2, 3], all_of(has_length(3), has_item(is(2)), not(is_empty())));
// ```
//
// A matcher is built from `is`, `contains` and the others below, and composed with `not`,
// `all_of` and `any_of`. `assert_that` reports a failure like `expected 4 to be greater than 5`
// and lets the test go on; `require_that` returns from the procedure it is written in.

Matcher_Kind :: enum {
    EQUAL;
    GREATER;
    AT_LEAST;
    LESS;
    AT_MOST;
    BETWEEN;
    NEAR;
    CONTAINS;
    STARTS_WITH;
    ENDS_WITH;
    IS_EMPTY;
    HAS_LENGTH;
    HAS_ITEM;
    EACH;
    IS_NULL;
    IS_TRUE;
    IS_FALSE;
    OF_TYPE;
    NOT;
    ALL_OF;
    ANY_OF;
}

// What a value should be. Build one with the procedures below; the operands live in temporary
// storage, so a matcher is for the test that makes it.
Matcher :: struct {
    kind: Matcher_Kind;
    first: Any;
    second: Any;
    type: *Type_Info;
    children: [] Matcher;
}

// Checks `value` against `matcher`; a failure is reported at the caller's line. Returns whether
// it held.
//
// ```jai,should_fail
// assert_that(4, greater_than(5));
// ```
assert_that :: (value: $T, matcher: Matcher, loc := #caller_location) -> bool {
    if holds(value, matcher) return true;
    report_failure(loc, failure_message(value, matcher, false));
    return false;
}

// `assert_that`, then returns from the procedure it is written in (which must return nothing)
// when the check fails.
require_that :: (value: $T, matcher: Matcher, loc := #caller_location) #expand {
    if !assert_that(value, matcher, loc) `return;
}

// The message `assert_that` would report, or an empty string when `value` matches.
explain :: (value: $T, matcher: Matcher) -> string {
    if holds(value, matcher) return "";
    return failure_message(value, matcher, false);
}

// Equal by value, as `expect_equal` compares; strings, arrays and structs fail with its diff.
is :: (want: $T) -> Matcher {
    return .{ kind = .EQUAL, first = capture(want) };
}

not :: (matcher: Matcher) -> Matcher {
    return .{ kind = .NOT, children = copy_matchers(.[matcher]) };
}

// Every matcher holds.
all_of :: (matchers: ..Matcher) -> Matcher {
    return .{ kind = .ALL_OF, children = copy_matchers(matchers) };
}

// At least one matcher holds.
any_of :: (matchers: ..Matcher) -> Matcher {
    return .{ kind = .ANY_OF, children = copy_matchers(matchers) };
}

// Numbers of any width compare by value, strings byte by byte.
greater_than :: (limit: $T) -> Matcher {
    return .{ kind = .GREATER, first = capture(limit) };
}

at_least :: (limit: $T) -> Matcher {
    return .{ kind = .AT_LEAST, first = capture(limit) };
}

less_than :: (limit: $T) -> Matcher {
    return .{ kind = .LESS, first = capture(limit) };
}

at_most :: (limit: $T) -> Matcher {
    return .{ kind = .AT_MOST, first = capture(limit) };
}

// `low <= value <= high`.
between :: (low: $T, high: $U) -> Matcher {
    return .{ kind = .BETWEEN, first = capture(low), second = capture(high) };
}

// `|value - want| <= tolerance`. NaN is never near anything.
near :: (want: float64, tolerance: float64) -> Matcher {
    return .{ kind = .NEAR, first = capture(want), second = capture(tolerance) };
}

// A string holds `part` as a substring; an array holds `part` as an element.
contains :: (part: $T) -> Matcher {
    return .{ kind = .CONTAINS, first = capture(part) };
}

// A string begins with `prefix`; an array begins with the elements of the array `prefix`.
starts_with :: (prefix: $T) -> Matcher {
    return .{ kind = .STARTS_WITH, first = capture(prefix) };
}

// A string ends with `suffix`; an array ends with the elements of the array `suffix`.
ends_with :: (suffix: $T) -> Matcher {
    return .{ kind = .ENDS_WITH, first = capture(suffix) };
}

// A string or array with nothing in it.
is_empty :: () -> Matcher {
    return .{ kind = .IS_EMPTY };
}

// A string with `count` bytes or an array with `count` elements.
has_length :: (count: s64) -> Matcher {
    return .{ kind = .HAS_LENGTH, first = capture(count) };
}

// An array with an element `matcher` holds for.
has_item :: (matcher: Matcher) -> Matcher {
    return .{ kind = .HAS_ITEM, children = copy_matchers(.[matcher]) };
}

// An array whose every element `matcher` holds for (an empty array qualifies).
each :: (matcher: Matcher) -> Matcher {
    return .{ kind = .EACH, children = copy_matchers(.[matcher]) };
}

is_null :: () -> Matcher {
    return .{ kind = .IS_NULL };
}

not_null :: () -> Matcher {
    return not(is_null());
}

is_true :: () -> Matcher {
    return .{ kind = .IS_TRUE };
}

is_false :: () -> Matcher {
    return .{ kind = .IS_FALSE };
}

// The value's type is `T` exactly; for an `Any`, the type of what it holds.
of_type :: ($T: Type) -> Matcher {
    return .{ kind = .OF_TYPE, type = type_info(T) };
}

#scope_file

capture :: (value: $T) -> Any {
    storage := cast(*T) talloc(size_of(T));
    storage.* = value;
    return Any.{ type_info(T), storage };
}

copy_matchers :: (matchers: [] Matcher) -> [] Matcher {
    copied: [] Matcher;
    copied.count = matchers.count;
    copied.data = cast(*Matcher) talloc(size_of(Matcher) * matchers.count);
    for matchers copied[it_index] = it;

    return copied;
}

// What `Any` holds, however many Any wrappers and variants are around it.
unwrapped :: (original: Any) -> Any {
    if !original.type return original;

    value := underlying(original);
    while value.type.type == .ANY {
        value = underlying((cast(*Any) value.value_pointer).*);
    }

    return value;
}

is_number :: (value: Any) -> bool {
    return is_integral(value) || value.type.type == .FLOAT;
}

as_float :: (value: Any) -> float64 {
    if value.type.type == .FLOAT return float_value(value);
    number := integer_value(value);
    magnitude := cast(float64) number.magnitude;
    return ifx number.negative then -magnitude else magnitude;
}

// Whether the two can be ordered, and if so -1, 0 or 1 for `a` before, equal to or after `b`.
order_of :: (a: Any, b: Any) -> (comparable: bool, order: s64) {
    if is_integral(a) && is_integral(b) {
        x := integer_value(a);
        y := integer_value(b);

        if x.negative != y.negative return true, ifx x.negative then -1 else 1;

        less := x.magnitude < y.magnitude;
        if x.negative less = x.magnitude > y.magnitude;
        if less return true, -1;
        if x.magnitude == y.magnitude return true, 0;

        return true, 1;
    }

    if is_number(a) && is_number(b) {
        x := as_float(a);
        y := as_float(b);

        if x < y return true, -1;
        if x > y return true, 1;
        if x <= y && x >= y return true, 0;

        return false, 0;
    }

    if a.type.type == .STRING && b.type.type == .STRING {
        x := (cast(*string) a.value_pointer).*;
        y := (cast(*string) b.value_pointer).*;

        for 0..min(x.count, y.count) - 1 {
            if x[it] < y[it] return true, -1;
            if x[it] > y[it] return true, 1;
        }

        if x.count < y.count return true, -1;
        if x.count > y.count return true, 1;

        return true, 0;
    }

    return false, 0;
}

string_has_at :: (haystack: string, needle: string, at: s64) -> bool {
    if at < 0 || at + needle.count > haystack.count return false;
    for 0..needle.count - 1 if haystack[at + it] != needle[it] return false;
    return true;
}

// The count of a string or an array; ok is false for anything else.
sequence_of :: (value: Any) -> (ok: bool, count: s64) {
    if value.type.type == .STRING return true, (cast(*string) value.value_pointer).*.count;

    if value.type.type == .ARRAY {
        _, count, _ := array_view(value);
        return true, count;
    }

    return false, 0;
}

// Whether the elements of array `part` sit in array `whole` starting at `at`.
elements_at :: (whole: Any, part: Any, at: s64) -> bool {
    ew, cw, dw := array_view(whole);
    ep, cp, dp := array_view(part);

    if at < 0 || at + cp > cw return false;

    for 0..cp - 1 {
        if !values_equal(element(ew, dw, at + it), element(ep, dp, it)) return false;
    }

    return true;
}

holds :: (original: Any, matcher: Matcher) -> bool {
    actual := unwrapped(original);
    want := unwrapped(matcher.first);

    if matcher.kind == {
        case .EQUAL;
            return values_equal(actual, want);

        case .GREATER;
            comparable, order := order_of(actual, want);

            return comparable && order > 0;
        case .AT_LEAST;
            comparable, order := order_of(actual, want);

            return comparable && order >= 0;
        case .LESS;
            comparable, order := order_of(actual, want);

            return comparable && order < 0;
        case .AT_MOST;
            comparable, order := order_of(actual, want);

            return comparable && order <= 0;
        case .BETWEEN;
            high := unwrapped(matcher.second);
            comparable_low, low_order := order_of(actual, want);
            comparable_high, high_order := order_of(actual, high);

            return comparable_low && comparable_high && low_order >= 0 && high_order <= 0;
        case .NEAR;
            if !is_number(actual) return false;

            difference := as_float(actual) - as_float(want);
            if difference < 0 difference = -difference;

            return difference <= as_float(unwrapped(matcher.second));

        case .CONTAINS;
            if actual.type.type == .STRING && want.type.type == .STRING {
                haystack := (cast(*string) actual.value_pointer).*;
                needle := (cast(*string) want.value_pointer).*;
                for 0..haystack.count - needle.count if string_has_at(haystack, needle, it) return true;
                return false;
            }

            if actual.type.type == .ARRAY {
                e, count, data := array_view(actual);
                for 0..count - 1 if values_equal(element(e, data, it), want) return true;
            }

            return false;
        case .STARTS_WITH;
            if actual.type.type == .STRING && want.type.type == .STRING {
                return string_has_at((cast(*string) actual.value_pointer).*, (cast(*string) want.value_pointer).*, 0);
            }

            if actual.type.type == .ARRAY && want.type.type == .ARRAY return elements_at(actual, want, 0);

            return false;
        case .ENDS_WITH;
            if actual.type.type == .STRING && want.type.type == .STRING {
                text := (cast(*string) actual.value_pointer).*;
                suffix := (cast(*string) want.value_pointer).*;
                return string_has_at(text, suffix, text.count - suffix.count);
            }

            if actual.type.type == .ARRAY && want.type.type == .ARRAY {
                _, count, _ := array_view(actual);
                _, suffix, _ := array_view(want);
                return elements_at(actual, want, count - suffix);
            }

            return false;
        case .IS_EMPTY;
            ok, count := sequence_of(actual);

            return ok && count == 0;
        case .HAS_LENGTH;
            ok, count := sequence_of(actual);

            return ok && integers_equal(want, capture(count));
        case .HAS_ITEM;
            if actual.type.type != .ARRAY return false;

            e, count, data := array_view(actual);
            for 0..count - 1 if holds(element(e, data, it), matcher.children[0]) return true;

            return false;
        case .EACH;
            if actual.type.type != .ARRAY return false;

            e, count, data := array_view(actual);
            for 0..count - 1 if !holds(element(e, data, it), matcher.children[0]) return false;

            return true;

        case .IS_NULL;
            if actual.type.type != .POINTER && actual.type.type != .PROCEDURE return false;

            return (cast(**void) actual.value_pointer).* == null;
        case .IS_TRUE;
            return actual.type.type == .BOOL && (cast(*bool) actual.value_pointer).*;
        case .IS_FALSE;
            return actual.type.type == .BOOL && !(cast(*bool) actual.value_pointer).*;
        case .OF_TYPE;
            return actual.type == matcher.type;

        case .NOT;
            return !holds(original, matcher.children[0]);
        case .ALL_OF;
            for matcher.children if !holds(original, it) return false;

            return true;
        case .ANY_OF;
            for matcher.children if holds(original, it) return true;

            return false;
    }

    return false;
}

// A value as it reads in a message: strings quoted, everything else as it prints.
show :: (value: Any) -> string {
    shown := unwrapped(value);
    if !shown.type return "";
    if shown.type.type == .STRING return quoted((cast(*string) shown.value_pointer).*);
    return tprint("%", shown);
}

// What the matcher asks of a value, to follow "expected V to".
phrase :: (matcher: Matcher) -> string {
    first := show(matcher.first);

    if matcher.kind == {
        case .EQUAL; return tprint("equal %", first);
        case .GREATER; return tprint("be greater than %", first);
        case .AT_LEAST; return tprint("be at least %", first);
        case .LESS; return tprint("be less than %", first);
        case .AT_MOST; return tprint("be at most %", first);
        case .BETWEEN; return tprint("be between % and %", first, show(matcher.second));
        case .NEAR; return tprint("be within % of %", show(matcher.second), first);
        case .CONTAINS; return tprint("contain %", first);
        case .STARTS_WITH; return tprint("start with %", first);
        case .ENDS_WITH; return tprint("end with %", first);
        case .IS_EMPTY; return "be empty";
        case .HAS_LENGTH; return tprint("have length %", first);
        case .HAS_ITEM; return tprint("have an item that should %", phrase(matcher.children[0]));
        case .EACH; return tprint("have every item that should %", phrase(matcher.children[0]));
        case .IS_NULL; return "be null";
        case .IS_TRUE; return "be true";
        case .IS_FALSE; return "be false";
        case .OF_TYPE; return tprint("be of type %", type_name(Any.{ matcher.type, null }));
        case .NOT; return tprint("not %", phrase(matcher.children[0]));
        case .ALL_OF; return tprint("satisfy all of: %", phrases(matcher.children));
        case .ANY_OF; return tprint("satisfy any of: %", phrases(matcher.children));
    }

    return "";
}

phrases :: (matchers: [] Matcher) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for matchers {
        if it_index > 0 append(*builder, "; ");
        append(*builder, phrase(it));
    }

    return builder_to_string(*builder,, temp);
}

// Why `matcher` (negated when `negated`) does not hold for the value: the failing part of a
// composite, and the diff for an equality of strings, arrays and structs.
failure_message :: (original: Any, matcher: Matcher, negated: bool) -> string {
    actual := unwrapped(original);

    if matcher.kind == {
        case .NOT;
            return failure_message(original, matcher.children[0], !negated);
        case .EQUAL;
            kind := actual.type.type;
            if !negated && kind == unwrapped(matcher.first).type.type && (kind == .STRING || kind == .ARRAY || kind == .STRUCT) {
                return describe_difference(actual, matcher.first);
            }
        case .ALL_OF;
            if !negated {
                for matcher.children if !holds(original, it) return failure_message(original, it, false);
            }
        case .ANY_OF;
            if negated {
                for matcher.children if holds(original, it) return failure_message(original, it, true);
            }
        case .EACH;
            if !negated && actual.type.type == .ARRAY {
                e, count, data := array_view(actual);

                for 0..count - 1 {
                    item := element(e, data, it);
                    if holds(item, matcher.children[0]) continue;

                    return tprint("expected % to %\n  [%]: %", show(actual), phrase(matcher), it, failure_message(item, matcher.children[0], false));
                }
            }
    }

    return tprint("expected % % %", show(actual), ifx negated then "not to" else "to", phrase(matcher));
}
