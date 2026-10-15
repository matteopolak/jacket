#include <clang-c/Index.h>
#include <stddef.h>
#include <stdio.h>

int main(void) {
    printf("CXString=%zu CXCursor=%zu CXType=%zu CXUnsavedFile=%zu\n",
           sizeof(CXString), sizeof(CXCursor), sizeof(CXType), sizeof(struct CXUnsavedFile));
    printf("CXIndexOptions=%zu PreambleStoragePath=%zu InvocationEmissionPath=%zu\n",
           sizeof(CXIndexOptions), offsetof(CXIndexOptions, PreambleStoragePath),
           offsetof(CXIndexOptions, InvocationEmissionPath));
    printf("CXToken=%zu ptr_data=%zu CXTokenKind=%zu\n", sizeof(CXToken),
           offsetof(CXToken, ptr_data), sizeof(enum CXTokenKind));
    return 0;
}
