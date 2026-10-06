#import "Basic";
#import "String";

kind :: (x: $T) -> string {
    #if T == u8 return "u8";
    else return "other";
}

main :: () {
    assert(kind(#char "a") == "u8");
    assert(kind(97) == "other");
    parts := split("a,b,c", #char ",");
    assert(parts.count == 3 && parts[2] == "c");
    print("ok\n");
}
