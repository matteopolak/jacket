// VERBOSE is a program parameter: every importer shares one instance.
#module_parameters ()(VERBOSE := false);

Message :: struct {
    text: string;
}

is_verbose :: () -> bool {
    return VERBOSE;
}
