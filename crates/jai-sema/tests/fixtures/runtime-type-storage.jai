First :: struct { value: int; }
Second :: struct { value: int; }
Holder :: struct { tag: Type; kinds: [2]Type; }
main :: () -> int {
    holder: Holder;
    if holder.tag != null return 1;
    if holder.kinds[0] != null return 2;
    holder.tag = First;
    holder.kinds = .[s32, Second];
    if holder.tag != First return 3;
    if holder.tag == holder.kinds[1] return 4;
    if holder.kinds[0] != s32 return 5;
    tag_pointer := *holder.tag;
    tag_pointer.* = Second;
    if holder.tag != Second return 6;
    copy := holder;
    if copy.kinds[1] != holder.tag return 7;
    return 42;
}
