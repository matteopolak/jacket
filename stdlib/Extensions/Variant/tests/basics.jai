// Making, switching, reading and comparing variants: variant, set, get_pointer, fields,
// case naming, `#complete` switches on `which`, and ==.
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

    check(v.which == .S64 && v.s64_ == 42, "an int is the S64 case");

    w: Number_Or_Text;

    check(w.which == .S64 && w.s64_ == 0, "the zero value is the first case, zeroed");

    set(*w, "hi");

    check(w.which == .STRING && w.string_ == "hi", "set switches the case and stores the value");

    set(*w, 7);

    check(w.which == .S64 && w.s64_ == 7, "set switches back");

    set(*w, 8);

    check(w.s64_ == 8, "set on the live case replaces the value");

    s := variant(Shape, Rect.{ 3, 4 });

    check(s.which == .RECT && s.rect.height == 4, "the case is picked by the value's type");

    e := variant(Shape, Empty.{});

    check(e.which == .EMPTY, "a case without a payload");

    // A Variant of one case is just a wrapper.
    single := variant(Variant(string), "only");

    check(single.which == .STRING && single.string_ == "only", "one case");
}

pointers :: () {
    v := variant(Number_Or_Text, "hello");
    pointer := get_pointer(*v, string);

    check(pointer != null && pointer.* == "hello", "get_pointer points at the payload");
    check(get_pointer(*v, s64) == null, "get_pointer is null for another case");

    pointer.* = "changed";

    check(v.string_ == "changed", "a write through get_pointer is seen");
    check(cast(*u8) pointer == v.payload.data, "the pointer is into the variant's own storage");

    s := variant(Shape, Rect.{ 3, 4 });

    get_pointer(*s, Rect).height = 9;

    check(s.rect.height == 9, "write one member through get_pointer");
    check(get_pointer(*s, Empty) == null, "get_pointer of the Empty case while another is live");
}

switching :: () {
    s: Shape;

    hit := "";

    // A `#complete` switch on `which` covers every case and sees the locals around it.
    for i: 0..3 {
        if i == 1 set(*s, Circle.{ 2 });
        if i == 2 set(*s, Rect.{ 3, 4 });
        if i == 3 set(*s, Empty.{});

        if #complete s.which == {
            case .CIRCLE; hit = tprint("%c%", hit, s.circle.radius);
            case .RECT; hit = tprint("%r%", hit, s.rect.width * s.rect.height);
            case .EMPTY; hit = tprint("%e", hit);
        }
    }

    check(hit == "c0c2r12e", "every case, the zero value first");

    // The cases are fields: writing one is seen by the helpers.
    s.which = .RECT;
    s.rect = .{ 5, 6 };

    check(get_pointer(*s, Rect).height == 6, "a case written as a field");
}

naming :: () {
    // Named cases may share a type.
    Contact :: Variant(struct {
        email: string;
        phone: string;
    });

    c: Contact;

    check(c.which == .EMAIL && c.email == "", "a fresh variant is its first case");

    c.which = .PHONE;
    c.phone = "555";

    check(tprint("%", c.which) == "PHONE" && c.phone == "555", "a named case");
    check(Contact.case_names[0] == "email" && Contact.case_count == 2, "case names in order");

    // A type list names each case after its type.
    check(Mixed.case_names[0] == "http_request", "Snake_Case name");
    check(Mixed.case_names[1] == "http_response", "PascalCase name");
    check(Mixed.case_names[2] == "http_server", "an acronym");
    check(Mixed.case_names[3] == "branch_node", "a pointer takes its target's name");
    check(Mixed.case_names[4] == "s64_" && Mixed.case_names[5] == "string_" && Mixed.case_names[6] == "bool_", "built-in types get a trailing _");
    check(cast(s64) Mixed.Which.HTTP_REQUEST == 0 && cast(s64) Mixed.Which.HTTP_SERVER == 2 && cast(s64) Mixed.Which.S64 == 4, "tags count from 0 in case order");

    m: Mixed;

    m.which = .STRING;
    m.string_ = "text";

    check(get_pointer(*m, string).* == "text", "a built-in case by tag and by type");
}

Branch_Node :: struct {
    children: int;
}

Mixed :: Variant(Http_Request, HttpResponse, HTTPServer, *Branch_Node, s64, string, bool);

Http_Request :: struct {
    path: string;
}

HttpResponse :: struct {
    status: int;
}

HTTPServer :: struct {
    port: u16;
}

equality :: () {
    a := variant(Number_Or_Text, 5);
    b := variant(Number_Or_Text, 5);
    c := variant(Number_Or_Text, 6);
    d := variant(Number_Or_Text, "5");

    check(a == b && !(a != b), "equal numbers");
    check(a != c && !(a == c), "different numbers");
    check(a != d, "different cases are never equal");

    none_a: Number_Or_Text;
    none_b: Number_Or_Text;

    check(none_a == none_b && none_a != a, "two zero values are equal");

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
    Pair :: Variant(struct {
        inner: Number_Or_Text;
        pair: [2] int;
    });

    p := variant(Pair, variant(Number_Or_Text, "x"));
    q := variant(Pair, variant(Number_Or_Text, "x"));
    r := variant(Pair, variant(Number_Or_Text, "y"));

    check(p == q && p != r, "a nested variant");

    n1 := variant(Pair, int.[1, 2]);
    n2 := variant(Pair, int.[1, 2]);
    n3 := variant(Pair, int.[1, 3]);

    check(n1 == n2 && n1 != n3, "a fixed array case");

    Numbers :: Variant(struct {
        real: float64;
        list: [..] int;
    });

    list_a := variant(Numbers, cast(float64) 1.5);
    list_b := variant(Numbers, cast(float64) 1.5);

    check(list_a == list_b, "a float case");

    // Floats compare as numbers: 0.0 equals -0.0, and NaN equals nothing.
    zero: float64 = 0;
    nan := zero / zero;

    check(variant(Numbers, zero) == variant(Numbers, -zero), "0.0 equals -0.0");
    check(variant(Numbers, nan) != variant(Numbers, nan), "NaN equals nothing");

    // A view compares element by element, and strings in it by content.
    Words :: Variant(struct {
        view: [] string;
        count: int;
    });

    words_a: [] string = string.["a", "b"];
    words_b: [] string = string.["a", "b"];
    words_c: [] string = string.["a", "c"];

    check(variant(Words, words_a) == variant(Words, words_b) && variant(Words, words_a) != variant(Words, words_c), "a view of strings");

    // A distinct type compares as what it is made from.
    Name :: #type,distinct string;

    Named :: Variant(struct {
        name: Name;
        count: int;
    });

    name_a := cast(Name) "x";
    name_b := cast(Name) "x";
    name_c := cast(Name) "y";

    check(variant(Named, name_a) == variant(Named, name_b) && variant(Named, name_a) != variant(Named, name_c), "a distinct string");

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
    pointers();
    switching();
    naming();
    equality();

    assert(failures == 0);
    print("ok\n");
}
