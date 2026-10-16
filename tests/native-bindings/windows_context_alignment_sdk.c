// Compile against the actual target SDK. This fixture never defines SDK types.
#include <stddef.h>
#include <windows.h>

_Static_assert(sizeof(M128A) == 16, "M128A size");
_Static_assert(_Alignof(M128A) == 16, "M128A alignment");
_Static_assert(offsetof(M128A, Low) == 0, "M128A.Low");
_Static_assert(offsetof(M128A, High) == 8, "M128A.High");
_Static_assert(sizeof(CONTEXT) == 1232, "x64 CONTEXT size");
_Static_assert(_Alignof(CONTEXT) == 16, "x64 CONTEXT alignment");
_Static_assert(offsetof(CONTEXT, ContextFlags) == 48, "CONTEXT.ContextFlags");
_Static_assert(offsetof(CONTEXT, Rip) == 248, "CONTEXT.Rip");
_Static_assert(offsetof(CONTEXT, FltSave) == 256, "CONTEXT.FltSave");
_Static_assert(offsetof(CONTEXT, Xmm0) == 416, "CONTEXT.Xmm0");
_Static_assert(offsetof(CONTEXT, VectorRegister) == 768,
               "CONTEXT.VectorRegister");
_Static_assert(offsetof(CONTEXT, VectorControl) == 1184,
               "CONTEXT.VectorControl");

struct vector_container {
  unsigned char prefix;
  M128A values[2];
};
struct context_container {
  unsigned char prefix;
  CONTEXT values[2];
};
_Static_assert(_Alignof(struct vector_container) == 16, "embedded M128A align");
_Static_assert(offsetof(struct vector_container, values) == 16,
               "embedded M128A offset");
_Static_assert(sizeof(struct vector_container) == 48, "M128A array stride");
_Static_assert(_Alignof(struct context_container) == 16,
               "embedded CONTEXT align");
_Static_assert(offsetof(struct context_container, values) == 16,
               "embedded CONTEXT offset");
_Static_assert(sizeof(struct context_container) == 2480,
               "CONTEXT array stride");
