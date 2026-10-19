// rules: scope.3
// A #scope_file name is not visible to the other files of its module.
#import,dir "../../compiler/regressions/modules/Rules_Scoping_File_Leak";
main :: () {
    x := leak();
}
