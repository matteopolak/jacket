// test: compile_fail("Json: Doc.contact: `untagged` cannot tell apart two cases of the same type. Drop `untagged` so each case is written with its tag")
#import "Basic";
#import "Extensions/Json";
#import "Extensions/Variant";

// Without tags, two cases of one type could not be told apart.
Doc :: struct {
    contact: Variant(struct {
        email: string;
        phone: string;
    }); @json(untagged)
}

main :: () {
    value: Doc;

    print("%\n", to_json(value));
}
