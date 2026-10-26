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

Wrapper :: struct @json(transparent) {
    inner: int;
}

Doc :: struct {
    shape: Variant(Circle, Wrapper); @json(tag=kind)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
