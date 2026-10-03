Node :: struct { next: *Node; value: int; }
main :: () -> int {
    pointer_type: Type = *Node;
    info := type_info(*Node);
    header := cast(*Type_Info) pointer_type;
    if cast(*void) header != cast(*void) info return 1;
    if cast(u32)header.type != 4 return 2;
    nested := cast(*Type_Info_Pointer) header;
    if cast(*void) nested.pointer_to != cast(*void) type_info(Node) return 3;
    actual: Type = Node;
    if cast(*void) cast(*Type_Info)actual != cast(*void)nested.pointer_to return 4;
    if type_of(actual) != Type return 5;
    return 42;
}
