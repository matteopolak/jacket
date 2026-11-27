// test: compile_fail("set: `string` is the type of more than one case of `Variant(struct { email: string; phone: string; })` (.EMAIL (string), .PHONE (string)). Set the case by name instead: `v.which = .EMAIL; v.email = value;`")
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
