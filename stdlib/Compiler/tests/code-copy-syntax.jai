#import "Basic";
C :: #import "Compiler";

copy :: (code: Code) -> Code {
    root := C.compiler_get_nodes(code);
    return C.compiler_get_code(root, code);
}

main :: () {
    #insert #run copy(#code {
        Pair :: struct {
            left: s64;
            right: s64;
        }

        Choice :: enum {
            FIRST;
            SECOND;
        }

        value := Pair.{ 2, 3 };

        using,only(left) value;
        assert(left == 2);

        twice :: (x: s64) -> s64 {
            return x * 2;
        }

        sum: s64;

        for item, index: s64.[1, 2, 3] {
            if index == 1 continue;
            sum += twice(item);
        }

        while sum < 10 sum += 1;

        a, b := twice(2), twice(3);

        assert(a == 4 && b == 6 && sum == 10);
        assert((a + b) * 2 == 20);
        selected := ifx sum == 10 then Choice.SECOND else .FIRST;
        if selected == {
            case .FIRST; assert(false);
            case .SECOND; sum += cast(s64) cast(u32) 1;
        }

        assert(sum == 11);
        assert(cast,no_check(u8) 257 == 1);
    });
    print("ok\n");
}
