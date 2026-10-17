// Module for tests/compiler/regressions/module-parameter-instances.jai.
#module_parameters(LOUD := false);

is_loud :: () -> bool {
    return LOUD;
}
