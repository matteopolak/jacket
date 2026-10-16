#import "Basic";

Record :: struct {
    descriptor: Type;
    count: int;
}

Box :: struct(T: Type) {
    item: T;
}

Distinct_Record :: #type,distinct Record;
Distinct_Type :: #type,distinct Type;

Outer :: struct {
    child: Record;
    values: [2] Type;
    records: [2][2] Record;
    generic: Box(Type);
}

global_scalar: Type;
global_record: Outer;

main :: () {
    scalar: Type;
    value: Outer;
    distinct: Distinct_Record;
    distinct_type: Distinct_Type;
    override: Record = .{ descriptor = u8, count = 19 };
    descriptor_pointer: *Type_Info;
    empty: [0] Type;

    assert(scalar == void && global_scalar == void);
    assert(value.child.descriptor == void && value.child.count == 0);
    assert(global_record.child.descriptor == void && global_record.values[1] == void);
    assert(value.values[0] == void && value.values[1] == void);
    assert(value.records[0][0].descriptor == void && value.records[1][1].descriptor == void);
    assert(value.generic.item == void && global_record.generic.item == void);
    ordinary := cast(Record) distinct;

    assert(ordinary.descriptor == void && cast(Type) distinct_type == void);
    assert(override.descriptor == u8 && override.count == 19);
    assert(descriptor_pointer == null && empty.count == 0);

    initialized: Record = ---;
    info := type_info(Record);

    info.initializer(cast(*void) *initialized);
    assert(initialized.descriptor == void && initialized.count == 0);

    print("%\n", scalar);
    print("%\n", value.child);
}
