// Reviewed AppKit facade; contextual window fields remain in objective-c.md.
// Complete class/member/enum contracts: docs/specs/stdlib/modules/appkit.md.
#assert OS == .MACOS, "Objective_C/AppKit requires macOS";

using NSWindowCollectionBehavior :: enum_flags O.NSUInteger {
    NSWindowCollectionBehaviorDefault :: 0;
    NSWindowCollectionBehaviorCanJoinAllSpaces :: 1 << 0;
    NSWindowCollectionBehaviorMoveToActiveSpace :: 1 << 1;
    NSWindowCollectionBehaviorManaged :: 1 << 2;
    NSWindowCollectionBehaviorTransient :: 1 << 3;
    NSWindowCollectionBehaviorStationary :: 1 << 4;
    NSWindowCollectionBehaviorParticipatesInCycle :: 1 << 5;
    NSWindowCollectionBehaviorIgnoresCycle :: 1 << 6;
    NSWindowCollectionBehaviorFullScreenPrimary :: 1 << 7;
    NSWindowCollectionBehaviorFullScreenAuxiliary :: 1 << 8;
    NSWindowCollectionBehaviorFullScreenNone :: 1 << 9;
    NSWindowCollectionBehaviorFullScreenAllowsTiling :: 1 << 11;
    NSWindowCollectionBehaviorFullScreenDisallowsTiling :: 1 << 12;
}

NSResponder :: struct {
    using #as super: O.NSObject;
}

NSWindow :: struct {
    using #as responder: NSResponder;

    initWithContentRect :: (self: *NSWindow, rect: O.NSRect, styleMask: NSWindowStyleMask,
                           backing: NSBackingStoreType, _defer: O.BOOL) -> *NSWindow {
        Send :: #type (receiver: *NSWindow, selector: O.Selector, rect: O.NSRect,
                       styleMask: NSWindowStyleMask, backing: NSBackingStoreType,
                       defer_creation: O.BOOL) -> *NSWindow #c_call;
        send := cast(Send) binding_send;

        return send(self, O.selector("initWithContentRect:styleMask:backing:defer:"),
                    rect, styleMask, backing, _defer);
    }

    initWithContentRect :: (self: O.id, rect: O.NSRect, styleMask: NSWindowStyleMask,
                           backing: NSBackingStoreType, _defer: O.BOOL,
                           screen: *NSScreen) -> *NSWindow {
        Send :: #type (receiver: O.id, selector: O.Selector, rect: O.NSRect,
                       styleMask: NSWindowStyleMask, backing: NSBackingStoreType,
                       defer_creation: O.BOOL, screen: *NSScreen) -> *NSWindow #c_call;
        send := cast(Send) binding_send;

        return send(self, O.selector("initWithContentRect:styleMask:backing:defer:screen:"),
                    rect, styleMask, backing, _defer, screen);
    }

    setReleasedWhenClosed :: (self: *NSWindow, value: O.BOOL) {
        send_argument(self, "setReleasedWhenClosed:", value);
    }

    setTitle :: (self: *NSWindow, title: string) -> O.id {
        // Review specifies an unused/unspecified id result. Call the native
        // void method with its actual ABI; do not read an undefined register.
        text := O.NSString.getTempString(title);
        Send :: #type (receiver: *NSWindow, selector: O.Selector, title: *O.NSString) #c_call;
        send := cast(Send) binding_send;

        send(self, O.selector("setTitle:"), text);

        return cast(O.id) self;
    }

    setCollectionBehavior :: (self: *NSWindow, behavior: NSWindowCollectionBehavior) {
        send_argument(self, "setCollectionBehavior:", behavior);
    }

    makeKeyAndOrderFront :: (self: *NSWindow, sender: O.id) {
        send_argument(self, "makeKeyAndOrderFront:", sender);
    }

    setDelegate :: (self: *NSWindow, dg: O.id) {
        send_argument(self, "setDelegate:", dg);
    }

    setContentView :: (self: *NSWindow, contentView: O.id) {
        send_argument(self, "setContentView:", contentView);
    }

    makeFirstResponder :: (self: *NSWindow, view: O.id) {
        // AppKit returns BOOL; this reviewed convenience signature discards it.
        Send :: #type (receiver: *NSWindow, selector: O.Selector, view: O.id) -> O.BOOL #c_call;
        send := cast(Send) binding_send;
        send(self, O.selector("makeFirstResponder:"), view);
    }

    contentView :: (self: *NSWindow) -> *NSView {
        return send_result(self, "contentView", *NSView);
    }

    center :: (self: *NSWindow) {
        Send :: #type (receiver: *NSWindow, selector: O.Selector) #c_call;
        send := cast(Send) binding_send;
        send(self, O.selector("center"));
    }

    styleMask :: (self: *NSWindow) -> NSWindowStyleMask {
        return send_result(self, "styleMask", NSWindowStyleMask);
    }

    toggleFullScreen :: (self: *NSWindow, sender: O.id) {
        send_argument(self, "toggleFullScreen:", sender);
    }

    frame :: NSView.frame;
    backingScaleFactor :: NSView.backingScaleFactor;
}

#load "generated.jai";

NSApp: *NSApplication #elsewhere AppKit;
NSDefaultRunLoopMode: O.NSRunLoopMode #elsewhere Cocoa;
NSApplicationMain :: (argc: s32, argv: **u8) -> s32 #foreign AppKit;

#scope_file
AppKit :: #library,system "AppKit";
Cocoa :: #library,system "Cocoa";

O :: #import "Objective_C";

#library,system,link_always "AppKit";
libobjc :: #library,system "objc";
binding_send :: () #foreign libobjc "objc_msgSend";
#if CPU == .X64 binding_send_stret :: () #foreign libobjc "objc_msgSend_stret";

send_argument :: (receiver: *void, selector: string, value: $T) {
    Send :: #type (receiver: *void, selector: O.Selector, value: T) #c_call;
    send := cast(Send) binding_send;
    send(receiver, O.selector(selector), value);
}

send_result :: (receiver: *void, selector: string, $Result: Type) -> Result {
    Send :: #type (receiver: *void, selector: O.Selector) -> Result #c_call;
    send: Send;

    #if CPU == .X64 && size_of(Result) > 16 {
        send = cast(Send) binding_send_stret;
    } else {
        send = cast(Send) binding_send;
    }

    return send(receiver, O.selector(selector));
}
