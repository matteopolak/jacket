#include "bindings-vtable-workflow.hpp"
#include <new>
#if defined(_WIN32)
#define DISPATCH_EXPORT __declspec(dllexport)
#else
#define DISPATCH_EXPORT
#endif
extern "C" DISPATCH_EXPORT IndependentDispatch *dispatch_create() { return new IndependentDispatch; }
extern "C" DISPATCH_EXPORT void dispatch_destroy(IndependentDispatch *object) {
    object->IndependentDispatch::~IndependentDispatch();
    ::operator delete(object);
}
