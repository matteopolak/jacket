#include "developer-tools-enum-fixed.h"
static_assert(sizeof(JaiSmallMode) == 1, "Jai enum width");
int main() { return JSM_Red + JSM_Blue != 1; }
