// test: compile_fail("Json: Tagged.kind: the key `kind` is also the tag of the Variant it is a case of")
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

Tagged :: struct {
    kind: string;
}

Doc :: struct {
    shape: Variant(Circle, Tagged); @json(tag=kind)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
