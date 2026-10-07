// A runtime field read cannot bind a baked parameter: `Push(*arr, xx p.t)` picks the `elem` overload.
#import "Basic";

S :: struct {
    t: *Type_Info;
}

Push :: (arr: *[..] $T, $initialize := true) -> *T {
    exit(1);
    return null;
}

Push :: (arr: *[..] $T, elem: T) -> *T {
    array_add(arr, elem);
    return null;
}

main :: () {
    s := S.{ type_info(int) };
    p := *s;
    arr: [..] *Type_Info_Struct;
    Push(*arr, xx p.t);
}
