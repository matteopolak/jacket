#import "Basic";
#import "Compiler";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

main :: () {
    #insert #run rebuild(#code {
        R :: struct #no_padding {
            a: u8;
            b: u32;
        }

        U :: union {
            signed: s64;
            unsigned: u64;
        }

        E :: enum_flags u32 {
            A;
            B;
            BOTH :: A | B;
            #if true {
                EXTRA;
            }
        }

        subtract :: (x: int, y: int = 4) -> int {
            return x - y;
        }

        F :: (int) -> int;
        pointer: *int;
        view: [] int;
        resizable: [..] int;
        r := R.{ a = 3, b = 7 };
        u: U;
        u.signed = -1;
        assert(size_of(R) == 5 && r.a == 3 && r.b == 7);
        assert(u.unsigned == 0xffff_ffff_ffff_ffff);
        assert(E.BOTH == E.A | E.B);
        assert(subtract(9) == 5);
        callback: F;

        assert(callback == null && pointer == null && view.count == 0 && resizable.count == 0);
    });
}
