#import "Basic";

Record :: struct {
    left: int;
    right: int;
    extra: int;
    other: int;
}

main :: () {
    records: [..] Record;

    defer array_free(records);
    array_add(*records, .{ 1, 2, 3, 4 });
    record := pop(*records);

    records.count = records.allocated;
    array_add(*records, record);
    consume(record);
}

consume :: (record: Record) {
    print("%", record);
}
