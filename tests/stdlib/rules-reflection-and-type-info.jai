// rules: reflect.1 reflect.2 reflect.3 reflect.4 reflect.5 reflect.6 reflect.7 reflect.8 reflect.10 reflect.11
// Claims of docs/metaprogramming/reflection-and-type-info.md, one group per rule.
#import "Basic";
#import "Compiler";

Shade :: enum u8 {
    DARK;
    MID;
    LIGHT;
}

Point3 :: struct {
    px: float;
    py: float;
    label: string;
}

Box :: struct(Item: Type, N: int) {
    slots: [N] Item;
}

Meters :: #type,distinct float64;

only_in_table :: struct {
    q: u16;
}

main :: () {
    // reflect.1: type_info(T) is a pointer to the descriptor; asking twice gives the same one.
    a := type_info(Point3);
    b := type_info(Point3);
    assert(a != null && a == b, "reflect.1: one descriptor per type");

    // reflect.2: a Type value at run time is the descriptor's address.
    t: Type = Point3;
    assert(cast(*Type_Info) t == cast(*Type_Info) a, "reflect.2: Type is the descriptor address");

    // reflect.3: the descriptor struct matches the kind of type.
    assert(type_info(s32).type == .INTEGER && type_info(s32).signed, "reflect.3: integer");
    assert(!type_info(u8).signed, "reflect.3: unsigned integer");
    assert(type_info(float32).type == .FLOAT && type_info(float32).runtime_size == 4, "reflect.3: float");
    assert(type_info(string).type == .STRING, "reflect.3: string");
    assert(type_info(*u16).pointer_to == cast(*Type_Info) type_info(u16), "reflect.3: pointer target");
    proc := type_info(#type (int, bool) -> float);
    assert(proc.type == .PROCEDURE && proc.argument_types.count == 2 && proc.return_types.count == 1, "reflect.3: procedure");
    assert(type_info([3] u8).array_count == 3, "reflect.3: array count");
    assert(type_info([3] u8).element_type == cast(*Type_Info) type_info(u8), "reflect.3: array element");
    assert(type_info(Shade).values.count == 3, "reflect.3: enum");
    assert(type_info(Meters).type == .VARIANT && type_info(Meters).variant_of == cast(*Type_Info) type_info(float64), "reflect.3: distinct type is a variant");
    assert(type_info(bool).type == .BOOL && type_info(bool).runtime_size == 1, "reflect.3: plain Type_Info for bool");

    // reflect.4: every descriptor converts to *Type_Info through its header.
    header: *Type_Info = type_info(Point3);
    assert(header.type == .STRUCT && header.runtime_size == size_of(Point3), "reflect.4: header via #as");

    // reflect.5: the documented struct and enum descriptors.
    assert(a.name == "Point3" && a.runtime_size == 24 && a.members.count == 3, "reflect.5: struct");
    assert(a.members[0].name == "px" && a.members[0].offset_in_bytes == 0 && a.members[0].type.type == .FLOAT, "reflect.5: px");
    assert(a.members[1].name == "py" && a.members[1].offset_in_bytes == 4 && a.members[1].type.type == .FLOAT, "reflect.5: py");
    assert(a.members[2].name == "label" && a.members[2].offset_in_bytes == 8 && a.members[2].type.type == .STRING, "reflect.5: label");
    e := type_info(Shade);
    assert(e.names.count == 3 && e.names[0] == "DARK" && e.names[1] == "MID" && e.names[2] == "LIGHT", "reflect.5: enum names");
    assert(e.values[0] == 0 && e.values[1] == 1 && e.values[2] == 2, "reflect.5: enum values");

    // reflect.6: pointer and array tags.
    assert(type_info(*int).type == .POINTER, "reflect.6: pointer tag");
    assert(type_info([4] int).type == .ARRAY, "reflect.6: array tag");

    // reflect.7: members carry name, offset and type.
    assert(a.members[2].type == cast(*Type_Info) type_info(string), "reflect.7: member type pointer");

    // reflect.8: polymorphic instances record their parameters and source struct.
    box := type_info(Box(u8, 5));
    assert(box.specified_parameters.count == 2, "reflect.8: specified parameters");
    assert(box.specified_parameters[0].name == "Item" && box.specified_parameters[1].name == "N", "reflect.8: parameter names");
    assert(box.polymorph_source_struct != null, "reflect.8: polymorph source");
    assert(box.polymorph_source_struct.name == "Box", "reflect.8: source names the generic struct");

    // reflect.10: the runtime type table lists the program's descriptors.
    table := get_type_table();
    found := false;
    for table if it == cast(*Type_Info) type_info(only_in_table) found = true;
    assert(found, "reflect.10: type table contains a used struct");

    // reflect.11: the table is filled once every descriptor exists, so it holds ones made late too.
    late := false;
    for get_runtime_info().type_table if it == cast(*Type_Info) type_info(Box(u8, 5)) late = true;
    assert(late, "reflect.11: type table contains a polymorph instance");

    print("ok\n");
}
