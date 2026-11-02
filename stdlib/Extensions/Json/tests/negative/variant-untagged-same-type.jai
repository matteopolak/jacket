// test: compile_fail
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
