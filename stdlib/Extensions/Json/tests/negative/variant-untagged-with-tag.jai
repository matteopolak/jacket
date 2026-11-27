// test: compile_fail("Json: Doc.shape: `untagged` cannot be combined with `tag=` or `content=`")
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
    shape: Variant(Circle, Rect); @json(untagged,tag=kind)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
