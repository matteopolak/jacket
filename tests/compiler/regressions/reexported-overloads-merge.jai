// A module's own procedures and the procedures of a module it re-exports with the same name are
// one overload set for the importer.
#import "Basic";
#import "Reexported_Overload";

main :: () {
    assert(combine(1, 2, 3) == 6);
    assert(combine(1, "ab") == 3);

    print("ok\n");
}
