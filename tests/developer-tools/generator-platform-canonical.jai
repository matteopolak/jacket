#import "Basic";
B :: #import "Bindings_Generator";
H :: #import "Hash_Table";

#assert size_of(B.Token_Info) == 16;
#assert offset_of(B.Token_Info, "token_pointer") == 0;
#assert offset_of(B.Token_Info, "num_tokens") == 8;

main :: () {
    generator: B.Generator;
    library: B.Library_Info;
    symbols: B.Library_Symbols;
    token: B.CXToken;
    tokens: B.Token_Info;

    assert(generator.libraries.count == 0 && generator.tokens_to_free.count == 0);
    assert(generator.tab_depth == 0 && generator.template_depth == 0);
    assert(generator.counts.functions == 0 && !generator.translation_unit);
    symbols.os = OS;
    symbols.cpu = CPU;
    symbols.full_path = "canonical object";
    symbols.symbol_names = string.["original"];
    library.symbol_names = symbols.symbol_names;
    array_add(*generator.libraries, *library);
    defer array_free(generator.libraries);
    assert(generator.libraries[0] == *library);
    assert(generator.libraries[0].symbol_names.data == symbols.symbol_names.data);
    tokens.token_pointer = *token;
    tokens.num_tokens = 1;
    array_add(*generator.tokens_to_free, tokens);
    defer array_free(generator.tokens_to_free);
    assert(generator.tokens_to_free[0].token_pointer == *token);

    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string("original-canonical-state.h");

    defer free(filename);
    source :: "extern int original_canonical_value;\n";
    contents := to_c_string(source);

    defer free(contents);
    unsaved: B.CXUnsavedFile = .{ Filename = filename, Contents = contents };
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, *unsaved, 1, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    root := B.clang_getTranslationUnitCursor(unit);
    declaration: B.Declaration;

    H.table_add(*generator.declarations_by_cursor, root, *declaration);
    found, same := H.table_find(*generator.declarations_by_cursor, root);

    assert(found && same == *declaration);
    assert(generator.declarations_by_cursor.hash_function(root) == B.clang_hashCursor(root));
    assert(generator.declarations_by_cursor.compare_function(root, root));
    H.table_reset(*generator.declarations_by_cursor);
    print("ok\n");
}
