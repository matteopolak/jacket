#include <clang-c/Index.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>

static enum CXChildVisitResult inspect(CXCursor cursor, CXCursor parent, CXClientData data) {
    (void)parent;
    (void)data;
    enum CXCursorKind kind = clang_getCursorKind(cursor);
    CXString spelling = clang_getCursorSpelling(cursor);
    const char *name = clang_getCString(spelling);
    if (kind == CXCursor_EnumDecl) {
        CXType underlying = clang_getEnumDeclIntegerType(cursor);
        CXString type_name = clang_getTypeSpelling(underlying);
        printf("enum %s type=%s kind=%d size=%lld align=%lld\n", name,
               clang_getCString(type_name), underlying.kind,
               clang_Type_getSizeOf(underlying), clang_Type_getAlignOf(underlying));
        clang_disposeString(type_name);
    }
    if (kind == CXCursor_EnumConstantDecl) {
        printf("value %s signed=%lld unsigned=%llu\n", name,
               clang_getEnumConstantDeclValue(cursor),
               clang_getEnumConstantDeclUnsignedValue(cursor));
    }
    if (kind == CXCursor_StructDecl) {
        CXType type = clang_getCursorType(cursor);
        printf("record %s defined=%u size=%lld align=%lld\n", name,
               clang_isCursorDefinition(cursor), clang_Type_getSizeOf(type),
               clang_Type_getAlignOf(type));
    }
    clang_disposeString(spelling);
    return CXChildVisit_Recurse;
}

int main(int argc, char **argv) {
    assert(argc == 2);
    CXIndex index = clang_createIndex(0, 0);
    CXTranslationUnit unit = NULL;
    const char *arguments[] = {"-x", "c", "-std=c11"};
    assert(clang_parseTranslationUnit2(index, argv[1], arguments, 3, NULL, 0,
                                     CXTranslationUnit_None, &unit) == CXError_Success);
    assert(unit && clang_getNumDiagnostics(unit) == 0);
    clang_visitChildren(clang_getTranslationUnitCursor(unit), inspect, NULL);
    clang_disposeTranslationUnit(unit);
    clang_disposeIndex(index);
    return 0;
}
