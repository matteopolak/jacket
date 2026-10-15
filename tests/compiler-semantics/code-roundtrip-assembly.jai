// Parser/serialization coverage on every host. No optional ISA execution claim.
#import "Basic";
#import "Compiler";
#import "Program_Print";

#run {
    code := #code #asm AVX512F, AVX512VL {
        a: gpr === a;
        scratch: vec === 3;
        mov.q a, [base + index*8 - 16];
        mov?u64 output, a;
        vaddps.x result!z &* mask, left, [data]!;
        lock_xadd.q [counter], a;
        pause;
    };

    original := compiler_get_nodes(code);
    copy := compiler_get_nodes(compiler_get_code(original, code));

    assert(original.kind == .ASM && copy.kind == .ASM);
    first, second: String_Builder;

    assert(print_expression(*first, original, skip_parens = true));
    assert(print_expression(*second, copy, skip_parens = true));
    a := builder_to_string(*first);
    b := builder_to_string(*second);

    assert(a == b, "all assembly operands/decorations must survive parsing again");
    free(a);
    free(b);
    free_buffers(*first);
    free_buffers(*second);
}

main :: () {}
