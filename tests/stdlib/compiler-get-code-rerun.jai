// A `#run` that asks for typed nodes is run again from the start; `compiler_get_code` on the
// same tree must give the same code in each run, so `code_to_string(compiler_get_code(root))`
// inside one expression finishes instead of asking for new nodes forever.
// rules: records.29
#import "Basic";
#import "Compiler";

#run {
    code :: #code value :: 42;
    root, expressions := compiler_get_nodes(code);
    print("nodes % %\n", expressions.count, code_to_string(compiler_get_code(root)));
    rendered := code_to_string(compiler_get_code(root));
    assert(rendered == "value :: 42", "rendered: %", rendered);
}

main :: () {
    print("ok\n");
}
