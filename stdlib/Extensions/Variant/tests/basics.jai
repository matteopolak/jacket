// Making, switching, reading and comparing variants: variant, set, is, get, get_pointer, unwrap,
// variant_type, variant_switch (both forms, every order) and ==.
//
//     jaic stdlib/Extensions/Variant/tests/basics.jai -interp

#import "Basic";
#import "Extensions/Variant";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Circle :: struct {
    radius: int;
}

Rect :: struct {
    width: int;
    height: int;
}

Empty :: struct {}

Number_Or_Text :: Variant(s64, string);
Shape :: Variant(Circle, Rect, Empty);

construction :: () {
    v := variant(Number_Or_Text, 42);

    check(v.index == 0, "an int is the first case");
    check(is(v, s64) && !is(v, string), "is sees the live case");
    check(variant_index(v) == 0, "variant_index");

    w: Number_Or_Text;

    check(w.index == 0 && is(w, s64), "the zero value is the first case");
    check(unwrap(w, s64) == 0, "the zero value is zeroed");

    set(*w, "hi");

    check(w.index == 1 && is(w, string) && !is(w, s64), "set switches the case");
    check(unwrap(w, string) == "hi", "set stores the value");

    set(*w, 7);

    check(w.index == 0 && unwrap(w, s64) == 7, "set switches back");

    set(*w, 8);

    check(unwrap(w, s64) == 8, "set on the live case replaces the value");

    s := variant(Shape, Rect.{ 3, 4 });

    check(s.index == 1, "the case is picked by the value's type");

    e := variant(Shape, Empty.{});

    check(e.index == 2 && is(e, Empty), "a case without a payload");

    // A Variant of one case is just a wrapper.
    single := variant(Variant(string), "only");

    check(single.index == 0 && unwrap(single, string) == "only", "one case");
}

types :: () {
    v := variant(Number_Or_Text, 1);

    check(variant_type(v) == s64, "variant_type of the first case");

    set(*v, "x");

    check(variant_type(v) == string, "variant_type follows set");

    which := "";

    if variant_type(v) == {
        case s64;
            which = "number";
        case string;
            which = "text";
    }

    check(which == "text", "variant_type in a switch");

    s := variant(Shape, Empty.{});

    check(variant_type(s) == Empty && variant_type(s) != Circle, "variant_type of a struct case");
}

reading :: () {
    v := variant(Number_Or_Text, 42);

    number, ok := get(v, s64);

    check(ok && number == 42, "get returns a copy and true");

    text, other := get(v, string);

    check(!other && text.count == 0, "get on another case returns a zero value and false");

    set(*v, "hello");

    number, ok = get(v, s64);

    check(!ok && number == 0, "get zeroes the value when another case is live");

    text, other = get(v, string);

    check(other && text == "hello", "get a string");

    pointer := get_pointer(*v, string);

    check(pointer != null && pointer.* == "hello", "get_pointer points at the payload");
    check(get_pointer(*v, s64) == null, "get_pointer is null for another case");

    pointer.* = "changed";

    check(unwrap(v, string) == "changed", "a write through get_pointer is seen");

    check(cast(*u8) pointer == v.payload.data, "the pointer is into the variant's own storage");

    s := variant(Shape, Rect.{ 3, 4 });
    rect, found := get(s, Rect);

    check(found && rect.width == 3 && rect.height == 4, "get a struct");

    get_pointer(*s, Rect).height = 9;

    check(unwrap(s, Rect).height == 9, "write one member through get_pointer");

    _, is_empty := get(s, Empty);

    check(!is_empty, "get of the Empty case while another is live");

    // get copies: changing the copy does not change the variant.
    rect.width = 100;

    check(unwrap(s, Rect).width == 3, "get returns a copy");
}

switching :: () {
    v := variant(Number_Or_Text, 42);

    variant_switch(v,
        (it: *s64) {
            assert(it.* == 42);
        },
        (it: *string) {
            assert(false);
        },
    );

    // The handlers can be given in any order.
    w := variant(Number_Or_Text, "abc");

    variant_switch(w,
        (it: *string) {
            assert(it.* == "abc");
        },
        (it: *s64) {
            assert(false);
        },
    );

    // A handler can change a variant switched by pointer, but not one switched by value.
    variant_switch(*v,
        (it: *s64) {
            it.* += 1;
        },
        (it: *string) {},
    );

    check(unwrap(v, s64) == 43, "a handler writes through the pointer form");

    variant_switch(v,
        (it: *s64) {
            it.* += 100;
        },
        (it: *string) {},
    );

    check(unwrap(v, s64) == 43, "the value form works on a copy");

    // Every case, including the one without a payload.
    shapes: [3] Shape;

    set(*shapes[0], Circle.{ 2 });
    set(*shapes[1], Rect.{ 3, 4 });
    set(*shapes[2], Empty.{});

    for i: 0..2 {
        variant_switch(shapes[i],
            (it: *Rect) {
                assert(it.width == 3 && it.height == 4);
            },
            (it: *Empty) {},
            (it: *Circle) {
                assert(it.radius == 2);
            },
        );
    }

    // Which handler ran is seen through a pointer to the variant's own storage.
    for i: 0..2 {
        variant_switch(*shapes[i],
            (it: *Circle) {
                it.radius = 20;
            },
            (it: *Rect) {
                it.width = 30;
            },
            (it: *Empty) {},
        );
    }

    check(unwrap(shapes[0], Circle).radius == 20 && unwrap(shapes[1], Rect).width == 30, "each case ran its own handler");

    // One handler per case also when there is only one case.
    single := variant(Variant(string), "x");

    variant_switch(single, (it: *string) {
        assert(it.* == "x");
    });
}

W0 :: struct {
    v: int;
}

W1 :: struct {
    v: int;
}

W2 :: struct {
    v: int;
}

W3 :: struct {
    v: int;
}

W4 :: struct {
    v: int;
}

W5 :: struct {
    v: int;
}

W6 :: struct {
    v: int;
}

W7 :: struct {
    v: int;
}

W8 :: struct {
    v: int;
}

W9 :: struct {
    v: int;
}

W10 :: struct {
    v: int;
}

W11 :: struct {
    v: int;
}

W12 :: struct {
    v: int;
}

W13 :: struct {
    v: int;
}

W14 :: struct {
    v: int;
}

W15 :: struct {
    v: int;
}

Sixteen :: Variant(W0, W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15);

// Handlers are procedures and cannot see the locals around them.
hit := -1;

sixteen :: () {
    v := variant(Sixteen, W15.{ 15 });

    check(v.index == 15 && is(v, W15), "the sixteenth case");

    hit = -1;

    // The most cases a variant_switch takes.
    variant_switch(*v,
        (it: *W0) {
            hit = 0;
        }, (it: *W1) {
            hit = 1;
        }, (it: *W2) {
            hit = 2;
        }, (it: *W3) {
            hit = 3;
        },
        (it: *W4) {
            hit = 4;
        }, (it: *W5) {
            hit = 5;
        }, (it: *W6) {
            hit = 6;
        }, (it: *W7) {
            hit = 7;
        },
        (it: *W8) {
            hit = 8;
        }, (it: *W9) {
            hit = 9;
        }, (it: *W10) {
            hit = 10;
        }, (it: *W11) {
            hit = 11;
        },
        (it: *W12) {
            hit = 12;
        }, (it: *W13) {
            hit = 13;
        }, (it: *W14) {
            hit = 14;
        }, (it: *W15) {
            hit = it.v;
        },
    );

    check(hit == 15, "variant_switch with sixteen handlers");
}

equality :: () {
    a := variant(Number_Or_Text, 5);
    b := variant(Number_Or_Text, 5);
    c := variant(Number_Or_Text, 6);
    d := variant(Number_Or_Text, "5");

    check(a == b && !(a != b), "equal numbers");
    check(a != c && !(a == c), "different numbers");
    check(a != d, "different cases are never equal");

    first := variant(Number_Or_Text, copy_string("same text"));
    second := variant(Number_Or_Text, copy_string("same text"));

    check(first == second, "strings compare by content, not address");

    set(*second, copy_string("other text"));

    check(first != second, "different strings");

    free_variant(*first);
    free_variant(*second);

    shape_a := variant(Shape, Rect.{ 1, 2 });
    shape_b := variant(Shape, Rect.{ 1, 2 });
    shape_c := variant(Shape, Rect.{ 1, 3 });

    check(shape_a == shape_b && shape_a != shape_c, "structs compare member by member");
    check(variant(Shape, Empty.{}) == variant(Shape, Empty.{}), "the Empty case equals itself");

    // Stale bytes of an earlier case do not matter.
    stale := variant(Shape, Rect.{ 9, 9 });

    set(*stale, Circle.{ 1 });

    check(stale == variant(Shape, Circle.{ 1 }), "equality ignores what an earlier case left");

    // Nested variants and arrays compare through their own cases.
    Pair :: Variant(Number_Or_Text, [2] int);

    p := variant(Pair, variant(Number_Or_Text, "x"));
    q := variant(Pair, variant(Number_Or_Text, "x"));
    r := variant(Pair, variant(Number_Or_Text, "y"));

    check(p == q && p != r, "a nested variant");

    n1 := variant(Pair, int.[1, 2]);
    n2 := variant(Pair, int.[1, 2]);
    n3 := variant(Pair, int.[1, 3]);

    check(n1 == n2 && n1 != n3, "a fixed array case");

    Numbers :: Variant(float64, [..] int);

    list_a := variant(Numbers, cast(float64) 1.5);
    list_b := variant(Numbers, cast(float64) 1.5);

    check(list_a == list_b, "a float case");

    v1: [..] int;
    v2: [..] int;

    array_add(*v1, 1);
    array_add(*v1, 2);
    array_add(*v2, 1);
    array_add(*v2, 2);

    la := variant(Numbers, v1);
    lb := variant(Numbers, v2);

    check(la == lb, "a resizable array case compares its elements");

    array_add(*v2, 3);

    lb = variant(Numbers, v2);

    check(la != lb, "arrays of different length");

    array_free(v1);
    array_free(v2);
}

main :: () {
    construction();
    types();
    reading();
    switching();
    sixteen();
    equality();

    assert(failures == 0);
    print("ok\n");
}
