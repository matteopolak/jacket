/* Original public-SDK oracle: a discarded string expression is not evaluated. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <clang-c/Index.h>

static unsigned evaluated;
static unsigned expressions;
static unsigned literals;

static enum CXChildVisitResult inspect(CXCursor cursor, CXCursor parent, CXClientData data) {
    (void)parent;
    (void)data;
    if (!clang_isExpression(clang_getCursorKind(cursor))) return CXChildVisit_Recurse;
    if (clang_getCursorType(cursor).kind == CXType_Void) return CXChildVisit_Recurse;
    CXEvalResult result = clang_Cursor_Evaluate(cursor);
    unsigned kind = result ? (unsigned)clang_EvalResult_getKind(result) : 0;
    printf("expression=%u type=%u evaluated=%u\n", (unsigned)clang_getCursorKind(cursor),
           (unsigned)clang_getCursorType(cursor).kind, kind);
    ++expressions;
    if (clang_getCursorKind(cursor) == CXCursor_StringLiteral) {
        assert(clang_getCursorType(cursor).kind == CXType_ConstantArray);
        CXString spelling = clang_getCursorSpelling(cursor);
        assert(strcmp(clang_getCString(spelling), "\"value\"") == 0);
        clang_disposeString(spelling);
        ++literals;
    }
    if (result) ++evaluated;
    if (result) clang_EvalResult_dispose(result);
    return CXChildVisit_Recurse;
}

int main(void) {
    const char source[] = "#define ORIGINAL_STRING \"value\"\n"
                          "void probe(void) { (void)(ORIGINAL_STRING); }\n";
    struct CXUnsavedFile input = {"original-platform-macro.c", source, sizeof source - 1};
    CXIndex index = clang_createIndex(0, 0);
    assert(index != NULL);
    CXTranslationUnit unit = NULL;
    assert(clang_parseTranslationUnit2(index, input.Filename, NULL, 0, &input, 1, 0, &unit) == CXError_Success);
    assert(unit != NULL && clang_getNumDiagnostics(unit) == 0);
    clang_visitChildren(clang_getTranslationUnitCursor(unit), inspect, NULL);
    assert(evaluated == 0 && expressions == 3 && literals == 1);
    printf("expressions=%u evaluations=%u diagnostics=0\n", expressions, evaluated);
    clang_disposeTranslationUnit(unit);
    clang_disposeIndex(index);
    return 0;
}
