// Original official C API oracle for the source tag/typedef relationship.
#include <clang-c/Index.h>
#include <stdio.h>
#include <string.h>
static CXCursor record_tag, enum_tag;
static int failed;
static enum CXChildVisitResult inspect(CXCursor cursor, CXCursor parent, CXClientData data) {
  (void)parent; (void)data;
  if (cursor.kind == CXCursor_TypedefDecl) {
    CXType type = clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(cursor));
    CXCursor tag = clang_getTypeDeclaration(type);
    CXString name = clang_getCursorSpelling(cursor);
    CXString tag_name = clang_getCursorSpelling(tag);
    const char *alias = clang_getCString(name);
    CXCursor canonical = clang_getCanonicalCursor(tag);
    if (!strcmp(alias, "Anonymous_Record")) record_tag = canonical;
    if (!strcmp(alias, "Anonymous_Mode")) enum_tag = canonical;
    if (!strcmp(alias, "Record_Alias") && !clang_equalCursors(record_tag, canonical)) failed = 1;
    if (!strcmp(alias, "Mode_Alias") && !clang_equalCursors(enum_tag, canonical)) failed = 1;
    CXTranslationUnit unit = clang_Cursor_getTranslationUnit(cursor);
    CXToken *token = clang_getToken(unit, clang_getCursorLocation(tag));
    if (!token) { failed = 1; return CXChildVisit_Break; }
    CXString source = clang_getTokenSpelling(unit, *token);
    printf("%s|%s|%u|%u|%u|%s\n", alias, clang_getCString(tag_name),
           tag.kind, clang_Cursor_isAnonymous(tag), clang_Cursor_isAnonymousRecordDecl(tag), clang_getCString(source));
    clang_disposeString(source);
    clang_disposeTokens(unit, token, 1);
    clang_disposeString(tag_name);
    clang_disposeString(name);
  }
  return CXChildVisit_Continue;
}
int main(int argc, char **argv) {
  if (argc != 2) return 1;
  CXIndex index = clang_createIndex(0, 0);
  CXTranslationUnit unit = clang_parseTranslationUnit(index, argv[1], NULL, 0, NULL, 0, 0);
  if (!unit || clang_getNumDiagnostics(unit)) return 2;
  clang_visitChildren(clang_getTranslationUnitCursor(unit), inspect, NULL);
  clang_disposeTranslationUnit(unit);
  clang_disposeIndex(index);
  return failed;
}
