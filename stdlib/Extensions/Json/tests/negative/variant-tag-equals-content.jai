// test: compile_fail("Json: Doc.shape: `tag=` and `content=` must be different keys")
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
    shape: Variant(Circle, Rect); @json(tag=x,content=x)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
