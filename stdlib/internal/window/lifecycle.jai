Closed_Proc :: #type (window: WT.Window_Type);

listen :: (proc: Closed_Proc) {
    for existing: listeners if existing == proc return;
    if !listeners.allocator.proc listeners.allocator = context.default_allocator;
    Basic.array_add(*listeners, proc);
    #if OS == .MACOS install_observer();
}

notify_closed :: (window: WT.Window_Type) {
    for proc: listeners proc(window);
}

#scope_module
WT :: #import "Window_Type";
Basic :: #import "Basic";

listeners: [..] Closed_Proc;

#if OS == .MACOS {
    O :: #import "Objective_C";
    Cocoa :: #import "internal/window/cocoa";

    observer: *void;

    install_observer :: () {
        if observer return;

        cls := O.objc_allocateClassPair(O.objc_getClass("NSObject"), "CleanroomWindowLifecycleObserver", 0);

        Basic.assert(cls != null, "window lifecycle observer class allocation failed");
        Basic.assert(O.class_addMethod(cls, O.selector("windowClosed:"), cast(O.IMP) window_closed, "v@:@") != 0, "window lifecycle observer registration failed");
        O.objc_registerClassPair(cls);
        observer = Cocoa.result_message(cls, "new", *void);
        Basic.assert(observer != null, "window lifecycle observer allocation failed");
        center := O.NSNotificationCenter.defaultCenter();

        O.NSNotificationCenter.addObserver(center, cast(O.id) observer, O.selector("windowClosed:"), cast(*O.NSString) Cocoa.native_string("NSWindowWillCloseNotification"), null);
    }

    window_closed :: (#discard receiver: *void, #discard selector: *void, notification: *void) #c_call {
        push_context, defer_pop;
        notify_closed(Cocoa.result_message(notification, "object", WT.Window_Type));
    }
}
