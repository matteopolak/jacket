// Small language rules newer libraries rely on (AST_Utils, Jai-Shader-Transpiler).
#import "Basic";
#import "Compiler";
#import "Math";

// Any expression passed to a plain (non-macro) `Code` parameter becomes code.
code_kind :: (code: Code) -> bool { return true; }

Op :: enum { PLUS; MINUS; }
strict :: (op: Op) -> int { return xx op; }

main :: () {
    assert(code_kind(1 + 2 * 3));

    // `value.(T, trunc)` takes cast flags after the type.
    big: s64 = 0x1_0000_0005;
    assert(big.(u32, trunc) == 5);

    // `E.loose` converts to `E`.
    loose: Op.loose = .MINUS;
    assert(strict(loose) == 1);

    // `xx a == b` casts to the other operand's type.
    err: s32 = 0;
    assert(xx err == false);

    // Matrices list their elements first: positional literals fill _11, _12, ...
    m := Matrix2.{1, 2, 3, 4};
    assert(m._12 == 2 && m._21 == 3 && m.v[1].y == 4 && m.floats[0] == 1);
}
