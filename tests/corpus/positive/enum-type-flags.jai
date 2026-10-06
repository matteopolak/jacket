// Type_Info_Enum.enum_type_flags reports enum_flags (0x1), #complete (0x2) and
// #specified (0x4). Found through The_Way_to_Jai examples/16/16.3_enum_specified.jai.
#import "Basic";

Plain :: enum { A; B; }
Pinned :: enum u16 #specified { A :: 1; B :: 2; }
Bits :: enum_flags u8 { X; Y; }
Pinned_Bits :: enum_flags u32 #specified { X :: 0x10; }

main :: () {
    print("%\n", cast(u16) type_info(Plain).enum_type_flags);
    print("%\n", cast(u16) type_info(Pinned).enum_type_flags);
    print("%\n", cast(u16) type_info(Bits).enum_type_flags);
    print("%\n", cast(u16) type_info(Pinned_Bits).enum_type_flags);
    info := type_info(Pinned);
    if info.enum_type_flags & .SPECIFIED print("specified\n");
}
