# Objective_C/AppKit

## Availability

This is the AppKit submodule imported as `Objective_C/AppKit`; it is available on macOS and binds the system AppKit framework. Its class declarations are scoped to this submodule and do not add names to the root `Objective_C` namespace.

## Public declarations

- `NSApp: *NSApplication #elsewhere`
- `NSApplicationMain :: (argc: s32, argv: **u8) -> s32 #foreign AppKit`
- `init_app_kit :: ()`
- `NSApplication :: struct`
- `NSMenu :: struct`
- `NSMenuItem :: struct`
- `NSView :: struct`
- `NSOpenGLContext :: struct`
- `NSOpenGLPixelFormat :: struct`
- `NSOpenGLPixelFormatAttribute :: u32`
- `NSOpenGLContextParameter :: enum NSInteger`
- `NSApplicationActivationPolicy :: enum NSInteger`
- `NSApplicationPresentationOptions :: enum NSUInteger`
- `NSRequestUserAttentionType :: enum NSUInteger`
- `NSWindowStyleMask :: enum_flags NSUInteger`
- `NSWindowCollectionBehavior :: enum_flags NSUInteger`
- `NSBackingStoreType :: enum NSUInteger`
- `NSEventMask :: enum u64`
- `NSEventType :: enum NSUInteger`
- `NSEventModifierFlags :: enum_flags NSUInteger`
- `NSDefaultRunLoopMode: NSRunLoopMode #elsewhere Cocoa`
- `NSScreen :: struct`
- `NSEvent :: struct`
- `NSOpenGLPFAAllRenderers: NSOpenGLPixelFormatAttribute: 1`
- `NSOpenGLPFATripleBuffer: NSOpenGLPixelFormatAttribute: 3`
- `NSOpenGLPFADoubleBuffer: NSOpenGLPixelFormatAttribute: 5`
- `NSOpenGLPFAAuxBuffers: NSOpenGLPixelFormatAttribute: 7`
- `NSOpenGLPFAColorSize: NSOpenGLPixelFormatAttribute: 8`
- `NSOpenGLPFAAlphaSize: NSOpenGLPixelFormatAttribute: 11`
- `NSOpenGLPFADepthSize: NSOpenGLPixelFormatAttribute: 12`
- `NSOpenGLPFAStencilSize: NSOpenGLPixelFormatAttribute: 13`
- `NSOpenGLPFAAccumSize: NSOpenGLPixelFormatAttribute: 14`
- `NSOpenGLPFAMinimumPolicy: NSOpenGLPixelFormatAttribute: 51`
- `NSOpenGLPFAMaximumPolicy: NSOpenGLPixelFormatAttribute: 52`
- `NSOpenGLPFASampleBuffers: NSOpenGLPixelFormatAttribute: 55`
- `NSOpenGLPFASamples: NSOpenGLPixelFormatAttribute: 56`
- `NSOpenGLPFAAuxDepthStencil: NSOpenGLPixelFormatAttribute: 57`
- `NSOpenGLPFAColorFloat: NSOpenGLPixelFormatAttribute: 58`
- `NSOpenGLPFAMultisample: NSOpenGLPixelFormatAttribute: 59`
- `NSOpenGLPFASupersample: NSOpenGLPixelFormatAttribute: 60`
- `NSOpenGLPFASampleAlpha: NSOpenGLPixelFormatAttribute: 61`
- `NSOpenGLPFARendererID: NSOpenGLPixelFormatAttribute: 70`
- `NSOpenGLPFANoRecovery: NSOpenGLPixelFormatAttribute: 72`
- `NSOpenGLPFAAccelerated: NSOpenGLPixelFormatAttribute: 73`
- `NSOpenGLPFAClosestPolicy: NSOpenGLPixelFormatAttribute: 74`
- `NSOpenGLPFABackingStore: NSOpenGLPixelFormatAttribute: 76`
- `NSOpenGLPFAScreenMask: NSOpenGLPixelFormatAttribute: 84`
- `NSOpenGLPFAAllowOfflineRenderers: NSOpenGLPixelFormatAttribute: 96`
- `NSOpenGLPFAAcceleratedCompute: NSOpenGLPixelFormatAttribute: 97`
- `NSOpenGLPFAOpenGLProfile: NSOpenGLPixelFormatAttribute: 99`
- `NSOpenGLPFAVirtualScreenCount: NSOpenGLPixelFormatAttribute: 128`
- `NSOpenGLPFAStereo: NSOpenGLPixelFormatAttribute: 6`
- `NSOpenGLPFAOffScreen: NSOpenGLPixelFormatAttribute: 53`
- `NSOpenGLPFAFullScreen: NSOpenGLPixelFormatAttribute: 54`
- `NSOpenGLPFASingleRenderer: NSOpenGLPixelFormatAttribute: 71`
- `NSOpenGLPFARobust: NSOpenGLPixelFormatAttribute: 75`
- `NSOpenGLPFAMPSafe: NSOpenGLPixelFormatAttribute: 78`
- `NSOpenGLPFAWindow: NSOpenGLPixelFormatAttribute: 80`
- `NSOpenGLPFAMultiScreen: NSOpenGLPixelFormatAttribute: 81`
- `NSOpenGLPFACompliant: NSOpenGLPixelFormatAttribute: 83`
- `NSOpenGLPFAPixelBuffer: NSOpenGLPixelFormatAttribute: 90`
- `NSOpenGLPFARemotePixelBuffer: NSOpenGLPixelFormatAttribute: 91`
- `NSOpenGLProfileVersionLegacy: NSOpenGLPixelFormatAttribute: 0x1000`
- `NSOpenGLProfileVersion3_2Core: NSOpenGLPixelFormatAttribute: 0x3200`
- `NSOpenGLProfileVersion4_1Core: NSOpenGLPixelFormatAttribute: 0x4100`
- `init_core_graphics_runtime_constants :: ()`
- `CGWindowLevelKey :: enum s32`
- `CGWindowLevelForKey :: (key: CGWindowLevelKey) -> CGWindowLevel #foreign CoreGraphics`
- `CGWindowLevel :: s32`

These are typed constants with initialized values. `NSOpenGLPixelFormatAttribute` is the `u32` value type used for the null-terminated attribute/value list passed to pixel-format initialization.

## Public type members

```jai
using NSWindowCollectionBehavior :: enum_flags NSUInteger {
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
```

The `using` declaration exposes these thirteen prefixed enum members directly
in the AppKit import scope. The module declares no short aliases such as
`FullScreenPrimary`; the importable names retain the full
`NSWindowCollectionBehavior` prefix. The flags have `NSUInteger` underlying
type. The native SDK documents mutually exclusive choices within the
managed/transient/stationary, participates/ignores-cycle, primary/auxiliary/
none, and allows/disallows-tiling groups. The SDK's newer
`Primary`, `Auxiliary`, and `CanJoinAllApplications` members are not declared
by this Jai enum and are not part of this public surface.

`NSApplication` embeds `NSObject` as `using #as super: NSObject` and declares these Objective-C message members:

```jai
sharedApplication :: () -> *NSApplication #no_context
run :: (self: *NSApplication) #no_context
stop :: (self: *NSApplication, sender: id) #no_context
mainWindow :: (self: *NSApplication) -> *NSWindow #no_context
keyWindow :: (self: *NSApplication) -> *NSWindow #no_context
sendEvent :: (self: *NSApplication, event: *NSEvent) #no_context
updateWindows :: (self: *NSApplication) #no_context
activateIgnoringOtherApps :: (self: *NSApplication, value: BOOL) #no_context
setActivationPolicy :: (self: *NSApplication, policy: NSApplicationActivationPolicy) -> BOOL #no_context
setDelegate :: (self: *NSApplication, del: *NSApplicationDelegate) -> id #no_context
finishLaunching :: (self: *NSApplication) -> id #no_context
setMainMenu :: (self: *NSApplication, menu: *NSMenu) -> id #no_context
nextEventMatchingMask :: (self: *NSApplication, mask: NSEventMask, untilDate: *NSDate, inMode: NSRunLoopMode, dequeue: BOOL) -> *NSEvent #no_context
presentationOptions :: (self: *NSApplication) -> NSApplicationPresentationOptions #no_context
setPresentationOptions :: (self: *NSApplication, options: NSApplicationPresentationOptions) #no_context
setAppleMenu :: (self: *NSApplication, menu: *NSMenu) #no_context
terminate :: (self: *NSApplication, sender: id) #no_context
requestUserAttention :: (self: *NSApplication, requestType: NSRequestUserAttentionType) -> NSInteger #no_context
```

The AppKit window receiver chain uses these exact public field directives:

```jai
NSResponder :: struct {
    using #as super: NSObject;
}

NSWindow :: struct {
    using #as responder: NSResponder;
}
```

`NSWindow` has no additional stored instance fields in this binding. The
`using #as` chain makes `*NSWindow` convertible through `*NSResponder` to the
`*NSObject` receiver type, including for methods whose public signature takes
`id`; no new cast or overload is required.

The `NSWindow` aggregate also publicly aliases `NSView.backingScaleFactor`.
Its binding signature is `(self: id) -> CGFloat`, so it is an instance
receiver, not a class method. The installed Apple SDK `NSWindow.h` declares a
readonly `CGFloat backingScaleFactor` property getter (available since macOS
10.7), matching that receiver and return shape.

`NSView` embeds `NSObject` as `using #as super: NSObject`, stores `pad: [152 - size_of(NSObject)] u8`, and exposes the relevant selector as:

```jai
addSubView :: (self: *NSView, view: *NSView)
setWantsLayer :: (self: id, yesno: BOOL)
frame :: (self: id) -> NSRect
bounds :: (self: id) -> NSRect
setFrame :: (self: id, frame: NSRect)
setWantsBestResolutionOpenGLSurface :: (self: id, val: BOOL)
convertRectToBacking :: (self: id, rect: NSRect) -> NSRect
convertRectFromBacking :: (self: id, rect: NSRect) -> NSRect
convertPointToBacking :: (self: id, point: NSPoint) -> NSPoint
convertPointFromBacking :: (self: id, point: NSPoint) -> NSPoint
backingScaleFactor :: (self: id) -> CGFloat
isInFullScreenMode :: (self: id) -> BOOL
enterFullScreenMode :: (self: id, screen: *NSScreen, withOptions: *NSDictionary) -> BOOL
exitFullScreenModeWithOptions :: (self: id, options: *NSDictionary)
layer :: (self: id) -> *CALayer
mouse_inRect :: (self: id, mouse: NSPoint, inRect: NSRect) -> BOOL
convertPoint :: (self: id, point: NSPoint, fromView: *NSView) -> NSPoint
drawRect :: (self: id, dirtyRect: NSRect)
registerForDraggedTypes :: (self: id, newTypes: *NSArray(NSPasteboardType))
```

`NSOpenGLContext` embeds `NSObject` as `using #as super: NSObject` and declares:

```jai
currentContext :: () -> *NSOpenGLContext
flushBuffer :: (self: id)
update :: (self: id) -> id
setView :: (self: id, view: *NSView)
view :: (self: id) -> *NSView
initWithFormat :: (self: id, format: *NSOpenGLPixelFormat, shareContext: *NSOpenGLContext) -> *NSOpenGLContext
makeCurrentContext :: (self: id)
setValues :: (self: id, values: *s32, forParameter: NSOpenGLContextParameter)
```

`NSMenu` and `NSMenuItem` are public Objective-C class records with these
contextual shapes:

```jai
NSMenu :: struct {
    using #as super: NSObject;
    addItem :: (self: *NSMenu, item: *NSMenuItem)
    addItemWithTitle :: (self: *NSMenu, title: *NSString, action: Selector, keyEquivalent: *NSString) -> *NSMenuItem
}

NSMenuItem :: struct {
    using #as super: NSObject;
    initWithTitle :: (self: *NSMenuItem, title: *NSString, action: Selector, keyEquivalent: *NSString) -> *NSMenuItem
    setSubmenu :: (self: *NSMenuItem, sub: *NSMenu)
    setKeyEquivalentModifierMask :: (self: *NSMenuItem, value: NSEventModifierFlags)
    keyEquivalentModifierMask :: (self: *NSMenuItem) -> NSEventModifierFlags
}
```

The `NSDictionary` declaration is generic as `NSDictionary :: struct (key:
Type, value: Type)`, but the fullscreen view methods are declared with the bare
`*NSDictionary` type. Their public signatures do not select a key/value
specialization; do not silently substitute `*NSDictionary(key, value)`.

The loaded Core Graphics declarations are also exported through this namespace.
`CGWindowLevel :: s32`; `CGWindowLevelKey :: enum s32` contains, in order,
`kCGBaseWindowLevelKey :: 0`, `kCGMinimumWindowLevelKey`,
`kCGDesktopWindowLevelKey`, `kCGBackstopMenuLevelKey`,
`kCGNormalWindowLevelKey`, `kCGFloatingWindowLevelKey`,
`kCGTornOffMenuWindowLevelKey`, `kCGDockWindowLevelKey`,
`kCGMainMenuWindowLevelKey`, `kCGStatusWindowLevelKey`,
`kCGModalPanelWindowLevelKey`, `kCGPopUpMenuWindowLevelKey`,
`kCGDraggingWindowLevelKey`, `kCGScreenSaverWindowLevelKey`,
`kCGMaximumWindowLevelKey`, `kCGOverlayWindowLevelKey`,
`kCGHelpWindowLevelKey`, `kCGUtilityWindowLevelKey`,
`kCGDesktopIconWindowLevelKey`, `kCGCursorWindowLevelKey`,
`kCGAssistiveTechHighWindowLevelKey`, and `kCGNumberOfWindowLevelKeys`.
`CGWindowLevelForKey :: (key: CGWindowLevelKey) -> CGWindowLevel #foreign CoreGraphics`.
The mutable `CGWindowLevel` globals are `kCGBaseWindowLevel`,
`kCGMinimumWindowLevel`, `kCGDesktopWindowLevel`,
`kCGDesktopIconWindowLevel`, `kCGBackstopMenuLevel`,
`kCGNormalWindowLevel`, `kCGFloatingWindowLevel`,
`kCGTornOffMenuWindowLevel`, `kCGDockWindowLevel`,
`kCGMainMenuWindowLevel`, `kCGStatusWindowLevel`,
`kCGModalPanelWindowLevel`, `kCGPopUpMenuWindowLevel`,
`kCGDraggingWindowLevel`, `kCGScreenSaverWindowLevel`,
`kCGCursorWindowLevel`, `kCGOverlayWindowLevel`, `kCGHelpWindowLevel`,
`kCGUtilityWindowLevel`, `kCGAssistiveTechHighWindowLevel`, and
`kCGMaximumWindowLevel`. Their values are populated by
`init_core_graphics_runtime_constants()` through `CGWindowLevelForKey`; they
are not compile-time constants. This helper returns early when
`kCGBaseWindowLevel` is nonzero. 

`NSOpenGLPixelFormat` embeds `NSObject` as `using #as super: NSObject` and declares:

```jai
initWithAttributes :: (self: id, attrs: *NSOpenGLPixelFormatAttribute) -> *NSOpenGLPixelFormat
getValues :: (self: id, values: *s32, forAttribute: NSOpenGLPixelFormatAttribute, forVirtualScreen: s32)
```

The public `NSOpenGLContextParameter` values are `NSOpenGLContextParameterSwapInterval=222`, `NSOpenGLContextParameterSurfaceOrder=235`, `NSOpenGLContextParameterSurfaceOpacity=236`, `NSOpenGLContextParameterSurfaceBackingSize=304`, `NSOpenGLContextParameterReclaimResources=308`, `NSOpenGLContextParameterCurrentRendererID=309`, `NSOpenGLContextParameterGPUVertexProcessing=310`, `NSOpenGLContextParameterGPUFragmentProcessing=311`, `NSOpenGLContextParameterHasDrawable=314`, `NSOpenGLContextParameterMPSwapsInFlight=315`, `NSOpenGLContextParameterSwapRectangle=200`, `NSOpenGLContextParameterSwapRectangleEnable=201`, `NSOpenGLContextParameterRasterizationEnable=221`, `NSOpenGLContextParameterStateValidation=301`, and `NSOpenGLContextParameterSurfaceSurfaceVolatile=306`.

The public `NSApplicationActivationPolicy` values are `NSApplicationActivationPolicyRegular=0`, `NSApplicationActivationPolicyAccessory=1`, and `NSApplicationActivationPolicyProhibited=2`.

The API's free constants retain their full prefixed identifiers; they are not
shortened enum-member aliases. In particular, the `NSWindowStyleMask` members
are `NSWindowStyleMaskBorderless=0`, `NSWindowStyleMaskTitled=1<<0`,
`NSWindowStyleMaskClosable=1<<1`, `NSWindowStyleMaskMiniaturizable=1<<2`,
`NSWindowStyleMaskResizable=1<<3`, `NSWindowStyleMaskUtilityWindow=1<<4`,
`NSWindowStyleMaskDocModalWindow=1<<6`,
`NSWindowStyleMaskNonactivatingPanel=1<<7`,
`NSWindowStyleMaskTexturedBackground=1<<8`,
`NSWindowStyleMaskUnifiedTitleAndToolbar=1<<12`,
`NSWindowStyleMaskHUDWindow=1<<13`, `NSWindowStyleMaskFullScreen=1<<14`, and
`NSWindowStyleMaskFullSizeContentView=1<<15`.

`NSBackingStoreType` has `NSBackingStoreRetained=0` and
`NSBackingStoreNonretained=1` (both deprecated), and
`NSBackingStoreBuffered=2`. `NSEventMask` is an enum with underlying type `u64`;
its exported members are `NSEventMaskLeftMouseDown`, `NSEventMaskLeftMouseUp`,
`NSEventMaskRightMouseDown`, `NSEventMaskRightMouseUp`,
`NSEventMaskMouseMoved`, `NSEventMaskLeftMouseDragged`,
`NSEventMaskRightMouseDragged`, `NSEventMaskMouseEntered`,
`NSEventMaskMouseExited`, `NSEventMaskKeyDown`, `NSEventMaskKeyUp`,
`NSEventMaskFlagsChanged`, `NSEventMaskAppKitDefined`,
`NSEventMaskSystemDefined`, `NSEventMaskApplicationDefined`,
`NSEventMaskPeriodic`, `NSEventMaskCursorUpdate`, `NSEventMaskScrollWheel`,
`NSEventMaskTabletPoint`, `NSEventMaskTabletProximity`,
`NSEventMaskOtherMouseDown`, `NSEventMaskOtherMouseUp`,
`NSEventMaskOtherMouseDragged`, `NSEventMaskGesture`, `NSEventMaskMagnify`,
`NSEventMaskSwipe`, `NSEventMaskRotate`, `NSEventMaskBeginGesture`,
`NSEventMaskEndGesture`, and `NSEventMaskAny=NSUIntegerMax`. Each named event
mask is `1 <<` the matching `NSEventType` value; `NSEventMaskAny` is the all
events sentinel. The gesture-related enum members are present in the declaration
but have availability dependent on the OS runtime.

`NSEventType` is an enum over `NSUInteger` with these public values:
`NSEventTypeLeftMouseDown=1`, `NSEventTypeLeftMouseUp=2`,
`NSEventTypeRightMouseDown=3`, `NSEventTypeRightMouseUp=4`,
`NSEventTypeMouseMoved=5`, `NSEventTypeLeftMouseDragged=6`,
`NSEventTypeRightMouseDragged=7`, `NSEventTypeMouseEntered=8`,
`NSEventTypeMouseExited=9`, `NSEventTypeKeyDown=10`, `NSEventTypeKeyUp=11`,
`NSEventTypeFlagsChanged=12`, `NSEventTypeAppKitDefined=13`,
`NSEventTypeSystemDefined=14`, `NSEventTypeApplicationDefined=15`,
`NSEventTypePeriodic=16`, `NSEventTypeCursorUpdate=17`,
`NSEventTypeRotate=18`, `NSEventTypeBeginGesture=19`,
`NSEventTypeEndGesture=20`, `NSEventTypeScrollWheel=22`,
`NSEventTypeTabletPoint=23`, `NSEventTypeTabletProximity=24`,
`NSEventTypeOtherMouseDown=25`, `NSEventTypeOtherMouseUp=26`,
`NSEventTypeOtherMouseDragged=27`, `NSEventTypeGesture=29`,
`NSEventTypeMagnify=30`, and `NSEventTypeSwipe=31`.

`NSEventModifierFlags` has `enum_flags NSUInteger` members:
`NSEventModifierFlagCapsLock=1<<16`, `NSEventModifierFlagShift=1<<17`,
`NSEventModifierFlagControl=1<<18`, `NSEventModifierFlagOption=1<<19`,
`NSEventModifierFlagCommand=1<<20`, `NSEventModifierFlagNumericPad=1<<21`,
`NSEventModifierFlagHelp=1<<22`, `NSEventModifierFlagFunction=1<<23`, and
`NSEventModifierFlagDeviceIndependentFlagsMask=0xffff0000`.

`NSApplicationPresentationOptions` is an enum over `NSUInteger` with members
`NSApplicationPresentationDefault=0`, `NSApplicationPresentationAutoHideDock`,
`NSApplicationPresentationHideDock`, `NSApplicationPresentationAutoHideMenuBar`,
`NSApplicationPresentationHideMenuBar`, `NSApplicationPresentationDisableAppleMenu`,
`NSApplicationPresentationDisableProcessSwitching`,
`NSApplicationPresentationDisableForceQuit`,
`NSApplicationPresentationDisableSessionTermination`,
`NSApplicationPresentationDisableHideApplication`,
`NSApplicationPresentationDisableMenuBarTransparency`,
`NSApplicationPresentationFullScreen`,
`NSApplicationPresentationAutoHideToolbar`, and
`NSApplicationPresentationDisableCursorLocationAssistance`; nonzero members
are successive bit positions 0 through 12 as declared. `NSRequestUserAttentionType`
has `NSCriticalRequest=0` and `NSInformationalRequest=10`. The imported
`NSDefaultRunLoopMode` is a typed `NSRunLoopMode` external from Cocoa; its
runtime value is supplied by Cocoa rather than initialized in this binding.

For exact option values, the nonzero presentation members map in declaration
order to `1<<0` through `1<<12`: AutoHideDock, HideDock, AutoHideMenuBar,
HideMenuBar, DisableAppleMenu, DisableProcessSwitching, DisableForceQuit,
DisableSessionTermination, DisableHideApplication,
DisableMenuBarTransparency, FullScreen, AutoHideToolbar, and
DisableCursorLocationAssistance.

`NSEvent` embeds `NSObject` and exposes `mouseLocation() -> NSPoint`,
`type(self: *NSEvent) -> NSEventType`, `characters(self: *NSEvent) -> *NSString`,
`charactersIgnoringModifiers(self: *NSEvent) -> *NSString`,
`keyCode(self: *NSEvent) -> u16`, `modifierFlags(self: *NSEvent) -> NSEventModifierFlags`,
`isARepeat(self: *NSEvent) -> BOOL`, `keyRepeatDelay(self: *NSEvent) -> NSTimeInterval`,
`keyRepeatInterval(self: *NSEvent) -> NSTimeInterval`,
`pressedMouseButtons(self: *NSEvent) -> NSUInteger`,
`buttonNumber(self: *NSEvent) -> NSInteger`, `clickCount(self: *NSEvent) -> NSInteger`,
`associatedEventsMask(self: *NSEvent) -> NSEventMask`, and `deltaX`, `deltaY`,
`deltaZ` each taking `self: *NSEvent` and returning `CGFloat`. The modifier flag
type is `enum_flags NSUInteger` with caps-lock, shift, control, option, command,
numeric-pad, help, function, and device-independent mask members as named in
the API (`NSEventModifierFlagCapsLock` through
`NSEventModifierFlagDeviceIndependentFlagsMask`).

`NSScreen` embeds `NSObject`; `mainScreen() -> *NSScreen` is a class method,
while `frame(self: *NSScreen) -> NSRect` and
`backingScaleFactor(self: *NSScreen) -> CGFloat` are aliases of the corresponding
`NSView` method declarations.

`NSApplicationDelegate` is `struct { using #as nsobject: NSObject; } @ObjCProtocol`.
`NSEvent` is `struct { using #as super: NSObject; ... }`; it adds no stored
fields beyond that using member in this declaration.

`init_app_kit()` initializes the selector table when its `characters` field is
unset and then initializes the Core Graphics window-level runtime constants.
It does not itself create or run `NSApplication`; `sharedApplication()` does
that lookup and stores the result in `NSApp`. `NSApplication.run()` enters the
application event loop. `nextEventMatchingMask` receives the caller's event
mask, date, run-loop mode, and dequeue flag; this binding does not add a
separate event-loop policy or default mode argument.

On macOS, `Window_Type` and the `GL` OpenGL-context APIs use this AppKit
namespace's nominal `NSWindow` and `NSOpenGLContext` types. A GL context
returned as `*NSOpenGLContext` is the same AppKit class type accepted by
`NSOpenGLContext.setView`, `makeCurrentContext`, `update`, and related methods;
it is not a separate opaque GL-owned context type. This is a public cross-module
type identity contract.

`init_app_kit()` initializes the selector table only while its `characters`
field is unset, then calls the Core Graphics runtime-constant initializer.
That initializer populates the exported Core Graphics window-level values and
uses `kCGBaseWindowLevel` as its already-initialized check. The call is not an
application-launch operation.

Preserve these existing Jai signatures while dispatching to the SDK-declared
receiver kind. `NSView.convertPoint(self: id, point: NSPoint, fromView:
*NSView) -> NSPoint` sends `convertPoint:fromView:` to the supplied `self`
instance; the SDK declares that selector as an `NSView` instance method.
`NSEvent.keyRepeatDelay(self: *NSEvent) -> NSTimeInterval`,
`keyRepeatInterval(self: *NSEvent) -> NSTimeInterval`, and
`pressedMouseButtons(self: *NSEvent) -> NSUInteger` retain their legacy
Jai-visible `self` parameter, but that parameter is ignored for Objective-C
dispatch: each selector is sent to the `NSEvent` class object because the SDK
declares these as class properties. Do not dispatch these three selectors to
the passed event instance. This preserves the binding's API while matching the
receiver declarations of the platform SDK.

### Windows, panels, cursors and pasteboard types

`NSWindow` also has `close :: (self: *NSWindow)`.

`NSPanel` embeds `NSWindow` as `using #as window`. `NSSavePanel` embeds
`NSPanel`, and `NSOpenPanel` embeds `NSSavePanel`:

```jai
NSModalResponse :: NSInteger
NSModalResponseOK :: 1
NSModalResponseCancel :: 0

NSSavePanel.savePanel :: () -> *NSSavePanel
NSSavePanel.runModal :: (self: *NSSavePanel) -> NSModalResponse
NSSavePanel.URL :: (self: *NSSavePanel) -> *NSURL
NSSavePanel.directoryURL :: (self: *NSSavePanel) -> *NSURL
NSSavePanel.setDirectoryURL :: (self: *NSSavePanel, url: *NSURL)
NSSavePanel.setNameFieldStringValue :: (self: *NSSavePanel, name: string)
NSSavePanel.setAllowsOtherFileTypes :: (self: *NSSavePanel, value: BOOL)
NSOpenPanel.openPanel :: () -> *NSOpenPanel
NSOpenPanel.setCanChooseFiles :: (self: *NSOpenPanel, value: BOOL)
NSOpenPanel.setCanChooseDirectories :: (self: *NSOpenPanel, value: BOOL)
NSOpenPanel.setAllowsMultipleSelection :: (self: *NSOpenPanel, value: BOOL)
```

A panel title is set with the inherited `NSWindow.setTitle`.

`NSCursor` embeds `NSObject`. Its class getters return `*NSCursor`:
`arrowCursor`, `IBeamCursor`, `pointingHandCursor`, `openHandCursor`,
`closedHandCursor`, `crosshairCursor`, `operationNotAllowedCursor`,
`resizeLeftRightCursor` and `resizeUpDownCursor`. `set` and `push` take the
cursor; `pop`, `hide` and `unhide` are class methods. Without a window server
the getters can return null.

`NSPasteboardTypeFileURL: *NSString #elsewhere AppKit` is the pasteboard type
for file URLs.

## Contracts

`sharedApplication()` obtains the process-wide application object and stores that pointer in the exported `NSApp` variable. The class uses Objective-C message dispatch; these members are methods, not stored Jai data fields. `NSOpenGLPixelFormat.initWithAttributes` accepts the attribute/value sequence terminated by a zero attribute, as required by the system API. The returned context and pixel-format objects follow Objective-C ownership rules; this binding does not add automatic retain/release management.

The declared Jai results of `NSApplication.setDelegate`,
`NSApplication.finishLaunching`, `NSApplication.setMainMenu`, and
`NSOpenGLContext.update` are `id`. The corresponding AppKit selectors are
void-returning native operations, so the Jai `id` result has no meaningful
object-pointer guarantee and callers must ignore it. Dispatch these selectors
using a void-returning function type; if preserving the Jai result slot, return
`null` as an explicit compatibility placeholder rather than reading an
unspecified return register.

The pixel-format constants describe system pixel-format attributes, and the three profile constants are the profile values supplied with `NSOpenGLPFAOpenGLProfile`. Supported profiles and pixel formats depend on macOS and hardware. The binding does not promise that every requested attribute combination is available; callers must inspect the selected format and handle a null initialization result.
