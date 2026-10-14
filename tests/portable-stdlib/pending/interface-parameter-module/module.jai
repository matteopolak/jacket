#module_parameters(HOOKS: $I/interface Required = Required) {
    Required :: struct {
        run :: inline (value: int) {}
    }
}

touch :: () -> int {
    return 7;
}
