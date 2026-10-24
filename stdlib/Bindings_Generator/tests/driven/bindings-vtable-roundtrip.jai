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
    assert(object.vtable.apply(object, 32) == 42);
    object.value = 17;
    assert(object.vtable.apply(object, 25) == 42);
    assert(size_of(IndependentDispatch_VTable) == 2 * size_of(*void));
    assert(offset_of(IndependentDispatch, "vtable") == 0);

    #if expected_wrappers {
        assert(IndependentDispatch_apply(object, 25) == 42);
    } else {
        #assert(!#exists(IndependentDispatch_apply));
    }

    #if expected_accessor {
        assert(vtable(object) == object.vtable);
        assert(vtable(object).apply(object, 25) == 42);
    } else {
        #assert(!#exists(vtable));
    }

    print("ok\n");
}
