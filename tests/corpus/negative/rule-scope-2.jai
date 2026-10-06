// rules: scope.2
// A #scope_module name is not visible to an importer of the module.
RS :: #import,dir "../../stdlib/modules/Rules_Scoping";
main :: () {
    x := RS.module_only_value;
}
