// Canonical bootstrap context declarations.
Name_Mapper :: #type ([] string);

__command_line_arguments: [] *u8;

__runtime_support_disable_stack_trace := false;

FIRST_ADD_CONTEXT :: #code #add_context #as using base: Context_Base;
