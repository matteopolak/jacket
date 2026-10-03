// Generated from reviewed appkit.md by generate_appkit.py.

NSOpenGLPFAAllRenderers: NSOpenGLPixelFormatAttribute : 1;
NSOpenGLPFATripleBuffer: NSOpenGLPixelFormatAttribute : 3;
NSOpenGLPFADoubleBuffer: NSOpenGLPixelFormatAttribute : 5;
NSOpenGLPFAAuxBuffers: NSOpenGLPixelFormatAttribute : 7;
NSOpenGLPFAColorSize: NSOpenGLPixelFormatAttribute : 8;
NSOpenGLPFAAlphaSize: NSOpenGLPixelFormatAttribute : 11;
NSOpenGLPFADepthSize: NSOpenGLPixelFormatAttribute : 12;
NSOpenGLPFAStencilSize: NSOpenGLPixelFormatAttribute : 13;
NSOpenGLPFAAccumSize: NSOpenGLPixelFormatAttribute : 14;
NSOpenGLPFAMinimumPolicy: NSOpenGLPixelFormatAttribute : 51;
NSOpenGLPFAMaximumPolicy: NSOpenGLPixelFormatAttribute : 52;
NSOpenGLPFASampleBuffers: NSOpenGLPixelFormatAttribute : 55;
NSOpenGLPFASamples: NSOpenGLPixelFormatAttribute : 56;
NSOpenGLPFAAuxDepthStencil: NSOpenGLPixelFormatAttribute : 57;
NSOpenGLPFAColorFloat: NSOpenGLPixelFormatAttribute : 58;
NSOpenGLPFAMultisample: NSOpenGLPixelFormatAttribute : 59;
NSOpenGLPFASupersample: NSOpenGLPixelFormatAttribute : 60;
NSOpenGLPFASampleAlpha: NSOpenGLPixelFormatAttribute : 61;
NSOpenGLPFARendererID: NSOpenGLPixelFormatAttribute : 70;
NSOpenGLPFANoRecovery: NSOpenGLPixelFormatAttribute : 72;
NSOpenGLPFAAccelerated: NSOpenGLPixelFormatAttribute : 73;
NSOpenGLPFAClosestPolicy: NSOpenGLPixelFormatAttribute : 74;
NSOpenGLPFABackingStore: NSOpenGLPixelFormatAttribute : 76;
NSOpenGLPFAScreenMask: NSOpenGLPixelFormatAttribute : 84;
NSOpenGLPFAAllowOfflineRenderers: NSOpenGLPixelFormatAttribute : 96;
NSOpenGLPFAAcceleratedCompute: NSOpenGLPixelFormatAttribute : 97;
NSOpenGLPFAOpenGLProfile: NSOpenGLPixelFormatAttribute : 99;
NSOpenGLPFAVirtualScreenCount: NSOpenGLPixelFormatAttribute : 128;
NSOpenGLPFAStereo: NSOpenGLPixelFormatAttribute : 6;
NSOpenGLPFAOffScreen: NSOpenGLPixelFormatAttribute : 53;
NSOpenGLPFAFullScreen: NSOpenGLPixelFormatAttribute : 54;
NSOpenGLPFASingleRenderer: NSOpenGLPixelFormatAttribute : 71;
NSOpenGLPFARobust: NSOpenGLPixelFormatAttribute : 75;
NSOpenGLPFAMPSafe: NSOpenGLPixelFormatAttribute : 78;
NSOpenGLPFAWindow: NSOpenGLPixelFormatAttribute : 80;
NSOpenGLPFAMultiScreen: NSOpenGLPixelFormatAttribute : 81;
NSOpenGLPFACompliant: NSOpenGLPixelFormatAttribute : 83;
NSOpenGLPFAPixelBuffer: NSOpenGLPixelFormatAttribute : 90;
NSOpenGLPFARemotePixelBuffer: NSOpenGLPixelFormatAttribute : 91;
NSOpenGLProfileVersionLegacy: NSOpenGLPixelFormatAttribute : 0x1000;
NSOpenGLProfileVersion3_2Core: NSOpenGLPixelFormatAttribute : 0x3200;
NSOpenGLProfileVersion4_1Core: NSOpenGLPixelFormatAttribute : 0x4100;

NSOpenGLPixelFormatAttribute :: u32;

using NSOpenGLContextParameter :: enum O.NSInteger {
    NSOpenGLContextParameterSwapInterval :: 222;
    NSOpenGLContextParameterSurfaceOrder :: 235;
    NSOpenGLContextParameterSurfaceOpacity :: 236;
    NSOpenGLContextParameterSurfaceBackingSize :: 304;
    NSOpenGLContextParameterReclaimResources :: 308;
    NSOpenGLContextParameterCurrentRendererID :: 309;
    NSOpenGLContextParameterGPUVertexProcessing :: 310;
    NSOpenGLContextParameterGPUFragmentProcessing :: 311;
    NSOpenGLContextParameterHasDrawable :: 314;
    NSOpenGLContextParameterMPSwapsInFlight :: 315;
    NSOpenGLContextParameterSwapRectangle :: 200;
    NSOpenGLContextParameterSwapRectangleEnable :: 201;
    NSOpenGLContextParameterRasterizationEnable :: 221;
    NSOpenGLContextParameterStateValidation :: 301;
    NSOpenGLContextParameterSurfaceSurfaceVolatile :: 306;
}

using NSApplicationActivationPolicy :: enum O.NSInteger {
    NSApplicationActivationPolicyRegular :: 0;
    NSApplicationActivationPolicyAccessory :: 1;
    NSApplicationActivationPolicyProhibited :: 2;
}

using NSWindowStyleMask :: enum_flags O.NSUInteger {
    NSWindowStyleMaskBorderless :: 0;
    NSWindowStyleMaskTitled :: 1 << 0;
    NSWindowStyleMaskClosable :: 1 << 1;
    NSWindowStyleMaskMiniaturizable :: 1 << 2;
    NSWindowStyleMaskResizable :: 1 << 3;
    NSWindowStyleMaskUtilityWindow :: 1 << 4;
    NSWindowStyleMaskDocModalWindow :: 1 << 6;
    NSWindowStyleMaskNonactivatingPanel :: 1 << 7;
    NSWindowStyleMaskTexturedBackground :: 1 << 8;
    NSWindowStyleMaskUnifiedTitleAndToolbar :: 1 << 12;
    NSWindowStyleMaskHUDWindow :: 1 << 13;
    NSWindowStyleMaskFullScreen :: 1 << 14;
    NSWindowStyleMaskFullSizeContentView :: 1 << 15;
}

using NSEventType :: enum O.NSUInteger {
    NSEventTypeLeftMouseDown :: 1;
    NSEventTypeLeftMouseUp :: 2;
    NSEventTypeRightMouseDown :: 3;
    NSEventTypeRightMouseUp :: 4;
    NSEventTypeMouseMoved :: 5;
    NSEventTypeLeftMouseDragged :: 6;
    NSEventTypeRightMouseDragged :: 7;
    NSEventTypeMouseEntered :: 8;
    NSEventTypeMouseExited :: 9;
    NSEventTypeKeyDown :: 10;
    NSEventTypeKeyUp :: 11;
    NSEventTypeFlagsChanged :: 12;
    NSEventTypeAppKitDefined :: 13;
    NSEventTypeSystemDefined :: 14;
    NSEventTypeApplicationDefined :: 15;
    NSEventTypePeriodic :: 16;
    NSEventTypeCursorUpdate :: 17;
    NSEventTypeRotate :: 18;
    NSEventTypeBeginGesture :: 19;
    NSEventTypeEndGesture :: 20;
    NSEventTypeScrollWheel :: 22;
    NSEventTypeTabletPoint :: 23;
    NSEventTypeTabletProximity :: 24;
    NSEventTypeOtherMouseDown :: 25;
    NSEventTypeOtherMouseUp :: 26;
    NSEventTypeOtherMouseDragged :: 27;
    NSEventTypeGesture :: 29;
    NSEventTypeMagnify :: 30;
    NSEventTypeSwipe :: 31;
}

using NSBackingStoreType :: enum O.NSUInteger {
    NSBackingStoreRetained :: 0;
    NSBackingStoreNonretained :: 1;
    NSBackingStoreBuffered :: 2;
}

using NSEventMask :: enum u64 {
    NSEventMaskLeftMouseDown :: 1 << 1;
    NSEventMaskLeftMouseUp :: 1 << 2;
    NSEventMaskRightMouseDown :: 1 << 3;
    NSEventMaskRightMouseUp :: 1 << 4;
    NSEventMaskMouseMoved :: 1 << 5;
    NSEventMaskLeftMouseDragged :: 1 << 6;
    NSEventMaskRightMouseDragged :: 1 << 7;
    NSEventMaskMouseEntered :: 1 << 8;
    NSEventMaskMouseExited :: 1 << 9;
    NSEventMaskKeyDown :: 1 << 10;
    NSEventMaskKeyUp :: 1 << 11;
    NSEventMaskFlagsChanged :: 1 << 12;
    NSEventMaskAppKitDefined :: 1 << 13;
    NSEventMaskSystemDefined :: 1 << 14;
    NSEventMaskApplicationDefined :: 1 << 15;
    NSEventMaskPeriodic :: 1 << 16;
    NSEventMaskCursorUpdate :: 1 << 17;
    NSEventMaskRotate :: 1 << 18;
    NSEventMaskBeginGesture :: 1 << 19;
    NSEventMaskEndGesture :: 1 << 20;
    NSEventMaskScrollWheel :: 1 << 22;
    NSEventMaskTabletPoint :: 1 << 23;
    NSEventMaskTabletProximity :: 1 << 24;
    NSEventMaskOtherMouseDown :: 1 << 25;
    NSEventMaskOtherMouseUp :: 1 << 26;
    NSEventMaskOtherMouseDragged :: 1 << 27;
    NSEventMaskGesture :: 1 << 29;
    NSEventMaskMagnify :: 1 << 30;
    NSEventMaskSwipe :: 1 << 31;
    NSEventMaskAny :: 0xffffffffffffffff;
}

using NSEventModifierFlags :: enum_flags O.NSUInteger {
    NSEventModifierFlagCapsLock :: 1 << 16;
    NSEventModifierFlagShift :: 1 << 17;
    NSEventModifierFlagControl :: 1 << 18;
    NSEventModifierFlagOption :: 1 << 19;
    NSEventModifierFlagCommand :: 1 << 20;
    NSEventModifierFlagNumericPad :: 1 << 21;
    NSEventModifierFlagHelp :: 1 << 22;
    NSEventModifierFlagFunction :: 1 << 23;
    NSEventModifierFlagDeviceIndependentFlagsMask :: 0xffff0000;
}

using NSApplicationPresentationOptions :: enum O.NSUInteger {
    NSApplicationPresentationDefault :: 0;
    NSApplicationPresentationAutoHideDock :: 1 << 0;
    NSApplicationPresentationHideDock :: 1 << 1;
    NSApplicationPresentationAutoHideMenuBar :: 1 << 2;
    NSApplicationPresentationHideMenuBar :: 1 << 3;
    NSApplicationPresentationDisableAppleMenu :: 1 << 4;
    NSApplicationPresentationDisableProcessSwitching :: 1 << 5;
    NSApplicationPresentationDisableForceQuit :: 1 << 6;
    NSApplicationPresentationDisableSessionTermination :: 1 << 7;
    NSApplicationPresentationDisableHideApplication :: 1 << 8;
    NSApplicationPresentationDisableMenuBarTransparency :: 1 << 9;
    NSApplicationPresentationFullScreen :: 1 << 10;
    NSApplicationPresentationAutoHideToolbar :: 1 << 11;
    NSApplicationPresentationDisableCursorLocationAssistance :: 1 << 12;
}

using NSRequestUserAttentionType :: enum O.NSUInteger {
    NSCriticalRequest :: 0;
    NSInformationalRequest :: 10;
}

NSApplication :: struct {
    using #as super: O.NSObject;

    sharedApplication :: () -> *NSApplication #no_context {
        Send :: #type (receiver: *void, selector: O.Selector) -> *NSApplication #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSApplication) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        NSApp = send(binding_class("NSApplication"), binding_selector("sharedApplication"));

        return NSApp;
    }

    run :: (self: *NSApplication) #no_context {
        Send :: #type (receiver: *void, selector: O.Selector) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("run"));
    }

    stop :: (self: *NSApplication, sender: O.id) #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, sender: O.id) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("stop:"), sender);
    }

    mainWindow :: (self: *NSApplication) -> *NSWindow #no_context {
        Send :: #type (receiver: *void, selector: O.Selector) -> *NSWindow #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSWindow) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("mainWindow"));
    }

    keyWindow :: (self: *NSApplication) -> *NSWindow #no_context {
        Send :: #type (receiver: *void, selector: O.Selector) -> *NSWindow #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSWindow) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("keyWindow"));
    }

    sendEvent :: (self: *NSApplication, event: *NSEvent) #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, event: *NSEvent) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("sendEvent:"), event);
    }

    updateWindows :: (self: *NSApplication) #no_context {
        Send :: #type (receiver: *void, selector: O.Selector) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("updateWindows"));
    }

    activateIgnoringOtherApps :: (self: *NSApplication, value: O.BOOL) #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, value: O.BOOL) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("activateIgnoringOtherApps:"), value);
    }

    setActivationPolicy :: (self: *NSApplication, policy: NSApplicationActivationPolicy) -> O.BOOL #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, policy: NSApplicationActivationPolicy) -> O.BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.BOOL) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("setActivationPolicy:"), policy);
    }

    setDelegate :: (self: *NSApplication, del: *NSApplicationDelegate) -> O.id #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, del: *NSApplicationDelegate) #c_call;
        send := cast(Send) binding_send;

        send(self, binding_selector("setDelegate:"), del);

        // Reviewed unspecified Jai id slot; native operation returns void.
        return null;
    }

    finishLaunching :: (self: *NSApplication) -> O.id #no_context {
        Send :: #type (receiver: *void, selector: O.Selector) #c_call;
        send := cast(Send) binding_send;

        send(self, binding_selector("finishLaunching"));

        // Reviewed unspecified Jai id slot; native operation returns void.
        return null;
    }

    setMainMenu :: (self: *NSApplication, menu: *NSMenu) -> O.id #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, menu: *NSMenu) #c_call;
        send := cast(Send) binding_send;

        send(self, binding_selector("setMainMenu:"), menu);

        // Reviewed unspecified Jai id slot; native operation returns void.
        return null;
    }

    nextEventMatchingMask :: (self: *NSApplication, mask: NSEventMask, untilDate: *O.NSDate, inMode: O.NSRunLoopMode, dequeue: O.BOOL) -> *NSEvent #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, mask: NSEventMask, untilDate: *O.NSDate, inMode: O.NSRunLoopMode, dequeue: O.BOOL) -> *NSEvent #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSEvent) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("nextEventMatchingMask:untilDate:inMode:dequeue:"), mask, untilDate, inMode, dequeue);
    }

    presentationOptions :: (self: *NSApplication) -> NSApplicationPresentationOptions #no_context {
        Send :: #type (receiver: *void, selector: O.Selector) -> NSApplicationPresentationOptions #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSApplicationPresentationOptions) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("presentationOptions"));
    }

    setPresentationOptions :: (self: *NSApplication, options: NSApplicationPresentationOptions) #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, options: NSApplicationPresentationOptions) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setPresentationOptions:"), options);
    }

    setAppleMenu :: (self: *NSApplication, menu: *NSMenu) #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, menu: *NSMenu) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setAppleMenu:"), menu);
    }

    terminate :: (self: *NSApplication, sender: O.id) #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, sender: O.id) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("terminate:"), sender);
    }

    requestUserAttention :: (self: *NSApplication, requestType: NSRequestUserAttentionType) -> O.NSInteger #no_context {
        Send :: #type (receiver: *void, selector: O.Selector, requestType: NSRequestUserAttentionType) -> O.NSInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSInteger) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("requestUserAttention:"), requestType);
    }

}

NSView :: struct {
    using #as super: O.NSObject;
    pad: [152 - size_of(O.NSObject)] u8;

    addSubView :: (self: *NSView, view: *NSView) {
        Send :: #type (receiver: *void, selector: O.Selector, view: *NSView) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("addSubview:"), view);
    }

    setWantsLayer :: (self: O.id, yesno: O.BOOL) {
        Send :: #type (receiver: *void, selector: O.Selector, yesno: O.BOOL) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setWantsLayer:"), yesno);
    }

    frame :: (self: O.id) -> O.NSRect {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.NSRect #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSRect) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("frame"));
    }

    bounds :: (self: O.id) -> O.NSRect {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.NSRect #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSRect) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("bounds"));
    }

    setFrame :: (self: O.id, frame: O.NSRect) {
        Send :: #type (receiver: *void, selector: O.Selector, frame: O.NSRect) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setFrame:"), frame);
    }

    setWantsBestResolutionOpenGLSurface :: (self: O.id, val: O.BOOL) {
        Send :: #type (receiver: *void, selector: O.Selector, val: O.BOOL) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setWantsBestResolutionOpenGLSurface:"), val);
    }

    convertRectToBacking :: (self: O.id, rect: O.NSRect) -> O.NSRect {
        Send :: #type (receiver: *void, selector: O.Selector, rect: O.NSRect) -> O.NSRect #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSRect) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("convertRectToBacking:"), rect);
    }

    convertRectFromBacking :: (self: O.id, rect: O.NSRect) -> O.NSRect {
        Send :: #type (receiver: *void, selector: O.Selector, rect: O.NSRect) -> O.NSRect #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSRect) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("convertRectFromBacking:"), rect);
    }

    convertPointToBacking :: (self: O.id, point: O.NSPoint) -> O.NSPoint {
        Send :: #type (receiver: *void, selector: O.Selector, point: O.NSPoint) -> O.NSPoint #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSPoint) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("convertPointToBacking:"), point);
    }

    convertPointFromBacking :: (self: O.id, point: O.NSPoint) -> O.NSPoint {
        Send :: #type (receiver: *void, selector: O.Selector, point: O.NSPoint) -> O.NSPoint #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSPoint) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("convertPointFromBacking:"), point);
    }

    backingScaleFactor :: (self: O.id) -> O.CGFloat {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.CGFloat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.CGFloat) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("backingScaleFactor"));
    }

    isInFullScreenMode :: (self: O.id) -> O.BOOL {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.BOOL) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("isInFullScreenMode"));
    }

    enterFullScreenMode :: (self: O.id, screen: *NSScreen, withOptions: *O.NSDictionary) -> O.BOOL {
        Send :: #type (receiver: *void, selector: O.Selector, screen: *NSScreen, withOptions: *O.NSDictionary) -> O.BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.BOOL) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("enterFullScreenMode:withOptions:"), screen, withOptions);
    }

    exitFullScreenModeWithOptions :: (self: O.id, options: *O.NSDictionary) {
        Send :: #type (receiver: *void, selector: O.Selector, options: *O.NSDictionary) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("exitFullScreenModeWithOptions:"), options);
    }

    layer :: (self: O.id) -> *O.CALayer {
        Send :: #type (receiver: *void, selector: O.Selector) -> *O.CALayer #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*O.CALayer) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("layer"));
    }

    mouse_inRect :: (self: O.id, mouse: O.NSPoint, inRect: O.NSRect) -> O.BOOL {
        Send :: #type (receiver: *void, selector: O.Selector, mouse: O.NSPoint, inRect: O.NSRect) -> O.BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.BOOL) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("mouse:inRect:"), mouse, inRect);
    }

    convertPoint :: (self: O.id, point: O.NSPoint, fromView: *NSView) -> O.NSPoint {
        Send :: #type (receiver: *void, selector: O.Selector, point: O.NSPoint, fromView: *NSView) -> O.NSPoint #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSPoint) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("convertPoint:fromView:"), point, fromView);
    }

    drawRect :: (self: O.id, dirtyRect: O.NSRect) {
        Send :: #type (receiver: *void, selector: O.Selector, dirtyRect: O.NSRect) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("drawRect:"), dirtyRect);
    }

    registerForDraggedTypes :: (self: O.id, newTypes: *O.NSArray(O.NSPasteboardType)) {
        Send :: #type (receiver: *void, selector: O.Selector, newTypes: *O.NSArray(O.NSPasteboardType)) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("registerForDraggedTypes:"), newTypes);
    }

}

NSOpenGLContext :: struct {
    using #as super: O.NSObject;

    currentContext :: () -> *NSOpenGLContext {
        Send :: #type (receiver: *void, selector: O.Selector) -> *NSOpenGLContext #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSOpenGLContext) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(binding_class("NSOpenGLContext"), binding_selector("currentContext"));
    }

    flushBuffer :: (self: O.id) {
        Send :: #type (receiver: *void, selector: O.Selector) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("flushBuffer"));
    }

    update :: (self: O.id) -> O.id {
        Send :: #type (receiver: *void, selector: O.Selector) #c_call;
        send := cast(Send) binding_send;

        send(self, binding_selector("update"));

        // Reviewed unspecified Jai id slot; native operation returns void.
        return null;
    }

    setView :: (self: O.id, view: *NSView) {
        Send :: #type (receiver: *void, selector: O.Selector, view: *NSView) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setView:"), view);
    }

    view :: (self: O.id) -> *NSView {
        Send :: #type (receiver: *void, selector: O.Selector) -> *NSView #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSView) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("view"));
    }

    initWithFormat :: (self: O.id, format: *NSOpenGLPixelFormat, shareContext: *NSOpenGLContext) -> *NSOpenGLContext {
        Send :: #type (receiver: *void, selector: O.Selector, format: *NSOpenGLPixelFormat, shareContext: *NSOpenGLContext) -> *NSOpenGLContext #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSOpenGLContext) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("initWithFormat:shareContext:"), format, shareContext);
    }

    makeCurrentContext :: (self: O.id) {
        Send :: #type (receiver: *void, selector: O.Selector) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("makeCurrentContext"));
    }

    setValues :: (self: O.id, values: *s32, forParameter: NSOpenGLContextParameter) {
        Send :: #type (receiver: *void, selector: O.Selector, values: *s32, forParameter: NSOpenGLContextParameter) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setValues:forParameter:"), values, forParameter);
    }

}

NSOpenGLPixelFormat :: struct {
    using #as super: O.NSObject;

    initWithAttributes :: (self: O.id, attrs: *NSOpenGLPixelFormatAttribute) -> *NSOpenGLPixelFormat {
        Send :: #type (receiver: *void, selector: O.Selector, attrs: *NSOpenGLPixelFormatAttribute) -> *NSOpenGLPixelFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSOpenGLPixelFormat) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("initWithAttributes:"), attrs);
    }

    getValues :: (self: O.id, values: *s32, forAttribute: NSOpenGLPixelFormatAttribute, forVirtualScreen: s32) {
        Send :: #type (receiver: *void, selector: O.Selector, values: *s32, forAttribute: NSOpenGLPixelFormatAttribute, forVirtualScreen: s32) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("getValues:forAttribute:forVirtualScreen:"), values, forAttribute, forVirtualScreen);
    }

}

NSMenu :: struct {
    using #as super: O.NSObject;
    addItem :: (self: *NSMenu, item: *NSMenuItem) {
        Send :: #type (receiver: *void, selector: O.Selector, item: *NSMenuItem) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("addItem:"), item);
    }

    addItemWithTitle :: (self: *NSMenu, title: *O.NSString, action: O.Selector, keyEquivalent: *O.NSString) -> *NSMenuItem {
        Send :: #type (receiver: *void, selector: O.Selector, title: *O.NSString, action: O.Selector, keyEquivalent: *O.NSString) -> *NSMenuItem #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSMenuItem) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("addItemWithTitle:action:keyEquivalent:"), title, action, keyEquivalent);
    }

}

NSMenuItem :: struct {
    using #as super: O.NSObject;
    initWithTitle :: (self: *NSMenuItem, title: *O.NSString, action: O.Selector, keyEquivalent: *O.NSString) -> *NSMenuItem {
        Send :: #type (receiver: *void, selector: O.Selector, title: *O.NSString, action: O.Selector, keyEquivalent: *O.NSString) -> *NSMenuItem #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSMenuItem) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("initWithTitle:action:keyEquivalent:"), title, action, keyEquivalent);
    }

    setSubmenu :: (self: *NSMenuItem, sub: *NSMenu) {
        Send :: #type (receiver: *void, selector: O.Selector, sub: *NSMenu) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setSubmenu:"), sub);
    }

    setKeyEquivalentModifierMask :: (self: *NSMenuItem, value: NSEventModifierFlags) {
        Send :: #type (receiver: *void, selector: O.Selector, value: NSEventModifierFlags) #c_call;
        send := cast(Send) binding_send;
        send(self, binding_selector("setKeyEquivalentModifierMask:"), value);
    }

    keyEquivalentModifierMask :: (self: *NSMenuItem) -> NSEventModifierFlags {
        Send :: #type (receiver: *void, selector: O.Selector) -> NSEventModifierFlags #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSEventModifierFlags) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("keyEquivalentModifierMask"));
    }

}

NSScreen :: struct {
    using #as super: O.NSObject;
    mainScreen :: () -> *NSScreen {
        Send :: #type (receiver: *void, selector: O.Selector) -> *NSScreen #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSScreen) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(binding_class("NSScreen"), binding_selector("mainScreen"));
    }

    frame :: NSView.frame;
    backingScaleFactor :: NSView.backingScaleFactor;
}

NSApplicationDelegate :: struct {
    using #as nsobject: O.NSObject;
} @ObjCProtocol

NSEvent :: struct {
    using #as super: O.NSObject;
    mouseLocation :: () -> O.NSPoint {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.NSPoint #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSPoint) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(binding_class("NSEvent"), binding_selector("mouseLocation"));
    }

    type :: (self: *NSEvent) -> NSEventType {
        Send :: #type (receiver: *void, selector: O.Selector) -> NSEventType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSEventType) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("type"));
    }

    characters :: (self: *NSEvent) -> *O.NSString {
        Send :: #type (receiver: *void, selector: O.Selector) -> *O.NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*O.NSString) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("characters"));
    }

    charactersIgnoringModifiers :: (self: *NSEvent) -> *O.NSString {
        Send :: #type (receiver: *void, selector: O.Selector) -> *O.NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*O.NSString) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("charactersIgnoringModifiers"));
    }

    keyCode :: (self: *NSEvent) -> u16 {
        Send :: #type (receiver: *void, selector: O.Selector) -> u16 #c_call;
        send: Send;

        #if CPU == .X64 && size_of(u16) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("keyCode"));
    }

    modifierFlags :: (self: *NSEvent) -> NSEventModifierFlags {
        Send :: #type (receiver: *void, selector: O.Selector) -> NSEventModifierFlags #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSEventModifierFlags) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("modifierFlags"));
    }

    isARepeat :: (self: *NSEvent) -> O.BOOL {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.BOOL) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("isARepeat"));
    }

    keyRepeatDelay :: (self: *NSEvent) -> O.NSTimeInterval {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.NSTimeInterval #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSTimeInterval) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(binding_class("NSEvent"), binding_selector("keyRepeatDelay"));
    }

    keyRepeatInterval :: (self: *NSEvent) -> O.NSTimeInterval {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.NSTimeInterval #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSTimeInterval) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(binding_class("NSEvent"), binding_selector("keyRepeatInterval"));
    }

    pressedMouseButtons :: (self: *NSEvent) -> O.NSUInteger {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSUInteger) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(binding_class("NSEvent"), binding_selector("pressedMouseButtons"));
    }

    buttonNumber :: (self: *NSEvent) -> O.NSInteger {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.NSInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSInteger) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("buttonNumber"));
    }

    clickCount :: (self: *NSEvent) -> O.NSInteger {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.NSInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.NSInteger) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("clickCount"));
    }

    associatedEventsMask :: (self: *NSEvent) -> NSEventMask {
        Send :: #type (receiver: *void, selector: O.Selector) -> NSEventMask #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSEventMask) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("associatedEventsMask"));
    }

    deltaX :: (self: *NSEvent) -> O.CGFloat {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.CGFloat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.CGFloat) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("deltaX"));
    }

    deltaY :: (self: *NSEvent) -> O.CGFloat {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.CGFloat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.CGFloat) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("deltaY"));
    }

    deltaZ :: (self: *NSEvent) -> O.CGFloat {
        Send :: #type (receiver: *void, selector: O.Selector) -> O.CGFloat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(O.CGFloat) > 16 {
            send = cast(Send) binding_send_stret;
        } else {
            send = cast(Send) binding_send;
        }

        return send(self, binding_selector("deltaZ"));
    }

}

CGWindowLevel :: s32;

using CGWindowLevelKey :: enum s32 {
    kCGBaseWindowLevelKey :: 0;
    kCGMinimumWindowLevelKey :: 1;
    kCGDesktopWindowLevelKey :: 2;
    kCGBackstopMenuLevelKey :: 3;
    kCGNormalWindowLevelKey :: 4;
    kCGFloatingWindowLevelKey :: 5;
    kCGTornOffMenuWindowLevelKey :: 6;
    kCGDockWindowLevelKey :: 7;
    kCGMainMenuWindowLevelKey :: 8;
    kCGStatusWindowLevelKey :: 9;
    kCGModalPanelWindowLevelKey :: 10;
    kCGPopUpMenuWindowLevelKey :: 11;
    kCGDraggingWindowLevelKey :: 12;
    kCGScreenSaverWindowLevelKey :: 13;
    kCGMaximumWindowLevelKey :: 14;
    kCGOverlayWindowLevelKey :: 15;
    kCGHelpWindowLevelKey :: 16;
    kCGUtilityWindowLevelKey :: 17;
    kCGDesktopIconWindowLevelKey :: 18;
    kCGCursorWindowLevelKey :: 19;
    kCGAssistiveTechHighWindowLevelKey :: 20;
    kCGNumberOfWindowLevelKeys :: 21;
}

kCGBaseWindowLevel: CGWindowLevel;
kCGMinimumWindowLevel: CGWindowLevel;
kCGDesktopWindowLevel: CGWindowLevel;
kCGDesktopIconWindowLevel: CGWindowLevel;
kCGBackstopMenuLevel: CGWindowLevel;
kCGNormalWindowLevel: CGWindowLevel;
kCGFloatingWindowLevel: CGWindowLevel;
kCGTornOffMenuWindowLevel: CGWindowLevel;
kCGDockWindowLevel: CGWindowLevel;
kCGMainMenuWindowLevel: CGWindowLevel;
kCGStatusWindowLevel: CGWindowLevel;
kCGModalPanelWindowLevel: CGWindowLevel;
kCGPopUpMenuWindowLevel: CGWindowLevel;
kCGDraggingWindowLevel: CGWindowLevel;
kCGScreenSaverWindowLevel: CGWindowLevel;
kCGCursorWindowLevel: CGWindowLevel;
kCGOverlayWindowLevel: CGWindowLevel;
kCGHelpWindowLevel: CGWindowLevel;
kCGUtilityWindowLevel: CGWindowLevel;
kCGAssistiveTechHighWindowLevel: CGWindowLevel;
kCGMaximumWindowLevel: CGWindowLevel;

CGWindowLevelForKey :: (key: CGWindowLevelKey) -> CGWindowLevel #foreign CoreGraphics;

init_core_graphics_runtime_constants :: () {
    if kCGBaseWindowLevel return;

    kCGBaseWindowLevel = CGWindowLevelForKey(.kCGBaseWindowLevelKey);
    kCGMinimumWindowLevel = CGWindowLevelForKey(.kCGMinimumWindowLevelKey);
    kCGDesktopWindowLevel = CGWindowLevelForKey(.kCGDesktopWindowLevelKey);
    kCGDesktopIconWindowLevel = CGWindowLevelForKey(.kCGDesktopIconWindowLevelKey);
    kCGBackstopMenuLevel = CGWindowLevelForKey(.kCGBackstopMenuLevelKey);
    kCGNormalWindowLevel = CGWindowLevelForKey(.kCGNormalWindowLevelKey);
    kCGFloatingWindowLevel = CGWindowLevelForKey(.kCGFloatingWindowLevelKey);
    kCGTornOffMenuWindowLevel = CGWindowLevelForKey(.kCGTornOffMenuWindowLevelKey);
    kCGDockWindowLevel = CGWindowLevelForKey(.kCGDockWindowLevelKey);
    kCGMainMenuWindowLevel = CGWindowLevelForKey(.kCGMainMenuWindowLevelKey);
    kCGStatusWindowLevel = CGWindowLevelForKey(.kCGStatusWindowLevelKey);
    kCGModalPanelWindowLevel = CGWindowLevelForKey(.kCGModalPanelWindowLevelKey);
    kCGPopUpMenuWindowLevel = CGWindowLevelForKey(.kCGPopUpMenuWindowLevelKey);
    kCGDraggingWindowLevel = CGWindowLevelForKey(.kCGDraggingWindowLevelKey);
    kCGScreenSaverWindowLevel = CGWindowLevelForKey(.kCGScreenSaverWindowLevelKey);
    kCGCursorWindowLevel = CGWindowLevelForKey(.kCGCursorWindowLevelKey);
    kCGOverlayWindowLevel = CGWindowLevelForKey(.kCGOverlayWindowLevelKey);
    kCGHelpWindowLevel = CGWindowLevelForKey(.kCGHelpWindowLevelKey);
    kCGUtilityWindowLevel = CGWindowLevelForKey(.kCGUtilityWindowLevelKey);
    kCGAssistiveTechHighWindowLevel = CGWindowLevelForKey(.kCGAssistiveTechHighWindowLevelKey);
    kCGMaximumWindowLevel = CGWindowLevelForKey(.kCGMaximumWindowLevelKey);
}

init_app_kit :: () {
    if !selectors.characters initialize_selectors();
    init_core_graphics_runtime_constants();
}

#scope_file
Selectors :: struct {
    sharedApplication: O.Selector;
    run: O.Selector;
    stop_: O.Selector;
    mainWindow: O.Selector;
    keyWindow: O.Selector;
    sendEvent_: O.Selector;
    updateWindows: O.Selector;
    activateIgnoringOtherApps_: O.Selector;
    setActivationPolicy_: O.Selector;
    setDelegate_: O.Selector;
    finishLaunching: O.Selector;
    setMainMenu_: O.Selector;
    nextEventMatchingMask_untilDate_inMode_dequeue_: O.Selector;
    presentationOptions: O.Selector;
    setPresentationOptions_: O.Selector;
    setAppleMenu_: O.Selector;
    terminate_: O.Selector;
    requestUserAttention_: O.Selector;
    addSubview_: O.Selector;
    setWantsLayer_: O.Selector;
    frame: O.Selector;
    bounds: O.Selector;
    setFrame_: O.Selector;
    setWantsBestResolutionOpenGLSurface_: O.Selector;
    convertRectToBacking_: O.Selector;
    convertRectFromBacking_: O.Selector;
    convertPointToBacking_: O.Selector;
    convertPointFromBacking_: O.Selector;
    backingScaleFactor: O.Selector;
    isInFullScreenMode: O.Selector;
    enterFullScreenMode_withOptions_: O.Selector;
    exitFullScreenModeWithOptions_: O.Selector;
    layer: O.Selector;
    mouse_inRect_: O.Selector;
    convertPoint_fromView_: O.Selector;
    drawRect_: O.Selector;
    registerForDraggedTypes_: O.Selector;
    currentContext: O.Selector;
    flushBuffer: O.Selector;
    update: O.Selector;
    setView_: O.Selector;
    view: O.Selector;
    initWithFormat_shareContext_: O.Selector;
    makeCurrentContext: O.Selector;
    setValues_forParameter_: O.Selector;
    initWithAttributes_: O.Selector;
    getValues_forAttribute_forVirtualScreen_: O.Selector;
    addItem_: O.Selector;
    addItemWithTitle_action_keyEquivalent_: O.Selector;
    initWithTitle_action_keyEquivalent_: O.Selector;
    setSubmenu_: O.Selector;
    setKeyEquivalentModifierMask_: O.Selector;
    keyEquivalentModifierMask: O.Selector;
    mainScreen: O.Selector;
    mouseLocation: O.Selector;
    type: O.Selector;
    characters: O.Selector;
    charactersIgnoringModifiers: O.Selector;
    keyCode: O.Selector;
    modifierFlags: O.Selector;
    isARepeat: O.Selector;
    keyRepeatDelay: O.Selector;
    keyRepeatInterval: O.Selector;
    pressedMouseButtons: O.Selector;
    buttonNumber: O.Selector;
    clickCount: O.Selector;
    associatedEventsMask: O.Selector;
    deltaX: O.Selector;
    deltaY: O.Selector;
    deltaZ: O.Selector;
    initWithContentRect_styleMask_backing_defer_: O.Selector;
    initWithContentRect_styleMask_backing_defer_screen_: O.Selector;
    setTitle_: O.Selector;
    makeFirstResponder_: O.Selector;
    center: O.Selector;
}

selectors: Selectors;

initialize_selectors :: () {
    selectors.sharedApplication = binding_selector("sharedApplication");
    selectors.run = binding_selector("run");
    selectors.stop_ = binding_selector("stop:");
    selectors.mainWindow = binding_selector("mainWindow");
    selectors.keyWindow = binding_selector("keyWindow");
    selectors.sendEvent_ = binding_selector("sendEvent:");
    selectors.updateWindows = binding_selector("updateWindows");
    selectors.activateIgnoringOtherApps_ = binding_selector("activateIgnoringOtherApps:");
    selectors.setActivationPolicy_ = binding_selector("setActivationPolicy:");
    selectors.setDelegate_ = binding_selector("setDelegate:");
    selectors.finishLaunching = binding_selector("finishLaunching");
    selectors.setMainMenu_ = binding_selector("setMainMenu:");
    selectors.nextEventMatchingMask_untilDate_inMode_dequeue_ = binding_selector("nextEventMatchingMask:untilDate:inMode:dequeue:");
    selectors.presentationOptions = binding_selector("presentationOptions");
    selectors.setPresentationOptions_ = binding_selector("setPresentationOptions:");
    selectors.setAppleMenu_ = binding_selector("setAppleMenu:");
    selectors.terminate_ = binding_selector("terminate:");
    selectors.requestUserAttention_ = binding_selector("requestUserAttention:");
    selectors.addSubview_ = binding_selector("addSubview:");
    selectors.setWantsLayer_ = binding_selector("setWantsLayer:");
    selectors.frame = binding_selector("frame");
    selectors.bounds = binding_selector("bounds");
    selectors.setFrame_ = binding_selector("setFrame:");
    selectors.setWantsBestResolutionOpenGLSurface_ = binding_selector("setWantsBestResolutionOpenGLSurface:");
    selectors.convertRectToBacking_ = binding_selector("convertRectToBacking:");
    selectors.convertRectFromBacking_ = binding_selector("convertRectFromBacking:");
    selectors.convertPointToBacking_ = binding_selector("convertPointToBacking:");
    selectors.convertPointFromBacking_ = binding_selector("convertPointFromBacking:");
    selectors.backingScaleFactor = binding_selector("backingScaleFactor");
    selectors.isInFullScreenMode = binding_selector("isInFullScreenMode");
    selectors.enterFullScreenMode_withOptions_ = binding_selector("enterFullScreenMode:withOptions:");
    selectors.exitFullScreenModeWithOptions_ = binding_selector("exitFullScreenModeWithOptions:");
    selectors.layer = binding_selector("layer");
    selectors.mouse_inRect_ = binding_selector("mouse:inRect:");
    selectors.convertPoint_fromView_ = binding_selector("convertPoint:fromView:");
    selectors.drawRect_ = binding_selector("drawRect:");
    selectors.registerForDraggedTypes_ = binding_selector("registerForDraggedTypes:");
    selectors.currentContext = binding_selector("currentContext");
    selectors.flushBuffer = binding_selector("flushBuffer");
    selectors.update = binding_selector("update");
    selectors.setView_ = binding_selector("setView:");
    selectors.view = binding_selector("view");
    selectors.initWithFormat_shareContext_ = binding_selector("initWithFormat:shareContext:");
    selectors.makeCurrentContext = binding_selector("makeCurrentContext");
    selectors.setValues_forParameter_ = binding_selector("setValues:forParameter:");
    selectors.initWithAttributes_ = binding_selector("initWithAttributes:");
    selectors.getValues_forAttribute_forVirtualScreen_ = binding_selector("getValues:forAttribute:forVirtualScreen:");
    selectors.addItem_ = binding_selector("addItem:");
    selectors.addItemWithTitle_action_keyEquivalent_ = binding_selector("addItemWithTitle:action:keyEquivalent:");
    selectors.initWithTitle_action_keyEquivalent_ = binding_selector("initWithTitle:action:keyEquivalent:");
    selectors.setSubmenu_ = binding_selector("setSubmenu:");
    selectors.setKeyEquivalentModifierMask_ = binding_selector("setKeyEquivalentModifierMask:");
    selectors.keyEquivalentModifierMask = binding_selector("keyEquivalentModifierMask");
    selectors.mainScreen = binding_selector("mainScreen");
    selectors.mouseLocation = binding_selector("mouseLocation");
    selectors.type = binding_selector("type");
    selectors.characters = binding_selector("characters");
    selectors.charactersIgnoringModifiers = binding_selector("charactersIgnoringModifiers");
    selectors.keyCode = binding_selector("keyCode");
    selectors.modifierFlags = binding_selector("modifierFlags");
    selectors.isARepeat = binding_selector("isARepeat");
    selectors.keyRepeatDelay = binding_selector("keyRepeatDelay");
    selectors.keyRepeatInterval = binding_selector("keyRepeatInterval");
    selectors.pressedMouseButtons = binding_selector("pressedMouseButtons");
    selectors.buttonNumber = binding_selector("buttonNumber");
    selectors.clickCount = binding_selector("clickCount");
    selectors.associatedEventsMask = binding_selector("associatedEventsMask");
    selectors.deltaX = binding_selector("deltaX");
    selectors.deltaY = binding_selector("deltaY");
    selectors.deltaZ = binding_selector("deltaZ");
    selectors.initWithContentRect_styleMask_backing_defer_ = binding_selector("initWithContentRect:styleMask:backing:defer:");
    selectors.initWithContentRect_styleMask_backing_defer_screen_ = binding_selector("initWithContentRect:styleMask:backing:defer:screen:");
    selectors.setTitle_ = binding_selector("setTitle:");
    selectors.makeFirstResponder_ = binding_selector("makeFirstResponder:");
    selectors.center = binding_selector("center");
}

#scope_file
O :: #import "Objective_C";

libobjc :: #library,system "objc";
CoreGraphics :: #library,system "CoreGraphics";
binding_class :: (name: *u8) -> *void #foreign libobjc "objc_getClass";
binding_selector :: (name: *u8) -> O.Selector #foreign libobjc "sel_registerName";
binding_send :: () #foreign libobjc "objc_msgSend";
#if CPU == .X64 binding_send_stret :: () #foreign libobjc "objc_msgSend_stret";
