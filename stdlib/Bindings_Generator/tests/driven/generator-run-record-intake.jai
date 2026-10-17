// Original public constructor reduction: ignored top-level types retain normal
// record member construction. No run/private model replacement is involved.
#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    cursor, parent: B.CXCursor;
    saved: #Context;
}

visit :: (cur: B.CXCursor, parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context, defer_pop;
    context = capture.saved;

    if cur.kind == .StructDecl {
        capture.cursor = cur;
        capture.parent = parent;
    }

    return .Continue;
}

main :: () {
    push_context, defer_pop;
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    generator.clang_index = B.clang_createIndex(0, 0);
    source := "struct Intake { int member; };\n";
    name := to_c_string("owned-generator-run-intake.c");

    defer free(name);
    overlay: B.CXUnsavedFile;
    overlay.Filename = name;
    overlay.Contents = source.data;
    overlay.Length = cast(type_of(overlay.Length)) source.count;
    error := B.clang_parseTranslationUnit2(generator.clang_index, name, null, 0, *overlay, 1, 0, *generator.translation_unit);

    assert(error == .Success && generator.translation_unit != null);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), visit, *capture);
    declaration := B.create_declaration(capture.cursor, capture.parent, ignore_non_structs = true);

    assert(declaration != null && declaration.kind == .STRUCT);
    record := cast(*B.Struct) declaration;

    assert(record.declarations.count == 1);
    assert(record.declarations[0].name == "member");
    print("record-intake ok\n");
}
