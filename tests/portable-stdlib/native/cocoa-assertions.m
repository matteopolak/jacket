#import <AppKit/AppKit.h>
#import <objc/runtime.h>

int portable_cocoa_prepare(void) {
    if (!NSApplicationLoad()) return -1;
    NSApplication *app = [NSApplication sharedApplication];
    if ([app activationPolicy] != NSApplicationActivationPolicyProhibited &&
        ![app setActivationPolicy:NSApplicationActivationPolicyProhibited]) return -2;
    if ([app activationPolicy] != NSApplicationActivationPolicyProhibited) return -3;
    return 1;
}

int portable_cocoa_policy(void) { return (int)[NSApp activationPolicy]; }

int portable_cocoa_observer_has_method(void) {
    Class cls = objc_getClass("CleanroomWindowInputObserver");
    return cls != Nil && class_getInstanceMethod(cls, sel_registerName("windowNotification:")) != NULL;
}

int portable_cocoa_register_collision(void) {
    Class cls = objc_allocateClassPair([NSObject class], "CleanroomWindowInputObserver", 0);
    if (cls == Nil) return 0;
    objc_registerClassPair(cls);
    return 1;
}
