Error :: struct {
    code: enum {
        Success :: 0;
        DidNotWait;
        FullQueue;
        FileAccessFailed;
        Incomplete;
        Catastrophic;
    };
    platform_code: s64;
}

error :: (code: type_of(Error.code), platform_code: $T = 0) -> Error {
    ret: Error = ---;
    ret.code = code;
    ret.platform_code = xx platform_code;
    return ret;
}

main :: () -> int {
    value := error(.Catastrophic, 37);
    return cast(int)value.code + value.platform_code;
}
