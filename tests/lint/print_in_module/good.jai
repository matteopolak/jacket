#import "Basic";

describe :: (text: string) -> string {
    return tprint("parsing %", text);
}

fill :: (builder: *String_Builder, text: string) {
    print_to_builder(builder, "parsing %", text);
    print(builder, " (%)", text.count);
    append(builder, "\n");
}

report :: (text: string) {
    log("parsing %", text);
}

// A name that merely resembles the library's.
print_all :: (items: [] string) -> int {
    return items.count;
}

count :: (items: [] string) -> int {
    return print_all(items);
}
