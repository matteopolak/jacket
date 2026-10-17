/* Original public SDK function-pointer/qualified-type oracle. */
#include <clang-c/Index.h>
#include <string.h>
static enum CXChildVisitResult visit(CXCursor cursor, CXCursor parent, CXClientData data) {
  (void)parent;
  if (clang_getCursorKind(cursor) == CXCursor_FunctionDecl) {
    CXType function = clang_getCursorType(cursor);
    CXType pointer = clang_getArgType(function, 0);
    CXType pointee = clang_getPointeeType(pointer);
    CXType canonical = clang_getCanonicalType(pointee);
    *(int *)data = pointer.kind == CXType_Pointer && pointee.kind == CXType_FunctionProto &&
                   canonical.kind == CXType_FunctionProto && clang_getFunctionTypeCallingConv(canonical) == CXCallingConv_C;
  }
  return CXChildVisit_Recurse;
}
int main(void) {
  const char *source = "namespace original { long invoke(long (*callback)(long), long value); }";
  struct CXUnsavedFile file = {"independent-function-types.cpp", source, (unsigned long)strlen(source)};
  const char *args[] = {"-x", "c++", "-std=c++17"};
  CXIndex index = clang_createIndex(0, 0);
  CXTranslationUnit unit = NULL;
  if (clang_parseTranslationUnit2(index, file.Filename, args, 3, &file, 1, 0, &unit) != CXError_Success) return 1;
  int success = 0;
  clang_visitChildren(clang_getTranslationUnitCursor(unit), visit, &success);
  clang_disposeTranslationUnit(unit);
  clang_disposeIndex(index);
  return success ? 0 : 2;
}
