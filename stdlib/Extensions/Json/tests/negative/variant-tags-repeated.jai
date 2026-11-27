// test: compile_fail("Json: Doc.shape: the cases `Circle` and `Rect` are both tagged `same`. Give them `tags=` of their own")
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
    shape: Variant(Circle, Rect); @json(tags=same,same)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
