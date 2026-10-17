/* Independently authored official libclang SDK evidence; no reference inputs.
 * Proves an unused macro can be evaluated in a second TU preserving unsaved text.
 * This is native API evidence, not a public Jai implementation or macro policy.
 */
#include <clang-c/Index.h>
#include <stdio.h>
#include <string.h>

struct Capture { CXCursor declaration; };

static enum CXChildVisitResult capture(CXCursor cursor, CXCursor parent,
                                      CXClientData opaque) {
    (void)parent;
    struct Capture *state = opaque;
    CXString spelling = clang_getCursorSpelling(cursor);
    const char *text = clang_getCString(spelling);
    if (text && strcmp(text, "independent_evaluation") == 0)
        state->declaration = cursor;
    clang_disposeString(spelling);
    return CXChildVisit_Continue;
}

int main(void) {
    const char *original = "#define ORIGINAL_UNUSED_VALUE (29 + 13)\n";
    struct CXUnsavedFile input = {
        "stdlib/Bindings_Generator/tests/driven/bindings-unsaved-evaluation-sdk.h", original, (unsigned long)strlen(original)
    };
    const char *arguments[] = { "-x", "c", "-std=c11" };
    CXIndex index = clang_createIndex(0, 0);
    if (!index) return 1;
    CXTranslationUnit source = NULL;
    if (clang_parseTranslationUnit2(index, input.Filename, arguments, 3,
                                   &input, 1, CXTranslationUnit_DetailedPreprocessingRecord,
                                   &source) != CXError_Success) return 2;
    CXFile file = clang_getFile(source, input.Filename);
    size_t bytes = 0;
    const char *retained = clang_getFileContents(source, file, &bytes);
    if (!retained || bytes != strlen(original) || memcmp(retained, original, bytes)) return 3;
    const char suffix[] = "\nconst int independent_evaluation = ORIGINAL_UNUSED_VALUE;\n";
    char probe[4096];
    if (bytes > sizeof(probe) - sizeof(suffix)) return 8;
    memcpy(probe, retained, bytes);
    memcpy(probe + bytes, suffix, sizeof(suffix));
    struct CXUnsavedFile evaluation_input = {
        input.Filename, probe, (unsigned long)strlen(probe)
    };
    CXTranslationUnit evaluation = NULL;
    if (clang_parseTranslationUnit2(index, input.Filename, arguments, 3,
                                   &evaluation_input, 1, 0, &evaluation) != CXError_Success) return 4;
    if (clang_getNumDiagnostics(evaluation)) return 5;
    struct Capture state = { clang_getNullCursor() };
    clang_visitChildren(clang_getTranslationUnitCursor(evaluation), capture, &state);
    CXEvalResult result = clang_Cursor_Evaluate(state.declaration);
    if (!result || clang_EvalResult_getKind(result) != CXEval_Int) return 6;
    long long value = clang_EvalResult_getAsLongLong(result);
    clang_EvalResult_dispose(result);
    clang_disposeTranslationUnit(evaluation);
    /* The copied scalar remains usable after the temporary evaluation TU dies. */
    if (value != 42) return 7;
    clang_disposeTranslationUnit(source);
    clang_disposeIndex(index);
    puts("ok");
    return 0;
}
