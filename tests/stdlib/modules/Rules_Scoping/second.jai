// Second file of Rules_Scoping: sees the module-scope names of module.jai, has its own
// file-scope names.

second_reads_module_value :: () -> int {
    return module_only_value + 1;
}

second_file_value :: () -> int {
    return file_only_value;
}

#scope_file

file_only_value :: 30;
