/* Independent SDK oracle: evaluated declaration initializers in C and C++. */
#include <clang-c/Index.h>
#include <string.h>
typedef struct { unsigned variables; int failed; } State;
static enum CXChildVisitResult visit(CXCursor cursor, CXCursor parent, CXClientData data) {
  (void)parent;
  State *state = data;
  if (clang_getCursorKind(cursor) == CXCursor_VarDecl) {
    CXEvalResult value = clang_Cursor_Evaluate(cursor);
    if (!value || clang_EvalResult_getKind(value) != CXEval_StrLiteral) state->failed = 1;
    else if (!clang_EvalResult_getAsStr(value) || strcmp(clang_EvalResult_getAsStr(value), "value")) state->failed = 1;
    ++state->variables;
    if (value) clang_EvalResult_dispose(value);
  }
  return CXChildVisit_Recurse;
}
int main(void) {
  const char *text = "const char *direct = \"value\"; void probe(void) { __typeof__((\"value\")) local = (\"value\"); (void)(\"value\"); }";
  struct CXUnsavedFile file = {"independent-string-evaluation.c", text, (unsigned long)strlen(text)};
  CXIndex index = clang_createIndex(0, 0);
  CXTranslationUnit unit = NULL;
  for (unsigned mode = 0; mode < 2; ++mode) {
    const char *arguments[] = {"-x", mode ? "c++" : "c", mode ? "-std=c++17" : "-std=c11"};
    if (clang_parseTranslationUnit2(index, file.Filename, arguments, 3, &file, 1, 0, &unit) != CXError_Success) return 1;
    if (clang_getNumDiagnostics(unit)) return 2;
    State state = {0};
    clang_visitChildren(clang_getTranslationUnitCursor(unit), visit, &state);
    clang_disposeTranslationUnit(unit);
    if (state.failed || state.variables != 2) return 3;
  }
  clang_disposeIndex(index);
  return 0;
}
