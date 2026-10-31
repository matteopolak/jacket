// A procedure's notes follow its body (`f :: () {} @a:b`). The declaration an identifier
// resolves to carries them, as does the procedure header.
// rules: compiler.15 records.10
#import "Basic";
#import "Compiler";

plain :: (x: int) {} @after:other @without:Poison

inlined :: inline (x: int) {} @alpha:one @beta:two

single :: (x: int) {} @gamma

none :: (x: int) {}

notes_of :: ($name: Code) -> [..] string #expand {
    return #run -> [..] string {
        root := compiler_get_nodes(name);

        assert(root.kind == .IDENT);
        declaration := (cast(*Code_Ident) root).resolved_declaration;

        assert(declaration != null);

        result: [..] string;
        for declaration.notes array_add(*result, copy_string(it.text));

        // The header sees the same ones.
        header := cast(*Code_Procedure_Header) declaration.expression;

        assert(header.notes.count == result.count);

        return result;
    };
}

#run {
    plain_notes := notes_of(plain);

    assert(plain_notes.count == 2);
    assert(plain_notes[0] == "after:other");
    assert(plain_notes[1] == "without:Poison");

    inlined_notes := notes_of(inlined);

    assert(inlined_notes.count == 2);
    assert(inlined_notes[0] == "alpha:one");
    assert(inlined_notes[1] == "beta:two");

    assert(notes_of(single).count == 1);
    assert(notes_of(none).count == 0);
}

main :: () {
    print("ok\n");
}
