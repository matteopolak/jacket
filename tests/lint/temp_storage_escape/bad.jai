#import "Basic";

Entry :: struct {
    name: string;
}

current_label: string;
registry: [..] string;
title := tprint("title %", 1);

set_label :: (id: int) {
    // Gone after the next reset.
    current_label = tprint("item %", id);
}

rename :: (entry: *Entry, id: int) {
    name := tprint("entry %", id);
    entry.name = name;
}

register :: (id: int) {
    array_add(*registry, tprint("id %", id));
}

describe :: (id: int) -> string {
    return tprint("<%>", id);
}

main :: () {
    entry: Entry;

    set_label(1);
    rename(*entry, 2);
    register(3);
    print("% % % %\n", current_label, entry.name, registry.count, describe(4));
    print("%\n", title);
}
