Item :: struct {
    number: s64;
}

Container :: struct(Element: Type, projection: (Element) -> s64) {
    value: Element;
    apply :: projection;
}

Specialized :: Container(Item, item => item.number);
Named :: Container(Item, project);
Named_Again :: Container(Item, project);

project :: (item: $T) -> s64 {
    return item.number;
}

#assert Named == Named_Again;

Default_Container :: struct(Element: Type, projection: (Element) -> s64 = project) {
    value: Element;
    apply :: projection;
}

Defaulted :: Default_Container(Item);
Explicit :: Default_Container(Item, project);
#assert Defaulted == Explicit;

choose :: (item: string) -> s64 {
    return -1;
}

choose :: (item: Item) -> s64 {
    return item.number + 1;
}

Overloaded :: Container(Item, choose);

inspect :: () -> bool {
    info := cast(*Type_Info_Struct) type_info(Specialized);
    if info.specified_parameters.count != 2 return false;

    parameter := info.specified_parameters[1];
    if parameter.type.type != .PROCEDURE || parameter.offset_into_constant_storage < 0 return false;

    proc_info := cast(*Type_Info_Procedure) parameter.type;
    if proc_info.argument_types.count != 1 || proc_info.argument_types[0] != type_info(Item) return false;
    if proc_info.return_types.count != 1 || proc_info.return_types[0] != type_info(s64) return false;

    Callback :: (Item) -> s64;
    callback: Callback = (cast(*Callback) (info.constant_storage.data + parameter.offset_into_constant_storage)).*;
    value: Specialized;
    any: Any = value;

    return any.type.runtime_size == 8 && callback(.{ 19 }) == 19 && Specialized.apply(.{ 23 }) == 23
        && Named.apply(.{ 29 }) == 29 && Defaulted.apply(.{ 31 }) == 31
        && Overloaded.apply(.{ 36 }) == 37;
}

#assert inspect();

main :: () -> int {
    if !inspect() return 1;
    return 0;
}
