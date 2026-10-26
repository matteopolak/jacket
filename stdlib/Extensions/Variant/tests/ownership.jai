// free_variant releases what the live case owns and zeroes the variant; printing writes the
// payload.
//
//     jaic stdlib/Extensions/Variant/tests/ownership.jai -interp

#import "Basic";
#import "Extensions/Variant";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Tally :: struct {
    outstanding: int;
}

tally_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    tally := cast(*Tally) data;
    backing := context.default_allocator;
    result := backing.proc(mode, size, old_size, old_memory, backing.data);

    if mode == .ALLOCATE && result {
        tally.outstanding += 1;
    } else if mode == .RESIZE && result && !old_memory {
        tally.outstanding += 1;
    } else if mode == .FREE && old_memory {
        tally.outstanding -= 1;
    }

    return result;
}

Named :: struct {
    name: string;
    tags: [..] string;
    count: int;
}

Inner :: Variant(string, [..] int);
Outer :: Variant(int, Inner, Named);

Empty :: struct {}

Mixed :: Variant(Empty, string, [..] string, Named, Outer, [2] string, [] int);

freeing :: () {
    tally: Tally;
    cleared := true;

    {
        push_allocator(tally_proc, *tally);

        text := variant(Variant(int, string), copy_string("owned"));

        free_variant(*text);
        cleared = cleared && text.index == 0 && unwrap(text, int) == 0;

        // Freeing twice, or freeing a number, is harmless.
        free_variant(*text);

        number := variant(Variant(int, string), 5);

        free_variant(*number);
        cleared = cleared && number.index == 0 && unwrap(number, int) == 0;

        list: [..] string;

        array_add(*list, copy_string("a"));
        array_add(*list, copy_string("b"));

        strings := variant(Mixed, list);

        free_variant(*strings);
        cleared = cleared && strings.index == 0;

        named := Named.{ name = copy_string("n") };

        array_add(*named.tags, copy_string("t1"));
        array_add(*named.tags, copy_string("t2"));

        in_struct := variant(Mixed, named);

        free_variant(*in_struct);
        cleared = cleared && in_struct.index == 0;

        // A variant inside a variant, and a string inside that.
        deep := variant(Outer, variant(Inner, copy_string("deep")));

        free_variant(*deep);
        cleared = cleared && deep.index == 0;

        numbers: [..] int;

        array_add(*numbers, 1);

        deeper := variant(Outer, variant(Inner, numbers));

        free_variant(*deeper);

        // A fixed array owns its strings.
        pair := variant(Mixed, string.[copy_string("x"), copy_string("y")]);

        free_variant(*pair);

        // The case without a payload.
        none := variant(Mixed, Empty.{});

        free_variant(*none);
    }

    check(tally.outstanding == 0, tprint("free_variant left % blocks allocated", tally.outstanding));
    check(cleared, "a freed variant is zeroed: its first case, zeroed");
}

// Views and pointers are not owned: free_variant must leave what they point to alone.
borrowed :: () {
    numbers := int.[1, 2, 3];
    slice_of_numbers: [] int = numbers;
    view := variant(Mixed, slice_of_numbers);

    free_variant(*view);

    check(numbers[2] == 3, "a view's elements are untouched");

    literal := variant(Variant(int, string), "a literal");

    // Not freed here: a literal was never allocated. Resetting is the caller's job.
    literal = .{};

    check(literal.index == 0, "a variant can be reset by assigning the zero value");
}

printing :: () {
    use_variant_printing();
    defer context.print_style.struct_printer = null;

    number := variant(Variant(int, string), 42);
    text := variant(Variant(int, string), "hello");

    check(sprint("%", number) == "42", "a number prints as itself");
    check(sprint("%", text) == "hello", "a string prints as itself");
    check(sprint("<%>", text) == "<hello>", "inside a format string");
    check(variant_to_string(text) == "hello", "variant_to_string");

    pair := variant(Variant(Empty, int), Empty.{});

    check(sprint("%", pair) == "{}", "an empty struct prints as an empty struct");

    named := variant(Mixed, Named.{ name = "n", count = 3 });

    check(sprint("%", named) == "{\"n\", [], 3}", "a struct payload prints as the struct does");

    // A variant inside a struct, and inside another variant.
    Holder :: struct {
        label: string;
        value: Variant(int, string);
    }

    holder := Holder.{ "h", number };

    check(sprint("%", holder) == "{\"h\", 42}", "a variant member prints as its payload");

    nested := variant(Outer, variant(Inner, "inner text"));

    check(sprint("%", nested) == "inner text", "a variant inside a variant");

    // Calling it again changes nothing.
    use_variant_printing();

    check(sprint("%", number) == "42", "use_variant_printing twice");
}

other_printer :: (builder: *String_Builder, any: Any, data: *void) -> bool {
    if any.type == type_info(Empty) {
        append(builder, "<empty>");

        return true;
    }

    return false;
}

chaining :: () {
    // A printer installed before is still asked about everything that is not a variant.
    context.print_style.struct_printer = other_printer;
    use_variant_printing();
    defer context.print_style.struct_printer = null;

    check(sprint("%", Empty.{}) == "<empty>", "the earlier printer still works");
    check(sprint("%", variant(Variant(int, string), 5)) == "5", "variants print");

    // Outside the printer's reach, variant_to_string always prints the payload and puts things back.
    context.print_style.struct_printer = null;

    check(variant_to_string(variant(Variant(int, string), "x")) == "x", "variant_to_string without the printer");
    check(context.print_style.struct_printer == null, "variant_to_string restores the printer");
}

main :: () {
    freeing();
    borrowed();
    printing();
    chaining();

    assert(failures == 0);
    print("ok\n");
}
