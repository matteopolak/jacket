#import "Basic";
#import "Compiler";
PP :: #import "Program_Print";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

main :: () {
    #insert #run rebuild(#code {
        add :: ($bias: int, value: $T, rest: ..T) -> T #no_abc #no_aoc #modify {
            if T != int return false;
        } {
            sum := value + cast(T) bias;
            for rest sum += it;
            return sum;
        }

        O :: struct {
            x: int;
        }

        R :: struct {
            #as using base: O;
            tail: int;
        }

        r: R;
        r.x = 9;
        assert(r.base.x == 9);
        assert(add(2, 3, 4, 5) == 14);
        tail: [2] int = .[4, 5];

        assert(add(2, 3, ..tail) == 14);
        one: s64 = 1;
        shifted := (cast(u64) one) << 63;

        assert(type_of(shifted) == u64 && shifted == 0x8000_0000_0000_0000);
        lambda := (x: int) => x + 3;

        assert(lambda(8) == 11);
        count := 0;
        while: outer count < 5 {
            count += 1;
            if count == 3 break outer;
        }

        assert(count == 3);

        while active := count < 5 {
            assert(active);
            count += 1;
        }

        assert(count == 5);
        #assert(4 < 9, "assertion %", 7);

        computed :: #run -> int {
            value := 11;
            return value;
        };

        assert(computed == 11);
        snapshot := context;

        push_context, defer_pop snapshot;
        assert(type_of(context) == #Context);
    });

    // Inspection options must not truncate compiler source reconstruction.
    #run {
        PP.print_procedure_bodies = false;
    }

    procedure :: #insert #run rebuild(#code (x: int) -> int {
        return x * 2;
    });

    assert(procedure(9) == 18);
}
