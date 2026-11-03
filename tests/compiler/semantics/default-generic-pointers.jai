// Original default-type/pointer constraint regression, independent of any module.
Parcel :: struct(T: Type = void) {
    data: *T;
    count: int;
}

Alias :: Parcel;

Dependent :: struct(T: Type = u16, P: Type = *T) {
    data: P;
}

Nested :: struct(T: Type = Parcel(void)) {
    data: *T;
}

Required :: struct(T: Type) {
    data: *T;
    count: int;
}

read_count :: (pointer: *$P/Parcel) -> int #no_context {
    return pointer.count;
}

write_count :: (pointer: *Parcel, count: int) #no_context {
    pointer.count = count;
}

explicit_count :: (pointer: *Parcel(void)) -> int #no_context {
    return pointer.count;
}

required_count :: (pointer: *$P/Required) -> int #no_context {
    return pointer.count;
}

#program_export "generic_pointer_check"
check :: () -> s32 #c_call {
    implicit: Parcel;
    explicit: Parcel(void);
    implicit_pointer: *Alias = *implicit;
    explicit_pointer: *Parcel(void) = implicit_pointer;

    #assert type_of(implicit_pointer) == type_of(explicit_pointer);
    #assert **Alias == **Parcel(void);
    implicit_call :: #procedure_of_call read_count(implicit_pointer);
    explicit_call :: #procedure_of_call read_count(explicit_pointer);

    #assert implicit_call == explicit_call;
    write_count(implicit_pointer, 19);
    if read_count(implicit_pointer) != 19 || explicit_count(explicit_pointer) != 19 return 1;

    write_count(*explicit, 23);
    if read_count(*explicit) != 23 return 2;

    nondefault: Parcel(u32);
    item: u32 = 37;

    nondefault.data = *item;
    write_count(*nondefault, 41);
    if read_count(*nondefault) != 41 || nondefault.data.* != 37 return 3;

    dependent: Dependent;
    dependent_pointer: *Dependent(u16, *u16) = *dependent;

    #assert type_of(dependent_pointer) == *Dependent(u16);
    small: u16 = 43;

    dependent_pointer.data = *small;
    if dependent.data.* != 43 return 4;

    nested: Nested(Parcel);

    #assert type_of(nested) == Nested(Parcel(void));
    nested.data = implicit_pointer;
    if read_count(nested.data) != 19 return 5;

    required: Required(u8);
    required.count = 47;
    if required_count(*required) != 47 return 6;

    return 0;
}

main :: () -> s32 {
    return check();
}
