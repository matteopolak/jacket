struct OriginalNode {
    int value;
    struct OriginalNode *next;
};
enum OriginalColor { ORIGINAL_NEGATIVE = -1, ORIGINAL_SEVEN = 7 };
typedef struct OriginalNode OriginalAlias;
/** Leading model comment. */
int original_counter = 9;
extern int original_apply(OriginalAlias *node, const char *name, int (*callback)(int));
int original_expression = (2 + 3) * -4; ///< trailing model comment.
int original_array[3];
int (*original_callback)(int);
__int128 original_signed_wide;
unsigned __int128 original_unsigned_wide;
unsigned __int128 original_wide_value = ((unsigned __int128)1) << 100;
