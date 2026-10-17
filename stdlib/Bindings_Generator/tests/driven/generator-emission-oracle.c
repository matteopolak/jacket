/* Original SDK oracle: no generated or reference implementation input. */
#include <clang-c/Index.h>
#include <stdio.h>

static enum CXChildVisitResult inspect(CXCursor cursor, CXCursor parent,
                                      CXClientData opaque) {
    (void)parent;
    if (clang_getCursorKind(cursor) != CXCursor_VarDecl)
        return CXChildVisit_Continue;
    unsigned *count = opaque;
    ++*count;
    CXString kind = clang_getCursorKindSpelling(clang_getCursorKind(cursor));
    CXString name = clang_getCursorSpelling(cursor);
    unsigned line = 0, column = 0;
    clang_getSpellingLocation(clang_getCursorLocation(cursor), NULL, &line,
                             &column, NULL);
    printf("%s %s %u:%u\n", clang_getCString(kind), clang_getCString(name), line,
           column);
    CXString raw = clang_Cursor_getRawCommentText(cursor);
    CXSourceRange range = clang_Cursor_getCommentRange(cursor);
    line = column = 0;
    clang_getSpellingLocation(clang_getRangeStart(range), NULL, &line, &column,
                             NULL);
    const char *comment = clang_getCString(raw);
    printf("comment [%s] %u:%u\n", comment ? comment : "", line, column);
    printf("definition=%u canonical=%u self_definition=%u\n",
           clang_isCursorDefinition(cursor),
           clang_equalCursors(cursor, clang_getCanonicalCursor(cursor)),
           clang_equalCursors(cursor, clang_getCursorDefinition(cursor)));
    clang_disposeString(raw);
    clang_disposeString(name);
    clang_disposeString(kind);
    return CXChildVisit_Continue;
}

struct walk { unsigned count, depth, maximum_depth; };
static enum CXChildVisitResult descendants(CXCursor cursor, CXCursor parent,
                                          CXClientData opaque) {
    (void)parent;
    struct walk *walk = opaque;
    ++walk->count;
    if (walk->depth > walk->maximum_depth) walk->maximum_depth = walk->depth;
    ++walk->depth;
    clang_visitChildren(cursor, descendants, walk);
    --walk->depth;
    return CXChildVisit_Continue;
}

int main(int argc, char **argv) {
    if (argc != 2) return 2;
    CXIndex index = clang_createIndex(0, 0);
    if (!index) return 3;
    CXTranslationUnit unit = NULL;
    if (clang_parseTranslationUnit2(index, argv[1], NULL, 0, NULL, 0, 0,
                                   &unit) != CXError_Success) {
        clang_disposeIndex(index);
        return 4;
    }
    unsigned count = 0;
    unsigned diagnostics = clang_getNumDiagnostics(unit);
    clang_visitChildren(clang_getTranslationUnitCursor(unit), inspect, &count);
    struct walk walk = {0};
    clang_visitChildren(clang_getTranslationUnitCursor(unit), descendants, &walk);
    printf("debug descendants=%u depth=%u\n", walk.count, walk.maximum_depth);
    clang_disposeTranslationUnit(unit);
    clang_disposeIndex(index);
    return diagnostics || count != 5;
}
