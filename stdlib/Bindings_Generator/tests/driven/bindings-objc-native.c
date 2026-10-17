#include <objc/runtime.h>

SEL SEL_adjust;
SEL SEL_class_adjust;
static Class original_class;
static int initialization_count;

static void original_class_initialize(id receiver, SEL selector) {
    (void)receiver;
    (void)selector;
    ++initialization_count;
}

static int original_adjust(id receiver, SEL selector, int input) {
    (void)selector;
    return *(int *)object_getIndexedIvars(receiver) + input;
}
static int original_class_adjust(id receiver, SEL selector, int input) {
    (void)receiver;
    (void)selector;
    return 10 + input;
}
__attribute__((constructor)) static void original_initialize(void) {
    SEL_adjust = sel_registerName("adjust:");
    SEL_class_adjust = SEL_adjust;
    original_class = objc_allocateClassPair(Nil, "IndependentToolsMessage", 0);
    class_addMethod(object_getClass((id)original_class), sel_registerName("initialize"), (IMP)original_class_initialize, "v@:");
    class_addMethod(original_class, SEL_adjust, (IMP)original_adjust, "i@:i");
    class_addMethod(object_getClass((id)original_class), SEL_adjust, (IMP)original_class_adjust, "i@:i");
    objc_registerClassPair(original_class);
}
void *original_object(void) {
    id object = class_createInstance(original_class, sizeof(int));
    *(int *)object_getIndexedIvars(object) = 10;
    return object;
}
void *original_class_object(void) { return (void *)original_class; }
void original_set_value(void *object, int value) { *(int *)object_getIndexedIvars((id)object) = value; }
void original_dispose(void *object) { object_dispose((id)object); }
int original_initializations(void) { return initialization_count; }
