Record :: struct {
    field: int;
    label: string;
}
square :: (number: int) -> int {
    return number * number;
}
main :: () {
    before := 11;
    text := "😀 café";
    record: Record;
    record.field = square(before);
    if record.field > 0 {
        nested := record.field + before;
        record.field = nested;
    }
    for index: 0..2 {
        record.field += index;
    }
    after := record.field;
}
last :: () {
    local := false;
    result := local;
}
