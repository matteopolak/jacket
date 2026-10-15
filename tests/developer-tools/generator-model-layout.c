#include <stddef.h>
#include <stdio.h>
#include "generator-model-construction.h"

int main(void) {
    printf("Native_Node_Size :: %zu;\nNative_Node_Alignment :: %zu;\nNative_Next_Bit_Offset :: %zu;\n",
           sizeof(struct OriginalNode), _Alignof(struct OriginalNode),
           offsetof(struct OriginalNode, next) * 8);
    printf("Native_Wide_Size :: %zu;\nNative_Wide_Alignment :: %zu;\n",
           sizeof(unsigned __int128), _Alignof(unsigned __int128));
    return 0;
}
