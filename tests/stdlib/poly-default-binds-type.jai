// A polymorphic parameter whose argument is left out takes its type from the default value
// (Focus's File_Async: `error :: (code: ..., platform_code: $T = 0)` called with one argument).
Result :: struct {
    code: int;
    platform: s64;
    size: int;
}

error :: (code: int, platform_code: $T = 0) -> Result {
    return .{ code, cast(s64) platform_code, size_of(T) };
}

scaled :: (x: $T = 2.5, factor := 2) -> T {
    return x * cast(T) factor;
}

main :: () -> s64 {
    defaulted := error(3);
    if defaulted.code != 3 || defaulted.platform != 0 || defaulted.size != 8 return 1;
    given := error(4, cast(u16) 9);
    if given.platform != 9 || given.size != 2 return 2;
    if scaled() != 5.0 return 3;
    if type_of(scaled()) != float32 return 4;
    if scaled(3) != 6 return 5;
    return 0;
}
