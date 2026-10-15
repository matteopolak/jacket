// Independent source-class reductions; no original corpus inputs.
#import "Basic";
#import "Compiler";
#import "Program_Print";

rebuild :: (code: Code) -> Code {
    node := compiler_get_nodes(code);
    if node.kind == .BLOCK for statement: (cast(*Code_Block) node).statements {
        builder: String_Builder;
        printed := print_expression(*builder, statement, skip_parens = true);

        if !printed print("unprinted kind %: %\n", statement.kind, builder_to_string(*builder));

        free_buffers(*builder);
        assert(printed);
    }

    return compiler_get_code(node, code);
}

main :: () -> int {
    #insert #run rebuild(#code {
        sum := 0;

        for item, index: 0..3 {
            sum += item;
            if !(item == index) return 1;
        }

        if !(sum == 6) return 2;

        if sum == {
            case 5, 6;
                sum += 2;
            case;
                sum = -1;
        }

        if !(sum == 8) return 3;

        value := ifx sum > 0 then 11 else 13;
        if !(value == 11) return 4;

        data: [3] int = .[3, 5, 7];
        p := *data[1];

        if !(p.* == 5) return 5;

        data[1] = 9;
        if !(p.* == 9) return 6;

        untouched: int = ---;
        untouched = 17;
        if untouched != 17 return 12;

        first, second: int;
        first, second = 19, 23;
        first =, third: int = 29, 31;
        if first != 29 || second != 23 || third != 31 return 13;

        text := "quote\" slash\\ tab\t nul\x00 high\xff";
        if !(text.count == 29) return 7;

        small := 0.1234567;
        large := 0.123456789;

        if !(type_of(small) == float32 && type_of(large) == float64) return 8;
        if !(small == 0.1234567 && large == 0.123456789) return 9;

        narrowed := cast,no_check(u8) 257;
        if !(narrowed == 1) return 10;

        i := 0;
        while i < 3 {
            defer i += 1;
        }

        if !(i == 3) return 11;
    });

    return 0;
}
