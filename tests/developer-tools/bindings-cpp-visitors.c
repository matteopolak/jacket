#include <clang-c/Index.h>
#include <stdio.h>
#include <string.h>

struct selected_type { CXType type; int found; };
struct visits { unsigned count; int stop; };

static enum CXChildVisitResult select_class(CXCursor cursor, CXCursor parent, CXClientData data) {
    (void)parent;
    if (clang_getCursorKind(cursor) == CXCursor_ClassDecl) {
        CXString name = clang_getCursorSpelling(cursor);
        if (strcmp(clang_getCString(name), "OriginalDerived") == 0) {
            struct selected_type *selected = data;
            selected->type = clang_getCursorType(cursor);
            selected->found = 1;
        }
        clang_disposeString(name);
    }
    return CXChildVisit_Continue;
}

static enum CXVisitorResult count_field(CXCursor cursor, CXClientData data) {
    (void)cursor;
    struct visits *visits = data;
    ++visits->count;
    return visits->stop ? CXVisit_Break : CXVisit_Continue;
}

int main(int argc, char **argv) {
    if (argc != 2) return 2;
    CXIndex index = clang_createIndex(0, 0);
    if (!index) return 3;
    const char *args[] = {"-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror"};
    CXTranslationUnit unit = NULL;
    if (clang_parseTranslationUnit2(index, argv[1], args, 6, NULL, 0, 0, &unit) != CXError_Success ||
        !unit || clang_getNumDiagnostics(unit) != 0) {
        if (unit) clang_disposeTranslationUnit(unit);
        clang_disposeIndex(index);
        return 3;
    }
    struct selected_type selected = {0};
    clang_visitChildren(clang_getTranslationUnitCursor(unit), select_class, &selected);
    if (!selected.found) { clang_disposeTranslationUnit(unit); clang_disposeIndex(index); return 4; }
    struct visits fields = {0}, bases = {0}, methods = {0}, stopped = {0, 1};
    unsigned field_result = clang_Type_visitFields(selected.type, count_field, &fields);
    unsigned base_result = clang_visitCXXBaseClasses(selected.type, count_field, &bases);
    unsigned method_result = clang_visitCXXMethods(selected.type, count_field, &methods);
    unsigned stop_result = clang_visitCXXMethods(selected.type, count_field, &stopped);
    printf("%u %u %u %u %u %u %u %u\n", field_result, fields.count, base_result, bases.count,
           method_result, methods.count, stop_result, stopped.count);
    clang_disposeTranslationUnit(unit);
    clang_disposeIndex(index);
    return 0;
}
