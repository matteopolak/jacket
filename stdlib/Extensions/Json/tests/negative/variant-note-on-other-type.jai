// test: compile_fail("Json: Doc.count: `tag=`, `content=`, `untagged` and `tags=` are for a field that holds a Variant")
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
