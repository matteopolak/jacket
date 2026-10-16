#include "generator-run-system.h"
#ifndef RUN_ARGUMENT
#error runner must preserve configured arguments
#endif
#define RUN_VALUE (RUN_ARGUMENT + 2)
#define RUN_UTF8 "héllo"
#define RUN_SAME 7
#define RUN_SAME 7
#define RUN_CHANGED 1
#undef RUN_CHANGED
#define RUN_CHANGED 2
extern "C" {
int run_function(int value);
int run_function(int value);
}
namespace RunScope { struct Record { int field; }; }
// Native overloads are distinct; the reviewed model erases reference shape.
int run_equivalent(int *value);
int run_equivalent(int &value);
int run_different(int value);
int run_different(double value);
