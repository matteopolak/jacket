// rules: scope.4
// A module's #scope_file #import does not leak to the importer.
#import,dir "../../stdlib/modules/Rules_Scoping";
main :: () {
    x := sqrt(4.0);
}
