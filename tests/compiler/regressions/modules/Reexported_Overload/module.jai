using #import "Reexported_Overload_Base";

combine :: (a: s32, text: string) -> s64 {
    return a + text.count;
}
