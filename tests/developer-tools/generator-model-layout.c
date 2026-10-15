#include <stddef.h>
#include <stdio.h>
#include "generator-model-construction.h"

int main(void) {
    printf("Native_Node_Size :: %zu;\nNative_Node_Alignment :: %zu;\nNative_Next_Bit_Offset :: %zu;\n",
           sizeof(struct OriginalNode), _Alignof(struct OriginalNode),
           offsetof(struct OriginalNode, next) * 8);
    return 0;
}
