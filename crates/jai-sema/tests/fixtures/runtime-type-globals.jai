Stored :: struct { value: int; }
selected: Type = Stored;
unset: Type;
scalar: Type = s32;
main :: () -> int {
    if selected != Stored return 1;
    if scalar != s32 return 2;
    if unset != null return 3;
    header := cast(*Type_Info) selected;
    if header.runtime_size != 8 return 4;
    unset = bool;
    if unset != bool return 5;
    return 42;
}
