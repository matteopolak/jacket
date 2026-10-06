// A struct literal's `null` replaces the bytes a union member's default put in the same
// storage (sjorsdonkers/toml-jai's `Guy.{mytag = .BANANA, banana = null}`).
// rules: struct.19
#import "Basic";

Plain :: struct {
    using u: union {
        p: *s64;
        s: string = "default";
    };
}

Tagged :: struct {
    using fruit: union tag : enum {
        BANANA;
        ORANGE;
    } = .ORANGE {
        .BANANA,, banana: *s64 = ---;
        .ORANGE,, orange: string = "default";
    };
}

main :: () {
    plain := Plain.{ p = null };
    assert(plain.p == null);
    tagged := Tagged.{ tag = .BANANA, banana = null };
    assert(tagged.tag == .BANANA && tagged.banana == null);
    constant :: Plain.{ p = null };
    assert(constant.p == null);
    print("ok\n");
}
