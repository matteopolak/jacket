// compiler_get_code turns an edited node tree back into Code that resolves names where
// the original code was written.
// rules: compiler.15 records.12 code.8
#import "Basic";
#import "Compiler";

double_literals :: (code: Code) -> Code {
    root, expressions := compiler_get_nodes(code);
    for expressions {
        if it.kind != .LITERAL continue;
        literal := cast(*Code_Literal) it;
        if literal.value_type == .NUMBER literal._s64 *= 2;
    }
    return compiler_get_code(root);
}

apply :: (code: Code) #expand {
    changed :: #run double_literals(code);
    #insert changed;
}

total := 1;

main :: () -> s64 {
    step :: #code {
        total += 5;
    };
    apply(step);
    apply(step);
    if total != 21 {
        print("total = %\n", total);
        return 1;
    }
    return 0;
}
