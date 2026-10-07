// Checks the claims of docs/language/directives-and-notes.md, one group per rule.
// rules: note.1 note.2 note.11 note.12
#import "Basic";
#import "Compiler";

// note.1: `#run,stallable` runs like `#run`.
stalled_value :: () -> int {
    return 6 * 7;
}

STALLED :: #run,stallable stalled_value();

#run,stallable {
    assert(STALLED == 42, "stallable run at compile time");
}

// note.2: both #assert forms, each with an optional message.
#assert size_of(u32) == 4;
#assert size_of(u32) == 4 "u32 is four bytes";
#assert(size_of(u16) == 2);
#assert(size_of(u16) == 2, "u16 is two bytes");

// note.11: unrecognised notes do not change behaviour.
Noted :: struct @Whatever @"free text" {
    a: int @Unknown;
    b: int;
}

noted_add :: (x: int, y: int) -> int {
    return x + y;
} @SomeNote
Plain :: struct {
    a: int;
    b: int;
}

// note.9 (struct half): struct notes reach metaprograms as Code_Struct.notes. The enum half
// (`enum_flags @Hi { ... }` giving Code_Enum.notes) is pending: jaic does not parse notes on an enum.
PROGRAM :: #string END
Box :: struct @Boxed { v: int; }
main :: () {}
END

#run {
    w := compiler_create_workspace("notes");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(PROGRAM, w);
    struct_ok := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .TYPECHECKED {
            checked := cast(*Message_Typechecked) message;
            for checked.declarations {
                e := it.expression.expression;
                if !e continue;
                if it.expression.name == "Box" && e.kind == .STRUCT {
                    notes := (cast(*Code_Struct) e).notes;
                    struct_ok = notes.count == 1 && notes[0].text == "Boxed";
                }
            }
        }
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
    assert(struct_ok, "Code_Struct.notes");
}

main :: () {
    // note.11
    n := Noted.{ 1, 2 };
    assert(n.a + n.b == 3, "noted struct works");
    assert(size_of(Noted) == size_of(Plain), "notes take no space");
    assert(type_info(Noted).members.count == 2, "notes add no members");
    assert(noted_add(2, 3) == 5, "noted procedure works");

    // note.12: an empty statement is a statement; `if cond ;` has an empty body.
    answer := true;
    reached := 0;
    if answer;
    reached += 1;
    ;;
    if !answer;
    else reached += 1;
    assert(reached == 2, "empty statements");

    print("ok\n");
}
