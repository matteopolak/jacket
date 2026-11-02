// The zero value of a Variant and the `void` case: a case with no payload, which is how a Variant
// says "nothing".
//
//     jaic stdlib/Extensions/Variant/tests/void-case.jai -interp

#import "Basic";
#import "Extensions/Variant";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

// A case type whose members have defaults of their own.
Rect :: struct {
    width := 5;
    height: int;
}

Maybe_Number :: Variant(void, s64, string);

Contact :: Variant(struct {
    none: void;
    email: string;
    phone: string;
});

Late :: Variant(s64, void);

Named_Empty :: Variant(struct {
    email: string;
    unset: void;
});

Two_Voids :: Variant(struct {
    missing: void;
    unknown: void;
    number: s64;
});

zero_value :: () {
    plain: Variant(Rect, s64);

    check(plain.which == .RECT, "the zero value is the first case");
    check(plain.rect.width == 0 && plain.rect.height == 0, "its payload is zeroed, not set to the type's defaults");

    all_zero := true;

    for plain.payload if it != 0 all_zero = false;

    check(all_zero, "every payload byte is zero");

    list: [2] Variant(Rect, s64);

    check(list[1].which == .RECT && list[1].rect.width == 0, "also in an array");

    heap := New(Variant(s64, Rect));

    check(heap.which == .S64 && heap.s64_ == 0, "also on the heap");
    free(heap);

    // The first case decides, whatever the cases after it have as defaults.
    Strings_First :: Variant(string, Rect);

    text: Strings_First;

    check(text.which == .STRING && text.string_ == "" && text.payload[0] == 0 && text.payload[8] == 0, "a later case's defaults do not leak in");

    late: Late;

    check(late.which == .S64, "a void case that is not first leaves the zero value alone");
}

complete :: () {
    // No case for a state that cannot happen.
    shape: Variant(Rect, s64);
    seen := "";

    for 0..1 {
        if it == 1 set(*shape, cast(s64) 7);

        if #complete shape.which == {
            case .RECT; seen = tprint("%r", seen);
            case .S64; seen = tprint("%s%", seen, shape.s64_);
        }
    }

    check(seen == "rs7", "#complete needs only the real cases");

    maybe: Maybe_Number;

    if #complete maybe.which == {
        case .NONE; seen = "none";
        case .S64; seen = "s64";
        case .STRING; seen = "string";
    }

    check(seen == "none", "#complete over a Variant with a void case");
}

void_cases :: () {
    maybe: Maybe_Number;

    check(maybe.which == .NONE, "a void case first: the zero value is none");
    check(Maybe_Number.case_names[0] == "none" && Maybe_Number.case_count == 3, "the void case is named none");
    check(cast(s64) Maybe_Number.Which.NONE == 0 && cast(s64) Maybe_Number.Which.STRING == 2, "tags count from 0");

    set(*maybe, "text");

    check(maybe.which == .STRING && maybe.string_ == "text", "set with a value");

    set(*maybe);

    check(maybe.which == .NONE, "set without a value picks the void case");

    check(variant(Maybe_Number).which == .NONE, "variant(V) makes the void case");
    check(variant(Maybe_Number, cast(s64) 3).which == .S64, "variant(V, value) as before");

    // Set after other cases leaves no stale bytes.
    set(*maybe, copy_string("owned"));
    free_variant(*maybe);

    check(maybe.which == .NONE, "free_variant resets to the void case");

    check(get_pointer(*maybe, s64) == null, "get_pointer is null while the void case is live");

    // The named form.
    contact: Contact;

    check(contact.which == .NONE, "a void field named none");

    contact.which = .PHONE;
    contact.phone = "555";

    check(contact.which == .PHONE, "the other cases are fields");

    set(*contact);

    check(contact.which == .NONE, "set without a value, named form");

    // A void case under another name has the tag of that name.
    named: Named_Empty;

    check(named.which == .EMAIL, "a void case that is not first: the first case is the zero value");

    set(*named);

    check(named.which == .UNSET, "a void field with its own name");

    late := variant(Late);

    check(late.which == .NONE, "variant(V) with the void case last");

    // Several void cases are picked by name.
    two: Two_Voids;

    two.which = .UNKNOWN;

    check(two.which == .UNKNOWN && two.which != .MISSING, "several void cases");
}

equality :: () {
    a: Maybe_Number;
    b := variant(Maybe_Number);

    check(a == b && !(a != b), "two void cases are equal");
    check(a != variant(Maybe_Number, cast(s64) 0), "a void case is not a number, even zero");
    check(variant(Maybe_Number, "x") != a, "or a string");

    check(variant(Contact) == variant(Contact), "named form");

    one: Two_Voids = .{ which = .MISSING };
    two: Two_Voids = .{ which = .UNKNOWN };

    check(one != two && one == one, "void cases with different tags differ");
}

printing :: () {
    check(variant_to_string(variant(Maybe_Number)) == "none", "a void case prints as none");
    check(variant_to_string(variant(Maybe_Number, cast(s64) 5)) == "5", "other cases print as before");
    check(variant_to_string(variant(Named_Empty, "a@b")) == "a@b", "named form, a value");

    named := variant(Named_Empty);

    check(variant_to_string(named) == "unset", "a void case prints as its name");
    check(tprint("%", named.which) == "UNSET", "and its tag as the tag");

    use_variant_printing();
    defer context.print_style.struct_printer = null;

    check(sprint("%", variant(Maybe_Number)) == "none", "printed through the struct printer");
    check(sprint("[%]", variant(Maybe_Number)) == "[none]", "inside a format string");
}

layout :: () {
    check(size_of(Variant(void, u8)) == 2, "Variant(void, u8): the tag and a byte");
    check(size_of(Variant(void)) == 2, "Variant(void) is just a tag and a spare byte");
    check(align_of(Variant(void, u8)) == 1, "a void case needs no alignment");
    check(align_of(Maybe_Number) == 8, "alignment comes from the other cases");
}

main :: () {
    zero_value();
    complete();
    void_cases();
    equality();
    printing();
    layout();

    assert(failures == 0);
    print("ok\n");
}
