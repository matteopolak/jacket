#import "Basic";

saved: string;

Pair :: struct {
    owned: string;
    temporary: string;
}

saved_pair: Pair;

mixed :: () -> string, string {
    scratch := tprint("%", 7);
    return copy_string(scratch), scratch;
} @TempAllocates

owned :: () -> string {
    return copy_string(tprint("%", 8));
} @TempAllocates

context_temporary :: () -> string {
    push_allocator(temp);
    return copy_string("temporary");
} @TempAllocates

other_context :: () -> string {
    push_context {
        context.logger_data = null;
        return copy_string("owned");
    }
} @TempAllocates

restored_context :: () -> string {
    {
        push_allocator(temp);
        scratch := tprint("scratch");
        assert(scratch.count > 0);
    }

    return copy_string("owned");
} @TempAllocates

explicit_heap :: () -> string {
    push_allocator(temp);
    push_allocator(context.default_allocator);
    return copy_string("owned");
} @TempAllocates

forward :: (value: string) -> string {
    return value;
} @TempAllocates

pair :: () -> Pair {
    return .{ copy_string("owned"), tprint("temporary") };
} @TempAllocates

unnoted :: () -> string {
    return tprint("temporary");
}

good_unnoted :: () {
    text := unnoted();
    print("%", text);
}

good_mixed :: () {
    heap, temporary := mixed();
    saved = heap;
}

good_single :: () {
    saved = owned();
}

good_context :: () {
    saved = other_context();
}

good_restored :: () {
    saved = restored_context();
}

good_explicit_heap :: () {
    saved = explicit_heap();
}

good_parameter :: () {
    saved = forward(copy_string("owned"));
}

good_field :: () {
    value := pair();
    saved = value.owned;
}

bad_temporary :: () {
    heap, temporary := mixed();
    saved = temporary;
}

bad_context :: () {
    saved = context_temporary();
}

bad_parameter :: () {
    saved = forward(tprint("temporary"));
}

bad_field :: () {
    value := pair();
    saved = value.temporary;
}

bad_pair :: () {
    saved_pair = pair();
}

bad_unnoted :: () {
    saved = unnoted();
}
