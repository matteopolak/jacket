#import "Basic";
#load "generated-objc.jai";

original_object :: () -> *void #foreign native;
original_class_object :: () -> *void #foreign native;
original_set_value :: (object: *void, value: s32) #foreign native;
original_dispose :: (object: *void) #foreign native;
original_initializations :: () -> s32 #foreign native;

main :: () {
    object := original_object();

    assert(object != null);
    defer original_dispose(object);
    assert(adjust(object, 32) == 42);
    original_set_value(object, 17);
    assert(adjust(object, 25) == 42);
    assert(class_adjust(original_class_object(), 32) == 42);
    assert(original_initializations() == 1);
    print("ok\n");
}
