#import "Basic";
#import "String";

Entry :: struct {
    name: string;
}

current_label: string;
registry: [..] string;

set_label :: (id: int) {
    // Copied out of temporary storage first.
    current_label = copy_string(tprint("item %", id));
}

rename :: (entry: *Entry, id: int) {
    entry.name = sprint("entry %", id);
}

register :: (id: int) {
    array_add(*registry, copy_string(tprint("id %", id)));
}

local_only :: (id: int) -> int {
    // A member of a local struct lives no longer than the procedure.
    entry: Entry;
    entry.name = tprint("entry %", id);

    text := tprint("<%>", id);

    return entry.name.count + text.count;
}

// The note tells callers the result is temporary.
describe :: (id: int) -> string {
    return tprint("<%>", id);
} @TempAllocates

owned :: (id: int) -> string {
    return sprint("<%>", id);
}

main :: () {
    entry: Entry;

    set_label(1);
    rename(*entry, 2);
    register(3);
    print("% % % % % %\n", current_label, entry.name, registry.count, describe(4), owned(5), local_only(6));
}
