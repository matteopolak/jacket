#include <clang-c/Index.h>
#include <cstdio>
#include <cstring>
struct ModelConversionProbe {
    virtual operator bool() { return true; }
    virtual int following() { return 73; }
};
static CXChildVisitResult visit(CXCursor cursor, CXCursor, CXClientData) {
    if (cursor.kind == CXCursor_ConversionFunction || cursor.kind == CXCursor_CXXMethod) {
        CXString name = clang_getCursorSpelling(cursor);
        std::printf("cursor_kind=%u name=%s virtual=%u\n", unsigned(cursor.kind), clang_getCString(name), clang_CXXMethod_isVirtual(cursor));
        clang_disposeString(name);
    }
    return CXChildVisit_Recurse;
}
int main() {
    const char text[] = "struct ModelConversionProbe { virtual operator bool() { return true; } virtual int following() { return 73; } };";
    CXUnsavedFile file = {"model-conversion-probe.cpp", text, sizeof(text)-1};
    const char *args[] = {"-x", "c++", "-std=c++17"};
    CXIndex index = clang_createIndex(0, 0);
    CXTranslationUnit unit = nullptr;
    if (clang_parseTranslationUnit2(index, file.Filename, args, 3, &file, 1, 0, &unit) != CXError_Success) return 1;
    if (clang_getNumDiagnostics(unit) != 0) return 2;
    clang_visitChildren(clang_getTranslationUnitCursor(unit), visit, nullptr);
    ModelConversionProbe object;
    void **table = nullptr;
    std::memcpy(&table, static_cast<void *>(&object), sizeof(table));
    using Conversion = bool (*)(ModelConversionProbe *);
    using Following = int (*)(ModelConversionProbe *);
    Conversion conversion;
    Following following;
    std::memcpy(&conversion, table, sizeof(conversion));
    std::memcpy(&following, table+1, sizeof(following));
    std::printf("slot0_conversion=%d slot1_following=%d\n", int(conversion(&object)), following(&object));
    clang_disposeTranslationUnit(unit);
    clang_disposeIndex(index);
}
