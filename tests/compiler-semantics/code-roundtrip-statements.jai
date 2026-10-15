// Original statement-root reconstruction and public compound/exists printing.
#import "Basic";
#import "Compiler";
#import "Program_Print";

pair :: () -> int, int {
    return 7, 9;
}

total: int;

rebuild :: (code: Code) -> Code {
    node := compiler_get_nodes(code);
    builder: String_Builder;
    assert(print_expression(*builder, node, skip_parens = true));
    return compiler_get_code(node, code);
}

declare :: () #expand {
    code :: #code left, `right := pair();
    copy :: #run rebuild(code);
    #insert copy;
}

main :: () {
    #insert #run rebuild(#code total += 7);
    assert(total == 7);
    existing := 1;

    #insert #run rebuild(#code {
        first, second := pair();
        existing =, made := pair();
        x, y: int = 11, 13;
    });
    assert(first == 7 && second == 9 && existing == 7 && made == 9);
    assert(x == 11 && y == 13);
    query := #insert #run rebuild(#code #exists(total));

    assert(query);
    declare();
    assert(right == 9);
}
