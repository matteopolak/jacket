Record :: struct {
    field: int;
    label: string;
}

square :: (number: int) -> int {
    return number * number;
}

main :: () -> int {
    before := 11;
    text := "😀 café";
    record: Record;
    record.field = square(before + text.count);
    if record.field > 0 {
        nested := record.field + before;
        record.field = nested;
    }

    for index: 0..2 {
        record.field += index;
    }

    after := record.field;

    if after < before return before;

    return after;
}

last :: () -> bool {
    local := false;
    result := !local;
    return result || local;
}
