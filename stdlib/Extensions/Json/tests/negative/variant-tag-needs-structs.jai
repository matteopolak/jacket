// test: compile_fail("Json: Doc.shape: `tag=kind` needs every case to be a struct, and `s64` is an integer. Use `tag=kind,content=c` to tag a case that is not a struct")
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
    shape: Variant(s64, Circle); @json(tag=kind)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
