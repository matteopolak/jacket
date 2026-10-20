// An untyped integer constant that does not fit the other `ifx` arm's integer type makes the
// result an `s64`; one that fits keeps the arm's type.
#import "Basic";

main :: () {
    yes := true;
    no := false;
    small: u8 = 1;

    wide := ifx yes then 300 else small;

    assert(type_of(wide) == s64 && wide == 300);
    wide_else := ifx no then small else 300;

    assert(type_of(wide_else) == s64 && wide_else == 300);
    narrow := ifx no then 300 else small;

    assert(type_of(narrow) == s64 && narrow == 1);
    fits := ifx yes then 200 else small;

    assert(type_of(fits) == u8 && fits == 200);

    print("ok\n");
}
