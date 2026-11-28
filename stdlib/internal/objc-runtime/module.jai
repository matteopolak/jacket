// The Objective-C runtime calls the window, clipboard, cursor, WebGPU and binding helpers share, declared once.
//
// Declaring them links nothing until a caller uses one, so portable code may import this module. Untyped pointers stand for classes, selectors and objects, so a caller needs no binding module to look
// one up. `objc_msgSend` is not here: each caller casts it to the signature of its own message.

// The class registered under `name`, or null.
objc_getClass :: (name: *u8) -> *void #foreign objc;
objc_lookUpClass :: (name: *u8) -> *void #foreign objc;

// The selector named `name`, registered when it is new.
sel_registerName :: (name: *u8) -> *void #foreign objc;

objc_retain :: (object: *void) -> *void #foreign objc;
objc_release :: (object: *void) #foreign objc;

objc_setAssociatedObject :: (object: *void, key: *void, value: *void, policy: u64) #foreign objc;
objc_getAssociatedObject :: (object: *void, key: *void) -> *void #foreign objc;

objc_allocateClassPair :: (parent: *void, name: *u8, extra: u64) -> *void #foreign objc;
objc_disposeClassPair :: (class: *void) #foreign objc;
object_getClass :: (object: *void) -> *void #foreign objc;
class_getSuperclass :: (class: *void) -> *void #foreign objc;
class_getInstanceSize :: (class: *void) -> u64 #foreign objc;

#scope_file
objc :: #library,system "objc"; @native(external)
