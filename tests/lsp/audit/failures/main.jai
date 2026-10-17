Record :: struct {
    value: int;
}

main :: () {
    record: Record;
    invalid := record.missing;
    valid := record.value;
}
