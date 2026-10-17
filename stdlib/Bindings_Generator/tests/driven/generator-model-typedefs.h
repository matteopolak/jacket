struct OriginalNamed { int field; };
typedef struct OriginalNamed __kernel_OriginalAlias;
typedef __kernel_OriginalAlias OriginalKeptAlias;
typedef struct { int value; } OriginalAnonymous;
typedef enum { ORIGINAL_ANONYMOUS_VALUE = 3 } OriginalAnonymousEnum;
typedef struct OriginalNamed CustomOriginalAlias;
typedef int int32_t;
typedef int * __kernel_OriginalPointer;
__kernel_OriginalAlias *original_alias_use;
