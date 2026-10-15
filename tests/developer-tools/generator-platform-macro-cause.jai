// Original diagnostic reduction; intentionally exits 1 while the callback bug remains.
#import "Basic";
S :: #import "String";
B :: #import "Bindings_Generator";

specific: bool;
generic: bool;
errors: s64;

Probe :: struct {
    expressions: u32;
    evaluations: u32;
    strings: u32;
}

inspect :: (cursor: B.CXCursor, _parent: B.CXCursor, data: *void) -> B.CXChildVisitResult #c_call {
    if B.clang_isExpression(cursor.kind) == 0 return .Recurse;

    type := B.clang_getCursorType(cursor);
    if type.kind == .Void return .Recurse;

    probe := cast(*Probe) data;
    probe.expressions += 1;
    result := B.clang_Cursor_Evaluate(cursor);
    if result {
        probe.evaluations += 1;
        B.clang_EvalResult_dispose(result);
    }

    if cursor.kind == .StringLiteral probe.strings += 1;

    return .Recurse;
}

report :: (text: string, _data: *void, info: Log_Info) {
    if !(info.common_flags & .ERROR) return;
    errors += 1;
    if S.contains(text, "not a supported numeric constant expression") specific = true;
    if S.contains(text, "callback model materialization failed") generic = true;
}

visit :: (_decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    return .RECURSE;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string("original-platform-macro.c");

    defer free(filename);
    source :: "#define ORIGINAL_STRING \"value\"\nvoid probe(void) { (void)(ORIGINAL_STRING); }\n";
    contents := to_c_string(source);

    defer free(contents);
    unsaved: B.CXUnsavedFile = .{ Filename = filename, Contents = contents };
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, *unsaved, 1, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    probe: Probe;

    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), inspect, *probe);
    assert(probe.expressions == 3 && probe.evaluations == 0 && probe.strings == 1);
    print("expressions=% evaluations=% diagnostics=0\n", probe.expressions, probe.evaluations);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-macro-string.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "original", identifier = "native" });
    defer array_free(options.libraries);
    options.add_generator_command = false;

    push_context {
        context.logger = report;
        normal, normal_ok := B.generate_bindings(options);
        defer free_buffers(*normal);
        assert(!normal_ok && specific && !generic && errors > 0);
    }

    specific = false;
    errors = 0;
    options.visitor = visit;

    push_context {
        context.logger = report;
        callback, callback_ok := B.generate_bindings(options);
        defer free_buffers(*callback);
        assert(!callback_ok && errors > 0);
    }

    print("callback-specific=% callback-generic=%\n", specific, generic);
    if !specific exit(1);
}
