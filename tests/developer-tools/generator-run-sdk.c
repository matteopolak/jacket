/* Original CIndex reduction of the top-level test's native premises. */
#include <clang-c/Index.h>
#include <stdio.h>
#include <string.h>

struct Observations {
    unsigned hidden_system, included_system, same, changed, functions, argument_system, predefined_system;
    unsigned linkage_children, semantic_linkage, ancestor_tu;
    unsigned equivalent_overloads;
    CXCursor first_function;
    CXCursor first_overload;
    int canonical_identity;
    int overloads_distinct;
};

static enum CXChildVisitResult visit(CXCursor cursor, CXCursor parent,
                                    CXClientData data) {
    struct Observations *observed = data;
    CXString spelling = clang_getCursorSpelling(cursor);
    const char *name = clang_getCString(spelling);
    enum CXCursorKind kind = clang_getCursorKind(cursor);
    if (name && strcmp(name, "RunHidden") == 0)
        observed->hidden_system += clang_Location_isInSystemHeader(clang_getCursorLocation(cursor)) != 0;
    if (name && strcmp(name, "RunIncluded") == 0)
        observed->included_system += clang_Location_isInSystemHeader(clang_getCursorLocation(cursor)) != 0;
    if (kind == CXCursor_MacroDefinition && name) {
        if (strcmp(name, "__clang__") == 0)
            observed->predefined_system += clang_Location_isInSystemHeader(clang_getCursorLocation(cursor)) != 0 &&
                clang_Cursor_isMacroBuiltin(cursor) == 0;
        if (strcmp(name, "RUN_ARGUMENT") == 0)
            observed->argument_system += clang_Location_isInSystemHeader(clang_getCursorLocation(cursor)) != 0;
        observed->same += strcmp(name, "RUN_SAME") == 0;
        observed->changed += strcmp(name, "RUN_CHANGED") == 0;
    }
    if (kind == CXCursor_FunctionDecl && name && strcmp(name, "run_function") == 0) {
        CXCursor canonical = clang_getCanonicalCursor(cursor);
        if (observed->functions == 0) observed->first_function = canonical;
        else observed->canonical_identity = clang_equalCursors(canonical, observed->first_function) != 0;
        observed->functions++;
        observed->linkage_children += clang_getCursorKind(parent) == CXCursor_LinkageSpec;
        CXCursor semantic = clang_getCursorSemanticParent(cursor);
        observed->semantic_linkage += clang_getCursorKind(semantic) == CXCursor_LinkageSpec;
        observed->ancestor_tu += clang_getCursorKind(clang_getCursorSemanticParent(semantic)) == CXCursor_TranslationUnit;
    }
    if (kind == CXCursor_FunctionDecl && name && strcmp(name, "run_equivalent") == 0) {
        CXCursor canonical = clang_getCanonicalCursor(cursor);
        if (observed->equivalent_overloads == 0) observed->first_overload = canonical;
        else observed->overloads_distinct = clang_equalCursors(canonical, observed->first_overload) == 0;
        observed->equivalent_overloads++;
    }
    clang_disposeString(spelling);
    return CXChildVisit_Recurse;
}

int main(int argc, char **argv) {
    if (argc != 2) return 2;
    CXIndex index = clang_createIndex(0, 0);
    CXTranslationUnit unit = NULL;
    const char *arguments[] = {"-DRUN_ARGUMENT=40"};
    enum CXErrorCode error = clang_parseTranslationUnit2(index, argv[1], arguments, 1,
        NULL, 0, CXTranslationUnit_DetailedPreprocessingRecord, &unit);
    if (error != CXError_Success || !unit) return 3;
    struct Observations observed = {0};
    clang_visitChildren(clang_getTranslationUnitCursor(unit), visit, &observed);
    int valid = clang_getNumDiagnostics(unit) == 0 && observed.hidden_system == 1 &&
        observed.included_system == 1 && observed.same == 2 && observed.changed == 2 &&
        observed.functions == 2 && observed.canonical_identity &&
        observed.linkage_children == 2 && observed.semantic_linkage == 2 && observed.ancestor_tu == 2 &&
        observed.argument_system == 0 && observed.predefined_system == 1 &&
        observed.equivalent_overloads == 2 && observed.overloads_distinct;
    printf("system=%u/%u macros=%u/%u functions=%u canonical=%d linkage=%u semantic_linkage=%u ancestor_tu=%u argument_system=%u predefined_system=%u overloads_distinct=%d\n",
        observed.hidden_system, observed.included_system, observed.same, observed.changed,
        observed.functions, observed.canonical_identity, observed.linkage_children,
        observed.semantic_linkage, observed.ancestor_tu, observed.argument_system, observed.predefined_system,
        observed.overloads_distinct);
    clang_disposeTranslationUnit(unit);
    clang_disposeIndex(index);
    return valid ? 0 : 4;
}
