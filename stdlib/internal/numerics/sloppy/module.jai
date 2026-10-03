is_approximately_zero :: (f: float, epsilon := DEFAULT_EPSILON) -> bool {
    return Math.abs(f) <= epsilon;
}

values_are_close :: (f0: float, f1: float, epsilon := DEFAULT_EPSILON) -> bool {
    return Math.abs(f0 - f1) <= epsilon;
}

values_are_close :: (a: Math.Vector2, b: Math.Vector2, epsilon := DEFAULT_EPSILON) -> bool {
    return values_are_close(a.x, b.x, epsilon) && values_are_close(a.y, b.y, epsilon);
}

values_are_close :: (a: Math.Vector3, b: Math.Vector3, epsilon := DEFAULT_EPSILON) -> bool {
    return values_are_close(a.x, b.x, epsilon) && values_are_close(a.y, b.y, epsilon) && values_are_close(a.z, b.z, epsilon);
}

max_of_type :: inline ($T: Type) -> T {
    #if T == float32 return Math.FLOAT32_MAX;
    else #if T == float64 return Math.FLOAT64_MAX;
    else {
        info := cast(*Type_Info_Integer) T;
        assert(info.type == .INTEGER, "numeric type required");
        if info.signed return cast(T) (((cast(u64) 1) << (size_of(T) * 8 - 1)) - 1);
        return cast,trunc(T) 0xffff_ffff_ffff_ffff;
    }
}

min_of_type :: inline ($T: Type) -> T {
    #if T == float32 return Math.FLOAT32_MIN;
    else #if T == float64 return Math.FLOAT64_MIN;
    else {
        info := cast(*Type_Info_Integer) T;
        assert(info.type == .INTEGER, "numeric type required");
        if info.signed return cast,trunc(T) ((cast(u64) 1) << (size_of(T) * 8 - 1));
        return 0;
    }
}

#scope_file
DEFAULT_EPSILON :: 0.0001;

Math :: #import "Math";
#import "Basic";
