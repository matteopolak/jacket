identity :: (value: Type) -> Type { return value; }
unused :: (value: Type) {}
main :: () -> int {
    unset: Type;
    if unset != null return 1;
    first: Type = float64;
    second := first;
    third := identity(s32);
    unused(bool);
    if first != float64 return 2;
    if second != first return 3;
    if third == first return 4;
    info := cast(*Type_Info) third;
    if cast(u32)info.type != 0 return 5;
    if info.runtime_size != 4 return 6;
    unset = Type;
    if cast(*Type_Info)unset != cast(*Type_Info)type_info(Type) return 7;
    return 42;
}
