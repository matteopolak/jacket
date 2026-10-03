Holder :: struct { tag: Type; kinds: [2] Type; }
choose :: () -> Type { return s32; }
SELECTED :: #run choose();
make_holder :: () -> Holder {
    holder: Holder;
    holder.tag = bool;
    holder.kinds[0] = u8;
    holder.kinds[1] = float64;
    return holder;
}
main :: () -> int {
    value: Type = #run choose();
    if value != s32 return 1;
    selected: Type = SELECTED;
    if selected != s32 return 6;
    holder := #run make_holder();
    if holder.tag != bool return 2;
    if holder.kinds[0] != u8 return 3;
    if holder.kinds[1] != float64 return 4;
    info := cast(*Type_Info) holder.kinds[1];
    if info.runtime_size != 8 return 5;
    return 42;
}
