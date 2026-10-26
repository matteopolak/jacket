#import "Basic";
#import "Extensions/Json";
#import "Extensions/Variant";

Circle :: struct {
    radius: int;
}

Rect :: struct {
    width: int;
    height: int;
}

Doc :: struct {
    count: int; @json(tag=kind)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
