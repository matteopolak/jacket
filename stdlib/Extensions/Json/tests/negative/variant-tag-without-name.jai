// test: compile_fail("Json: Doc.shape: `tag=` needs a key name, as in `@json(tag=kind)`")
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
    shape: Variant(Circle, Rect); @json(tag=)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
