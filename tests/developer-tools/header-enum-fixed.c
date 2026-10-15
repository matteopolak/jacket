#include "developer-tools-enum-fixed.h"
_Static_assert(sizeof(JaiSmallMode) == 1, "Jai enum width");
int main(void) { return JSM_Red + JSM_Blue != 1; }
