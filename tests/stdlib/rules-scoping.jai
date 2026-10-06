// Checks the rules of docs/language/scoping.md (with modules/Rules_Scoping).
// rules: scope.1 scope.2 scope.3 scope.4 scope.5 scope.6 scope.7 scope.10 scope.11 scope.12 scope.13 scope.14 scope.15 scope.16
#import "Basic";
using RS :: #import "Rules_Scoping";

Color :: enum {
    RED;
    GREEN;
    BLUE;
}

#if OS == .MACOS {
    PLATFORM_NAME :: "mac";
} else {
    PLATFORM_NAME :: "other";
}

#if #exists(exported_answer) HAS_ANSWER :: true;
else HAS_ANSWER :: false;

#if #exists(no_such_declaration_anywhere) HAS_MISSING :: true;
else HAS_MISSING :: false;

LEVEL_AT_COMPILE_TIME :: #run context.rules_scoping_level;

uses_body_import :: () -> float {
    #import "Math";
    return 1;
}

later_math_user :: () -> float {
    return PI;
}

seen: *void;
Remember :: () #expand {
    seen = cast(*void) #this;
}
remembering :: () {
    Remember();
}

Holder :: struct {
    field: u16;
    field_size :: () -> int {
        return size_of(type_of(field));
    }
}

main :: () {
    // scope.1: exported by default, both qualified and through using.
    assert(RS.exported_answer == 42 && exported_answer == 42, "scope.1");

    // scope.2: module-scope names are visible in both files of the module.
    assert(second_reads_module_value() == 101, "scope.2");

    // scope.3: each file has its own file-scope names; no clash.
    assert(first_total() == 100 + 101 + 3, "scope.3 first");
    assert(second_file_value() == 30, "scope.3 second");

    // scope.5: using an enum makes members unqualified.
    using Color;
    assert(GREEN == Color.GREEN, "scope.5");

    // scope.6: using a local struct makes its fields plain names.
    Pair :: struct {
        left: int;
        right: int;
    }
    pr := Pair.{ 2, 3 };
    {
        using pr;
        assert(left + right == 5, "scope.6");
    }

    // scope.7: the module alias and its members both work.
    assert(RS.first_root() == 9 && first_root() == 9, "scope.7");

    // scope.10: an import inside a procedure body serves the whole file.
    assert(later_math_user() > 3.14 && later_math_user() < 3.15, "scope.10");

    // scope.11, scope.12: top-level #if with braces and else; without braces.
    #if OS == .MACOS assert(PLATFORM_NAME == "mac", "scope.11");
    else assert(PLATFORM_NAME == "other", "scope.11 else");
    assert(RS.LOADED_PART, "scope.12 module #if with else");

    // scope.13: #exists.
    assert(HAS_ANSWER && !HAS_MISSING, "scope.13");

    // scope.14: the conditionally loaded #add_context is there for #run.
    assert(LEVEL_AT_COMPILE_TIME == 5 && context.rules_scoping_level == 5, "scope.14");

    // scope.15: #this inside a macro is the procedure it was expanded into.
    remembering();
    assert(seen == cast(*void) remembering, "scope.15");

    // scope.16: type_of(field) inside a procedure nested in a struct.
    assert(Holder.field_size() == 2, "scope.16");

    print("ok\n");
}
