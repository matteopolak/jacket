/* Independent official SDK/linker positive for the same native image. */
#include <assert.h>
extern int original_resolved(void);
int main(void) { assert(original_resolved() == 37); return 0; }
