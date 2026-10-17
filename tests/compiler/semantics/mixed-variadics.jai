trace: int;
tail: [2] int = .[2, 3];
empty: [0] int;
empty_view: [] int;
texts: [1] string = .["bc"];

mark :: (n: int) -> int {
    trace = trace * 10 + n;
    return n;
}

spread :: () -> [] int {
    trace = trace * 10 + 2;
    return tail;
}

encode :: (items: ..int) -> int {
    result: int;
    for items result = result * 10 + it;
    return result;
}

generic :: (items: ..$T) -> T {
    result: T;
    for items result = result * 10 + it;
    return result;
}

defaults :: (bias: int = 5, items: ..int, suffix: int = 7) -> int {
    return bias * 10000 + encode(..items) * 10 + suffix;
}

expanded :: (items: ..int) -> int #expand {
    return encode(..items);
}

verify_strings :: (items: ..string) -> bool {
    return items.count == 3 && items[0].count == 1 && items[0][0] == #char "a"
        && items[1][1] == #char "c" && items[2].count == 3 && items[2][2] == #char "f";
}

#program_export "variadic_check"
check :: () -> s32 #c_call {
    push_context {
        if encode(1, ..tail) != 123 return 1;
        if encode(..tail, 4, ..empty, 5) != 2345 return 2;
        if encode(..empty, 1) != 1 return 3;
        if generic(1, ..tail, 4) != 1234 return 4;
        if defaults(items = 1, ..tail, suffix = 9) != 51239 return 5;
        if expanded(1, ..tail) != 123 return 6;

        trace = 0;
        if encode(mark(1), ..spread(), mark(3)) != 1233 || trace != 123 return 7;
        if encode(1, ..empty_view, 2) != 12 return 8;
        if !verify_strings("a", ..texts, "def") return 9;
    }

    return 0;
}
