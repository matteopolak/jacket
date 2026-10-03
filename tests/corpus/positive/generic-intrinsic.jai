compare_and_swap :: (pointer: *$T, old: T, new: T) -> (success: bool, old_value: T) #intrinsic;
main :: () -> int {
    value: int = 3;
    success, previous := compare_and_swap(*value, 3, 7);
    if success && previous == 3 return value;
    return 0;
}
