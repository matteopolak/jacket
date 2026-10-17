// Original general reproducer; no Bindings_Generator dependency.
#module_parameters(VALUE: int = 0);
#add_context original_context_value: int;

set_value :: () {
    context.original_context_value = VALUE;
}
