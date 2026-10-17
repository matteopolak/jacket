#import "Basic";
#load "generated-vtable.jai";
#load "vtable-options.jai";

native :: #library "dispatch";
dispatch_create :: () -> *IndependentDispatch #foreign native;
dispatch_destroy :: (object: *IndependentDispatch) #foreign native;

main :: () {
    object := dispatch_create();

    assert(object != null);
    defer dispatch_destroy(object);
    assert(object.__vtable.slot_0(object, 32) == 42);
    object.value = 17;
    assert(object.__vtable.slot_0(object, 25) == 42);
    assert(size_of(IndependentDispatch.IndependentDispatch_VTable) == 2 * size_of(*void));
    assert(offset_of(IndependentDispatch, "__vtable") == 0);

    #if expected_wrappers {
        assert(IndependentDispatch.IndependentDispatch_slot_0(object, 25) == 42);
    } else {
        #assert(!#exists(IndependentDispatch.IndependentDispatch_slot_0));
    }

    #if expected_accessor {
        assert(IndependentDispatch.vtable(object) == object.__vtable);
        assert(IndependentDispatch.vtable(object).slot_0(object, 25) == 42);
    } else {
        #assert(!#exists(IndependentDispatch.vtable));
    }

    print("ok\n");
}
