#include "bindings-vtable-workflow.hpp"
#include <new>
extern "C" IndependentDispatch *dispatch_create() { return new IndependentDispatch; }
extern "C" void dispatch_destroy(IndependentDispatch *object) {
    object->IndependentDispatch::~IndependentDispatch();
    ::operator delete(object);
}
