#include "bindings-vtable-abi.hpp"
#if defined(_WIN32)
#define ABI_EXPORT __declspec(dllexport)
#else
#define ABI_EXPORT
#endif
DispatchDerived force_layout;
extern "C" ABI_EXPORT DispatchDerived *abi_create() { return new DispatchDerived; }
extern "C" ABI_EXPORT void abi_destroy(DispatchDerived *object) { delete object; }
