struct Original_Type_Pair { int left; unsigned long right; };
typedef struct Original_Type_Pair Original_Type_Alias;
volatile Original_Type_Alias original_type_pair;
double original_debug_types(int value, unsigned char token) {
    original_type_pair.left = value;
    original_type_pair.right = token;
    return (double)original_type_pair.left + (double)original_type_pair.right;
}
