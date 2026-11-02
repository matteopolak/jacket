// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

// Two cases of one type: picking a case by type is ambiguous.
Contact :: Variant(struct {
    email: string;
    phone: string;
});

main :: () {
    c: Contact;
    set(*c, "me@example.com");
}
