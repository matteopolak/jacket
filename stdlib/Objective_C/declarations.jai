// Generated from docs/specs/stdlib/modules/objective-c.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
ptrdiff_t :: s64;

Native :: #import "internal/bindings/native";

binding_objc_class :: (name: *u8) -> *void #foreign libobjc "objc_getClass";
binding_objc_selector :: (name: *u8) -> *void #foreign libobjc "sel_registerName";
binding_objc_send :: () #foreign libobjc "objc_msgSend";
binding_abort :: () #foreign libc "abort";
binding_strlen :: (text: *u8) -> u64 #foreign libc "strlen";

#if CPU == .X64 {
    binding_objc_send_stret :: () #foreign libobjc "objc_msgSend_stret";
}

#library,system,link_always "Foundation";
#library,system,link_always "QuartzCore";
Foundation_Framework :: #library,system "Foundation_Framework";
libSystem :: #library,system "System";
libc :: #library,system "c";
libobjc :: #library,system "objc";
#scope_export

// Types and layouts

CALayer :: struct {
    using nsobject: NSObject;
}

CGSize :: struct {
    width: CGFloat;
    height: CGFloat;
}

#if CPU == .ARM64 {
    mach_header :: struct {}

}

using NSActivityOptions :: enum u64 {
    NSActivityIdleDisplaySleepDisabled :: 1099511627776;
    NSActivityIdleSystemSleepDisabled :: 1048576;
    NSActivitySuddenTerminationDisabled :: 16384;
    NSActivityAutomaticTerminationDisabled :: 32768;
    NSActivityUserInitiated :: 16777215;
    NSActivityUserInitiatedAllowingIdleSystemSleep :: 15728639;
    NSActivityBackground :: 255;
    NSActivityLatencyCritical :: 1095216660480;
}

NSArray :: struct(Object_Type: Type) {
    using #as super: NSObject;

    arrayWithObject :: (obj: Object_Type) -> *NSArray(Object_Type) {
        Send :: #type (receiver: *void, op: *void, a0: type_of(obj)) -> *NSArray(Object_Type) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(Object_Type)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSArray"), binding_objc_selector("arrayWithObject:"), obj);
    }
    arrayWithObjects :: (objs: *Object_Type, count: NSUInteger) -> *NSArray(Object_Type) {
        Send :: #type (receiver: *void, op: *void, a0: type_of(objs), a1: type_of(count)) -> *NSArray(Object_Type) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(Object_Type)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSArray"), binding_objc_selector("arrayWithObjects:count:"), objs, count);
    }

    containsObject :: (self: *NSArray(Object_Type), anObject: Object_Type) -> BOOL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(anObject)) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("containsObject:"), anObject);
    }

    objectAtIndex :: (self: *NSArray(Object_Type), index: NSUInteger) -> Object_Type {
        Send :: #type (receiver: *void, op: *void, a0: type_of(index)) -> Object_Type #c_call;
        send: Send;

        #if CPU == .X64 && size_of(Object_Type) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("objectAtIndex:"), index);
    }

    count :: (self: *NSArray(Object_Type)) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("count"));
    }
}

NSAutoreleasePool :: struct {
    using #as super: NSObject;

    drain :: (self: *NSAutoreleasePool) {
        Send :: #type (receiver: *void, op: *void) -> void #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drain"));
    }

    showPools :: () {
        Send :: #type (receiver: *void, op: *void) -> void #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(binding_objc_class("NSAutoreleasePool"), binding_objc_selector("showPools"));
    }
}

NSBundle :: struct {
    using #as super: NSObject;

    resourcePath :: (self: *NSBundle) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("resourcePath"));
    }

    mainBundle :: () -> *NSBundle {
        Send :: #type (receiver: *void, op: *void) -> *NSBundle #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSBundle) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSBundle"), binding_objc_selector("mainBundle"));
    }
}

NSData :: struct {
    using #as super: NSObject;

    bytes :: (self: *NSData) -> *void {
        Send :: #type (receiver: *void, op: *void) -> *void #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*void) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bytes"));
    }

    length :: (self: *NSData) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("length"));
    }
}

NSDate :: struct {
    using #as super: NSObject;

    distantPast :: () -> *NSDate {
        Send :: #type (receiver: *void, op: *void) -> *NSDate #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSDate) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSDate"), binding_objc_selector("distantPast"));
    }

    distantFuture :: () -> *NSDate {
        Send :: #type (receiver: *void, op: *void) -> *NSDate #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSDate) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSDate"), binding_objc_selector("distantFuture"));
    }
}

NSDictionary :: struct(key: Type, value: Type) {
    using #as super: NSObject;
}

NSError :: struct {
    using #as super: NSObject;

    localizedDescription :: (self: *NSError) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("localizedDescription"));
    }
}

NSFileManager :: struct {
    using #as super: NSObject;

    defaultManager :: () -> *NSFileManager {
        Send :: #type (receiver: *void, op: *void) -> *NSFileManager #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSFileManager) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSFileManager"), binding_objc_selector("defaultManager"));
    }

    URLForDirectory :: (self: *NSFileManager, dir: NSSearchPathDirectory, inDomain: NSSearchPathDomainMask, appropriateForURL: *NSURL, create: BOOL, error: **NSError) -> *NSURL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(dir), a1: type_of(inDomain), a2: type_of(appropriateForURL), a3: type_of(create), a4: type_of(error)) -> *NSURL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSURL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("URLForDirectory:inDomain:appropriateForURL:create:error:"), dir, inDomain, appropriateForURL, create, error);
    }
}

NSHost :: struct {
    using #as super: NSObject;

    currentHost :: () -> *NSHost {
        Send :: #type (receiver: *void, op: *void) -> *NSHost #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSHost) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSHost"), binding_objc_selector("currentHost"));
    }

    localizedName :: (self: *NSHost) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("localizedName"));
    }
}

NSNotification :: struct {
    using #as super: NSObject;

    object :: (self: *NSNotification) -> *NSObject {
        Send :: #type (receiver: *void, op: *void) -> *NSObject #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSObject) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("object"));
    }
}

NSNotificationCenter :: struct {
    using #as super: NSObject;

    defaultCenter :: () -> *NSNotificationCenter {
        Send :: #type (receiver: *void, op: *void) -> *NSNotificationCenter #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSNotificationCenter) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSNotificationCenter"), binding_objc_selector("defaultCenter"));
    }

    addObserver :: (self: *NSNotificationCenter, observer: id, selector: Selector, name: NSNotificationName, object: id) {
        Send :: #type (receiver: *void, op: *void, a0: type_of(observer), a1: type_of(selector), a2: type_of(name), a3: type_of(object)) -> void #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("addObserver:selector:name:object:"), observer, selector, name, object);
    }
}

#if CPU == .X64 {
    NSObject :: struct {
        isa: Class;
    }

}

#if CPU == .ARM64 {
    NSObject :: struct {
        isa: Class;
    }

}

NSOperatingSystemVersion :: struct {
    majorVersion: NSInteger;
    minorVersion: NSInteger;
    patchVersion: NSInteger;
}

NSPoint :: struct {
    x: CGFloat;
    y: CGFloat;
}

NSProcessInfo :: struct {
    using nsobject: NSObject;
    _environment: *NSDictionary(*NSString, *NSString);
    _arguments: *NSArray(*NSString);
    _hostName: *NSString;
    _name: *NSString;
    automaticTerminationOptOutCounter: NSInteger;

    operatingSystem :: (self: *NSProcessInfo) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("operatingSystem"));
    }

    operatingSystemName :: (self: *NSProcessInfo) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("operatingSystemName"));
    }

    isOperatingSystemAtLeastVersion :: (self: *NSProcessInfo, version: NSOperatingSystemVersion) -> BOOL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(version)) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isOperatingSystemAtLeastVersion:"), version);
    }

    disableSuddenTermination :: (self: *NSProcessInfo) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("disableSuddenTermination"));
    }

    enableSuddenTermination :: (self: *NSProcessInfo) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("enableSuddenTermination"));
    }

    disableAutomaticTermination :: (self: *NSProcessInfo, reason: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(reason)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("disableAutomaticTermination:"), reason);
    }

    enableAutomaticTermination :: (self: *NSProcessInfo, reason: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(reason)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("enableAutomaticTermination:"), reason);
    }

    processInfo :: () -> *NSProcessInfo {
        Send :: #type (receiver: *void, op: *void) -> *NSProcessInfo #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSProcessInfo) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSProcessInfo"), binding_objc_selector("processInfo"));
    }

    environment :: (self: *NSProcessInfo) -> *NSDictionary {
        Send :: #type (receiver: *void, op: *void) -> *NSDictionary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSDictionary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("environment"));
    }

    arguments :: (self: *NSProcessInfo) -> *NSArray(*NSString) {
        Send :: #type (receiver: *void, op: *void) -> *NSArray(*NSString) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(*NSString)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("arguments"));
    }

    hostName :: (self: *NSProcessInfo) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("hostName"));
    }

    processName :: (self: *NSProcessInfo) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("processName"));
    }

    setProcessName :: (self: *NSProcessInfo, processName: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(processName)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setProcessName:"), processName);
    }

    processIdentifier :: (self: *NSProcessInfo) -> s32 {
        Send :: #type (receiver: *void, op: *void) -> s32 #c_call;
        send: Send;

        #if CPU == .X64 && size_of(s32) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("processIdentifier"));
    }

    globallyUniqueString :: (self: *NSProcessInfo) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("globallyUniqueString"));
    }

    operatingSystemVersionString :: (self: *NSProcessInfo) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("operatingSystemVersionString"));
    }

    operatingSystemVersion :: (self: *NSProcessInfo) -> NSOperatingSystemVersion {
        Send :: #type (receiver: *void, op: *void) -> NSOperatingSystemVersion #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSOperatingSystemVersion) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("operatingSystemVersion"));
    }

    processorCount :: (self: *NSProcessInfo) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("processorCount"));
    }

    activeProcessorCount :: (self: *NSProcessInfo) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("activeProcessorCount"));
    }

    physicalMemory :: (self: *NSProcessInfo) -> u64 {
        Send :: #type (receiver: *void, op: *void) -> u64 #c_call;
        send: Send;

        #if CPU == .X64 && size_of(u64) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("physicalMemory"));
    }

    systemUptime :: (self: *NSProcessInfo) -> NSTimeInterval {
        Send :: #type (receiver: *void, op: *void) -> NSTimeInterval #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSTimeInterval) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("systemUptime"));
    }

    automaticTerminationSupportEnabled :: (self: *NSProcessInfo) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("automaticTerminationSupportEnabled"));
    }

    setAutomaticTerminationSupportEnabled :: (self: *NSProcessInfo, automaticTerminationSupportEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(automaticTerminationSupportEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setAutomaticTerminationSupportEnabled:"), automaticTerminationSupportEnabled);
    }

}

using NSProcessInfoThermalState :: enum NSInteger {
    NSProcessInfoThermalStateNominal :: 0;
    NSProcessInfoThermalStateFair :: 1;
    NSProcessInfoThermalStateSerious :: 2;
    NSProcessInfoThermalStateCritical :: 3;
}

NSRange :: struct {
    location: NSUInteger;
    length: NSUInteger;
}

NSRect :: struct {
    origin: NSPoint;
    size: NSSize;
}

NSRunLoop :: struct {
    using #as super: NSObject;

    currentRunLoop :: () -> *NSRunLoop {
        Send :: #type (receiver: *void, op: *void) -> *NSRunLoop #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSRunLoop) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSRunLoop"), binding_objc_selector("currentRunLoop"));
    }

    run :: (self: *NSRunLoop) {
        Send :: #type (receiver: *void, op: *void) -> void #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("run"));
    }
}

using NSSearchPathDirectory :: enum NSUInteger {
    NSApplicationDirectory :: 1;
    NSDemoApplicationDirectory;
    NSDeveloperApplicationDirectory;
    NSAdminApplicationDirectory;
    NSLibraryDirectory;
    NSDeveloperDirectory;
    NSUserDirectory;
    NSDocumentationDirectory;
    NSDocumentDirectory;
    NSCoreServiceDirectory;
    NSAutosavedInformationDirectory :: 11;
    NSDesktopDirectory :: 12;
    NSCachesDirectory :: 13;
    NSApplicationSupportDirectory :: 14;
    NSDownloadsDirectory :: 15;
    NSInputMethodsDirectory :: 16;
    NSMoviesDirectory :: 17;
    NSMusicDirectory :: 18;
    NSPicturesDirectory :: 19;
    NSPrinterDescriptionDirectory :: 20;
    NSSharedPublicDirectory :: 21;
    NSPreferencePanesDirectory :: 22;
    NSApplicationScriptsDirectory :: 23;
    NSItemReplacementDirectory :: 99;
    NSAllApplicationsDirectory :: 100;
    NSAllLibrariesDirectory :: 101;
    NSTrashDirectory :: 102;
}

using NSSearchPathDomainMask :: enum_flags NSUInteger {
    NSUserDomainMask :: 1;
    NSLocalDomainMask :: 2;
    NSNetworkDomainMask :: 4;
    NSSystemDomainMask :: 8;
    NSAllDomainsMask :: 0x0ffff;
}

NSString :: struct {
    using #as super: NSObject;

    initWithBytes :: (self: *NSString, bytes: *void, length: NSUInteger, encoding: NSStringEncoding) -> *NSString {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bytes), a1: type_of(length), a2: type_of(encoding)) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("initWithBytes:length:encoding:"), bytes, length, encoding);
    }

    UTF8String :: (self: *NSString) -> *u8 {
        Send :: #type (receiver: *void, op: *void) -> *u8 #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*u8) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("UTF8String"));
    }

    dataUsingEncoding :: (self: *NSString, encoding: NSStringEncoding, allowLossyConversion := NO) -> *NSData {
        Send :: #type (receiver: *void, op: *void, a0: type_of(encoding), a1: type_of(allowLossyConversion)) -> *NSData #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSData) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("dataUsingEncoding:allowLossyConversion:"), encoding, allowLossyConversion);
    }

    stringByExpandingTildeInPath :: (self: *NSString) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stringByExpandingTildeInPath"));
    }

    length :: (self: *NSString) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("length"));
    }

    initWithString :: (self: *NSString, str: string) -> *NSString {
        return initWithBytes(self, str.data, cast(NSUInteger) str.count, NSUTF8StringEncoding);
    }

    getTempString :: (str: string) -> *NSString {
        object := cast(*NSString) objc_alloc(objc_getClass("NSString"));
        object = initWithBytes(object, str.data, cast(NSUInteger) str.count, NSUTF8StringEncoding);
        return cast(*NSString) autorelease(object);
    }
}

NSThread :: struct {
    _nsobject: NSObject;

    detachNewThreadSelectorToTargetWithObject :: (sel: Selector, toTarget: id, withObject: id) {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sel), a1: type_of(toTarget), a2: type_of(withObject)) -> void #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(binding_objc_class("NSThread"), binding_objc_selector("detachNewThreadSelector:toTarget:withObject:"), sel, toTarget, withObject);
    }

    isMultiThreaded :: () -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSThread"), binding_objc_selector("isMultiThreaded"));
    }
}

NSTimer :: struct {
    using #as super: NSObject;

    scheduledTimerWithTimeInterval :: (interval: NSTimeInterval, target: id, selector: Selector, userInfo: id, repeats: BOOL) -> *NSTimer {
        Send :: #type (receiver: *void, op: *void, a0: type_of(interval), a1: type_of(target), a2: type_of(selector), a3: type_of(userInfo), a4: type_of(repeats)) -> *NSTimer #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSTimer) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSTimer"), binding_objc_selector("scheduledTimerWithTimeInterval:target:selector:userInfo:repeats:"), interval, target, selector, userInfo, repeats);
    }
}

NSURL :: struct {
    using #as super: NSObject;

    URLWithString :: (str: *NSString) -> *NSURL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(str)) -> *NSURL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSURL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSURL"), binding_objc_selector("URLWithString:"), str);
    }

    fileURLWithPath :: (path: *NSString) -> *NSURL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(path)) -> *NSURL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSURL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSURL"), binding_objc_selector("fileURLWithPath:"), path);
    }

    fileURLWithPath :: (path: *NSString, isDirectory: BOOL) -> *NSURL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(path), a1: type_of(isDirectory)) -> *NSURL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSURL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("NSURL"), binding_objc_selector("fileURLWithPath:isDirectory:"), path, isDirectory);
    }

    path :: (self: *NSURL) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("path"));
    }

    URLByDeletingLastPathComponent :: (self: *NSURL) -> *NSURL {
        Send :: #type (receiver: *void, op: *void) -> *NSURL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSURL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("URLByDeletingLastPathComponent"));
    }

    hasDirectoryPath :: (self: *NSURL) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("hasDirectoryPath"));
    }

    URLByAppendingPathComponent :: (self: *NSURL, comp: *NSString) -> *NSURL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(comp)) -> *NSURL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSURL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("URLByAppendingPathComponent:"), comp);
    }

    lastPathComponent :: (self: *NSURL) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("lastPathComponent"));
    }
}

#if CPU == .X64 {
    OBJC_ASSOCIATION :: enum u32 {
        ASSIGN :: 0;
        RETAIN_NONATOMIC :: 1;

        COPY_NONATOMIC :: 3;

        RETAIN :: 769;

        COPY :: 771;

        OBJC_ASSOCIATION_ASSIGN :: ASSIGN;
        OBJC_ASSOCIATION_RETAIN_NONATOMIC :: RETAIN_NONATOMIC;

        OBJC_ASSOCIATION_COPY_NONATOMIC :: COPY_NONATOMIC;

        OBJC_ASSOCIATION_RETAIN :: RETAIN;

        OBJC_ASSOCIATION_COPY :: COPY;
    }

}

#if CPU == .ARM64 {
    OBJC_ASSOCIATION :: enum u32 {
        ASSIGN :: 0;
        RETAIN_NONATOMIC :: 1;

        COPY_NONATOMIC :: 3;

        RETAIN :: 769;

        COPY :: 771;

        OBJC_ASSOCIATION_ASSIGN :: ASSIGN;
        OBJC_ASSOCIATION_RETAIN_NONATOMIC :: RETAIN_NONATOMIC;

        OBJC_ASSOCIATION_COPY_NONATOMIC :: COPY_NONATOMIC;

        OBJC_ASSOCIATION_RETAIN :: RETAIN;

        OBJC_ASSOCIATION_COPY :: COPY;
    }

}

#if CPU == .X64 {
    objc_cache :: struct {
        mask: u32;
        occupied: u32;
        buckets: [1] Method;
    }

}

#if CPU == .ARM64 {
    objc_cache :: struct {
        mask: u32;
        occupied: u32;
        buckets: [1] Method;
    }

}

#if CPU == .X64 {
    objc_category :: struct {
        category_name: *u8;
        class_name: *u8;
        instance_methods: *objc_method_list;
        class_methods: *objc_method_list;
        protocols: *objc_protocol_list;
    }

}

#if CPU == .ARM64 {
    objc_category :: struct {
        category_name: *u8;
        class_name: *u8;
        instance_methods: *objc_method_list;
        class_methods: *objc_method_list;
        protocols: *objc_protocol_list;
    }

}

#if CPU == .X64 {
    objc_class :: struct {
        #as using super: NSObject;
        super_class: Class;
        name: *u8;
        version: s64;
        info: s64;
        instance_size: s64;
        ivars: *objc_ivar_list;
        methodLists: **objc_method_list;
        cache: *objc_cache;
        protocols: *objc_protocol_list;
    }

}

#if CPU == .ARM64 {
    objc_class :: struct {
        #as using super: NSObject;
        super_class: Class;
        name: *u8;
        version: s64;
        info: s64;
        instance_size: s64;
        ivars: *objc_ivar_list;
        methodLists: **objc_method_list;
        cache: *objc_cache;
        protocols: *objc_protocol_list;
    }

}

#if CPU == .X64 {
    objc_ivar :: struct {
        ivar_name: *u8;
        ivar_type: *u8;
        ivar_offset: s32;

        space: s32;
    }

}

#if CPU == .ARM64 {
    objc_ivar :: struct {
        ivar_name: *u8;
        ivar_type: *u8;
        ivar_offset: s32;

        space: s32;
    }

}

#if CPU == .X64 {
    objc_ivar_list :: struct {
        ivar_count: s32;

        space: s32;

        ivar_list: [1] objc_ivar;
    }

}

#if CPU == .ARM64 {
    objc_ivar_list :: struct {
        ivar_count: s32;

        space: s32;

        ivar_list: [1] objc_ivar;
    }

}

#if CPU == .X64 {
    objc_method :: struct {
        method_name: SEL;
        method_types: *u8;
        method_imp: IMP;
    }

}

#if CPU == .ARM64 {
    objc_method :: struct {
        method_name: SEL;
        method_types: *u8;
        method_imp: IMP;
    }

}

#if CPU == .X64 {
    objc_method_description :: struct {
        name: SEL;
        types: *u8;
    }

}

#if CPU == .ARM64 {
    objc_method_description :: struct {
        name: SEL;
        types: *u8;
    }

}

#if CPU == .X64 {
    objc_method_description_list :: struct {
        count: s32;
        list: [1] objc_method_description;
    }

}

#if CPU == .ARM64 {
    objc_method_description_list :: struct {
        count: s32;
        list: [1] objc_method_description;
    }

}

#if CPU == .X64 {
    objc_method_list :: struct {
        obsolete: *objc_method_list;

        method_count: s32;

        space: s32;

        method_list: [1] objc_method;
    }

}

#if CPU == .ARM64 {
    objc_method_list :: struct {
        obsolete: *objc_method_list;

        method_count: s32;

        space: s32;

        method_list: [1] objc_method;
    }

}

#if CPU == .X64 {
    objc_module :: struct {
        version: u64;
        size: u64;
        name: *u8;
        symtab: Symtab;
    }

}

#if CPU == .ARM64 {
    objc_module :: struct {
        version: u64;
        size: u64;
        name: *u8;
        symtab: Symtab;
    }

}

#if CPU == .X64 {
    objc_property :: struct {}

}

#if CPU == .ARM64 {
    objc_property :: struct {}

}

#if CPU == .X64 {
    objc_property_attribute_t :: struct {
        name: *u8;
        value: *u8;
    }

}

#if CPU == .ARM64 {
    objc_property_attribute_t :: struct {
        name: *u8;
        value: *u8;
    }

}

#if CPU == .X64 {
    objc_protocol_list :: struct {
        next: *objc_protocol_list;
        count: s64;
        list: [1] *Protocol;
    }

}

#if CPU == .ARM64 {
    objc_protocol_list :: struct {
        next: *objc_protocol_list;
        count: s64;
        list: [1] *Protocol;
    }

}

#if CPU == .X64 {
    objc_selector :: struct {}

}

#if CPU == .ARM64 {
    objc_selector :: struct {}

}

#if CPU == .X64 {
    objc_super :: struct {

        receiver: id;

        class: Class;
    }

}

#if CPU == .ARM64 {
    objc_super :: struct {

        receiver: id;

        class: Class;
    }

}

#if CPU == .X64 {
    objc_symtab :: struct {
        sel_ref_cnt: u64;
        refs: *SEL;
        cls_def_cnt: u16;
        cat_def_cnt: u16;
        defs: [1] *void;
    }

}

#if CPU == .ARM64 {
    objc_symtab :: struct {
        sel_ref_cnt: u64;
        refs: *SEL;
        cls_def_cnt: u16;
        cat_def_cnt: u16;
        defs: [1] *void;
    }

}

Objective_C_Block_Basic :: struct {
    isa: *void;
    flags: s32;
    reserved: s32;
    invoke: (*void, args: ..Any) -> () #c_call;
    descriptor: *Objective_C_Block_Basic_Descriptor;

}

Objective_C_Block_Basic_Descriptor :: struct {
    reserved: u64;
    size: u64;

    copy_helper: (dst: *void, src: *void) #c_call;
    dispose_helper: (src: *void) #c_call;

    signature: *u8;
}

// Callback and opaque types

#if CPU == .ARM64 {
    _objc_swiftMetadataInitializer :: #type (cls: Class, arg: *void) -> Class #c_call;
}

#if CPU == .X64 {
    IMP :: #type (unknown0: id, unknown1: SEL, __args: ..Any) -> id #c_call;
}

#if CPU == .ARM64 {
    IMP :: #type () -> void #c_call;
}

#if CPU == .ARM64 {
    objc_func_loadImage :: #type (header: *mach_header) -> void #c_call;
}

#if CPU == .ARM64 {
    objc_hook_getClass :: #type (name: *u8, outClass: *Class) -> BOOL #c_call;
}

#if CPU == .ARM64 {
    objc_hook_getImageName :: #type (cls: Class, outImageName: **u8) -> BOOL #c_call;
}

#if CPU == .ARM64 {
    objc_hook_lazyClassNamer :: #type (cls: Class) -> *u8 #c_call;
}

// Aliases and constants

#if CPU != .X64 {
    _objc_msgForward_stret :: _objc_msgForward;
}

#if CPU == .X64 {
    ARITH_SHIFT :: 32;
}

#if CPU == .ARM64 {
    ARITH_SHIFT :: 32;
}

#if CPU == .X64 {
    arith_t :: s64;
}

#if CPU == .ARM64 {
    arith_t :: s64;
}

#if OS == .MACOS {
    BOOL :: s8;
}

#if OS != .MACOS {
    BOOL :: bool;
}

#if CPU == .X64 {
    Cache :: *objc_cache;
}

#if CPU == .ARM64 {
    Cache :: *objc_cache;
}

#if CPU == .X64 {
    Category :: *objc_category;
}

#if CPU == .ARM64 {
    Category :: *objc_category;
}

CGFloat :: float64;

#if CPU == .X64 {
    Class :: *objc_class;
}

#if CPU == .ARM64 {
    Class :: *objc_class;
}

#if CPU != .X64 {
    class_getMethodImplementation_stret :: class_getMethodImplementation;
}

#if CPU == .X64 {
    CLS_CLASS :: 0x1;
}

#if CPU == .ARM64 {
    CLS_CLASS :: 0x1;
}

#if CPU == .X64 {
    CLS_CONSTRUCTING :: 0x10000;
}

#if CPU == .ARM64 {
    CLS_CONSTRUCTING :: 0x10000;
}

#if CPU == .X64 {
    CLS_EXT :: 0x20000;
}

#if CPU == .ARM64 {
    CLS_EXT :: 0x20000;
}

#if CPU == .X64 {
    CLS_FLUSH_CACHE :: 0x20;
}

#if CPU == .ARM64 {
    CLS_FLUSH_CACHE :: 0x20;
}

#if CPU == .X64 {
    CLS_FROM_BUNDLE :: 0x1000;
}

#if CPU == .ARM64 {
    CLS_FROM_BUNDLE :: 0x1000;
}

#if CPU == .X64 {
    CLS_GROW_CACHE :: 0x40;
}

#if CPU == .ARM64 {
    CLS_GROW_CACHE :: 0x40;
}

#if CPU == .X64 {
    CLS_HAS_CXX_STRUCTORS :: 0x2000;
}

#if CPU == .ARM64 {
    CLS_HAS_CXX_STRUCTORS :: 0x2000;
}

#if CPU == .X64 {
    CLS_HAS_LOAD_METHOD :: 0x8000;
}

#if CPU == .ARM64 {
    CLS_HAS_LOAD_METHOD :: 0x8000;
}

#if CPU == .X64 {
    CLS_INITIALIZED :: 0x4;
}

#if CPU == .ARM64 {
    CLS_INITIALIZED :: 0x4;
}

#if CPU == .X64 {
    CLS_INITIALIZING :: 0x800;
}

#if CPU == .ARM64 {
    CLS_INITIALIZING :: 0x800;
}

#if CPU == .X64 {
    CLS_JAVA_CLASS :: 0x400;
}

#if CPU == .ARM64 {
    CLS_JAVA_CLASS :: 0x400;
}

#if CPU == .X64 {
    CLS_JAVA_HYBRID :: 0x200;
}

#if CPU == .ARM64 {
    CLS_JAVA_HYBRID :: 0x200;
}

#if CPU == .X64 {
    CLS_MAPPED :: 0x10;
}

#if CPU == .ARM64 {
    CLS_MAPPED :: 0x10;
}

#if CPU == .X64 {
    CLS_META :: 0x2;
}

#if CPU == .ARM64 {
    CLS_META :: 0x2;
}

#if CPU == .X64 {
    CLS_METHOD_ARRAY :: 0x100;
}

#if CPU == .ARM64 {
    CLS_METHOD_ARRAY :: 0x100;
}

#if CPU == .X64 {
    CLS_NEED_BIND :: 0x80;
}

#if CPU == .ARM64 {
    CLS_NEED_BIND :: 0x80;
}

#if CPU == .X64 {
    CLS_NO_METHOD_ARRAY :: 0x4000;
}

#if CPU == .ARM64 {
    CLS_NO_METHOD_ARRAY :: 0x4000;
}

#if CPU == .X64 {
    CLS_POSING :: 0x8;
}

#if CPU == .ARM64 {
    CLS_POSING :: 0x8;
}

dispatch_data_t :: *NSObject;

dispatch_queue_t :: *NSObject;

#if CPU == .X64 {
    id :: *NSObject;
}

#if CPU == .ARM64 {
    id :: *NSObject;
}

#if CPU == .X64 {
    Ivar :: *objc_ivar;
}

#if CPU == .ARM64 {
    Ivar :: *objc_ivar;
}

#if CPU == .X64 {
    marg_list :: *void;
}

#if CPU == .ARM64 {
    marg_list :: *void;
}

#if CPU == .X64 {
    marg_prearg_size :: 0;
}

#if CPU == .ARM64 {
    marg_prearg_size :: 0;
}

#if CPU == .X64 {
    Method :: *objc_method;
}

#if CPU == .ARM64 {
    Method :: *objc_method;
}

#if CPU != .X64 {
    method_invoke_stret :: method_invoke;
}

#if CPU == .X64 {
    Module :: *objc_module;
}

#if CPU == .ARM64 {
    Module :: *objc_module;
}

NSColorSpaceName :: *NSString;

NSHPUXOperatingSystem :: 4;

NSInteger :: s64;

NSMACHOperatingSystem :: 5;

NSNotificationName :: *NSString;

NSOSF1OperatingSystem :: 7;

NSPasteboardType :: *NSString;

NSRunLoopMode :: *NSString;

NSSize :: CGSize;

NSSolarisOperatingSystem :: 3;

NSStringEncoding :: NSUInteger;

NSSunOSOperatingSystem :: 6;

NSTimeInterval :: float64;

NSUInteger :: u64;

NSUIntegerMax :: 0xFFFFFFFFFFFFFFFF;

NSUTF32LittleEndianStringEncoding :: 0x9c000100;

NSUTF32StringEncoding :: 0x8c000100;

NSUTF8StringEncoding :: 4;

NSWindows95OperatingSystem :: 2;

NSWindowsNTOperatingSystem :: 1;

#if CPU == .ARM64 {
    OBJC_ADDLOADIMAGEFUNC_DEFINED :: 1;
}

#if CPU == .X64 {
    objc_AssociationPolicy :: u64;
}

#if CPU == .ARM64 {
    objc_AssociationPolicy :: u64;
}

#if CPU == .X64 {
    OBJC_BOOL_IS_BOOL :: 0;
}

#if CPU == .ARM64 {
    OBJC_BOOL_IS_BOOL :: 1;
}

#if CPU == .X64 {
    OBJC_BOOL_IS_CHAR :: 1;
}

#if CPU == .ARM64 {
    OBJC_GETCLASSHOOK_DEFINED :: 1;
}

#if CPU != .X64 {
    objc_msgSend_stret :: objc_msgSend;
}

#if CPU != .X64 {
    objc_msgSendSuper_stret :: objc_msgSendSuper;
}

#if CPU == .X64 {
    OBJC_NEXT_METHOD_LIST :: 1;
}

#if CPU == .ARM64 {
    OBJC_NEXT_METHOD_LIST :: 1;
}

#if CPU == .X64 {
    objc_objectptr_t :: *void;
}

#if CPU == .ARM64 {
    objc_objectptr_t :: *void;
}

#if CPU == .X64 {
    objc_property_t :: *objc_property;
}

#if CPU == .ARM64 {
    objc_property_t :: *objc_property;
}

#if CPU == .ARM64 {
    OBJC_REALIZECLASSFROMSWIFT_DEFINED :: 1;
}

#if CPU == .ARM64 {
    OBJC_SETHOOK_LAZYCLASSNAMER_DEFINED :: 1;
}

#if CPU == .X64 {
    OBSOLETE_OBJC_GETCLASSES :: 1;
}

#if CPU == .ARM64 {
    OBSOLETE_OBJC_GETCLASSES :: 1;
}

#if CPU == .X64 {
    Protocol :: NSObject;
}

#if CPU == .ARM64 {
    Protocol :: NSObject;
}

#if CPU == .X64 {
    SEL :: *objc_selector;
}

#if CPU == .ARM64 {
    SEL :: *objc_selector;
}

Selector :: *void;

#if CPU == .X64 {
    STR :: *u8;
}

#if CPU == .ARM64 {
    STR :: *u8;
}

#if CPU == .X64 {
    Symtab :: *objc_symtab;
}

#if CPU == .ARM64 {
    Symtab :: *objc_symtab;
}

#if CPU == .X64 {
    uarith_t :: u64;
}

#if CPU == .ARM64 {
    uarith_t :: u64;
}

// Native storage

YES: BOOL : 1;

NO: BOOL : 0;

// Native entry points

_NSConcreteGlobalBlock :: () #foreign libSystem;

_NSConcreteStackBlock :: () #foreign libSystem;

#if CPU == .X64 {
    _objc_flush_caches :: (cls: Class) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    _objc_flush_caches :: (cls: Class) -> void #foreign libobjc;
}

#if CPU == .X64 {
    _objc_msgForward :: (receiver: id, sel: SEL, __args: ..Any) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    _objc_msgForward :: () -> void #foreign libobjc;
}

#if CPU == .X64 {
    _objc_msgForward_stret :: (receiver: id, sel: SEL, __args: ..Any) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    _objc_realizeClassFromSwift :: (cls: Class, previously: *void) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    class_addIvar :: (cls: Class, name: *u8, size: u64, alignment: u8, types: *u8) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    class_addIvar :: (cls: Class, name: *u8, size: u64, alignment: u8, types: *u8) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    class_addMethod :: (cls: Class, name: SEL, imp: IMP, types: *u8) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    class_addMethod :: (cls: Class, name: SEL, imp: IMP, types: *u8) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    class_addProperty :: (cls: Class, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    class_addProperty :: (cls: Class, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    class_addProtocol :: (cls: Class, protocol: *Protocol) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    class_addProtocol :: (cls: Class, protocol: *Protocol) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    class_conformsToProtocol :: (cls: Class, protocol: *Protocol) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    class_conformsToProtocol :: (cls: Class, protocol: *Protocol) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    class_copyIvarList :: (cls: Class, outCount: *u32) -> *Ivar #foreign libobjc;
}

#if CPU == .ARM64 {
    class_copyIvarList :: (cls: Class, outCount: *u32) -> *Ivar #foreign libobjc;
}

#if CPU == .X64 {
    class_copyMethodList :: (cls: Class, outCount: *u32) -> *Method #foreign libobjc;
}

#if CPU == .ARM64 {
    class_copyMethodList :: (cls: Class, outCount: *u32) -> *Method #foreign libobjc;
}

#if CPU == .X64 {
    class_copyPropertyList :: (cls: Class, outCount: *u32) -> *objc_property_t #foreign libobjc;
}

#if CPU == .ARM64 {
    class_copyPropertyList :: (cls: Class, outCount: *u32) -> *objc_property_t #foreign libobjc;
}

#if CPU == .X64 {
    class_copyProtocolList :: (cls: Class, outCount: *u32) -> **Protocol #foreign libobjc;
}

#if CPU == .ARM64 {
    class_copyProtocolList :: (cls: Class, outCount: *u32) -> **Protocol #foreign libobjc;
}

#if CPU == .X64 {
    class_createInstance :: (cls: Class, extraBytes: u64) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    class_createInstance :: (cls: Class, extraBytes: u64) -> id #foreign libobjc;
}

#if CPU == .X64 {
    class_createInstanceFromZone :: (unknown0: Class, idxIvars: u64, z: *void) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    class_createInstanceFromZone :: (unknown0: Class, idxIvars: u64, z: *void) -> id #foreign libobjc;
}

#if CPU == .X64 {
    class_getClassMethod :: (cls: Class, name: SEL) -> Method #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getClassMethod :: (cls: Class, name: SEL) -> Method #foreign libobjc;
}

#if CPU == .X64 {
    class_getClassVariable :: (cls: Class, name: *u8) -> Ivar #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getClassVariable :: (cls: Class, name: *u8) -> Ivar #foreign libobjc;
}

#if CPU == .X64 {
    class_getImageName :: (cls: Class) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getImageName :: (cls: Class) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    class_getInstanceMethod :: (cls: Class, name: SEL) -> Method #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getInstanceMethod :: (cls: Class, name: SEL) -> Method #foreign libobjc;
}

#if CPU == .X64 {
    class_getInstanceSize :: (cls: Class) -> u64 #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getInstanceSize :: (cls: Class) -> u64 #foreign libobjc;
}

#if CPU == .X64 {
    class_getInstanceVariable :: (cls: Class, name: *u8) -> Ivar #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getInstanceVariable :: (cls: Class, name: *u8) -> Ivar #foreign libobjc;
}

#if CPU == .X64 {
    class_getIvarLayout :: (cls: Class) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getIvarLayout :: (cls: Class) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    class_getMethodImplementation :: (cls: Class, name: SEL) -> IMP #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getMethodImplementation :: (cls: Class, name: SEL) -> IMP #foreign libobjc;
}

#if CPU == .X64 {
    class_getMethodImplementation_stret :: (cls: Class, name: SEL) -> IMP #foreign libobjc;
}

#if CPU == .X64 {
    class_getName :: (cls: Class) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getName :: (cls: Class) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    class_getProperty :: (cls: Class, name: *u8) -> objc_property_t #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getProperty :: (cls: Class, name: *u8) -> objc_property_t #foreign libobjc;
}

#if CPU == .X64 {
    class_getSuperclass :: (cls: Class) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getSuperclass :: (cls: Class) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    class_getVersion :: (cls: Class) -> s32 #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getVersion :: (cls: Class) -> s32 #foreign libobjc;
}

#if CPU == .X64 {
    class_getWeakIvarLayout :: (cls: Class) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    class_getWeakIvarLayout :: (cls: Class) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    class_isMetaClass :: (cls: Class) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    class_isMetaClass :: (cls: Class) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    class_lookupMethod :: (cls: Class, sel: SEL) -> IMP #foreign libobjc;
}

#if CPU == .ARM64 {
    class_lookupMethod :: (cls: Class, sel: SEL) -> IMP #foreign libobjc;
}

#if CPU == .X64 {
    class_replaceMethod :: (cls: Class, name: SEL, imp: IMP, types: *u8) -> IMP #foreign libobjc;
}

#if CPU == .ARM64 {
    class_replaceMethod :: (cls: Class, name: SEL, imp: IMP, types: *u8) -> IMP #foreign libobjc;
}

#if CPU == .X64 {
    class_replaceProperty :: (cls: Class, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    class_replaceProperty :: (cls: Class, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32) -> void #foreign libobjc;
}

#if CPU == .X64 {
    class_respondsToMethod :: (cls: Class, sel: SEL) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    class_respondsToMethod :: (cls: Class, sel: SEL) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    class_respondsToSelector :: (cls: Class, sel: SEL) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    class_respondsToSelector :: (cls: Class, sel: SEL) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    class_setIvarLayout :: (cls: Class, layout: *u8) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    class_setIvarLayout :: (cls: Class, layout: *u8) -> void #foreign libobjc;
}

#if CPU == .X64 {
    class_setSuperclass :: (cls: Class, newSuper: Class) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    class_setSuperclass :: (cls: Class, newSuper: Class) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    class_setVersion :: (cls: Class, version: s32) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    class_setVersion :: (cls: Class, version: s32) -> void #foreign libobjc;
}

#if CPU == .X64 {
    class_setWeakIvarLayout :: (cls: Class, layout: *u8) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    class_setWeakIvarLayout :: (cls: Class, layout: *u8) -> void #foreign libobjc;
}

#if CPU == .X64 {
    imp_getBlock :: (anImp: IMP) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    imp_getBlock :: (anImp: IMP) -> id #foreign libobjc;
}

#if CPU == .X64 {
    imp_implementationWithBlock :: (block: id) -> IMP #foreign libobjc;
}

#if CPU == .ARM64 {
    imp_implementationWithBlock :: (block: id) -> IMP #foreign libobjc;
}

#if CPU == .X64 {
    imp_removeBlock :: (anImp: IMP) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    imp_removeBlock :: (anImp: IMP) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    ivar_getName :: (v: Ivar) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    ivar_getName :: (v: Ivar) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    ivar_getOffset :: (v: Ivar) -> ptrdiff_t #foreign libobjc;
}

#if CPU == .ARM64 {
    ivar_getOffset :: (v: Ivar) -> ptrdiff_t #foreign libobjc;
}

#if CPU == .X64 {
    ivar_getTypeEncoding :: (v: Ivar) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    ivar_getTypeEncoding :: (v: Ivar) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    method_copyArgumentType :: (m: Method, index: u32) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    method_copyArgumentType :: (m: Method, index: u32) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    method_copyReturnType :: (m: Method) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    method_copyReturnType :: (m: Method) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    method_exchangeImplementations :: (m1: Method, m2: Method) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    method_exchangeImplementations :: (m1: Method, m2: Method) -> void #foreign libobjc;
}

#if CPU == .X64 {
    method_getArgumentType :: (m: Method, index: u32, dst: *u8, dst_len: u64) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    method_getArgumentType :: (m: Method, index: u32, dst: *u8, dst_len: u64) -> void #foreign libobjc;
}

#if CPU == .X64 {
    method_getDescription :: (m: Method) -> *objc_method_description #foreign libobjc;
}

#if CPU == .ARM64 {
    method_getDescription :: (m: Method) -> *objc_method_description #foreign libobjc;
}

#if CPU == .X64 {
    method_getImplementation :: (m: Method) -> IMP #foreign libobjc;
}

#if CPU == .ARM64 {
    method_getImplementation :: (m: Method) -> IMP #foreign libobjc;
}

#if CPU == .X64 {
    method_getName :: (m: Method) -> SEL #foreign libobjc;
}

#if CPU == .ARM64 {
    method_getName :: (m: Method) -> SEL #foreign libobjc;
}

#if CPU == .X64 {
    method_getNumberOfArguments :: (m: Method) -> u32 #foreign libobjc;
}

#if CPU == .ARM64 {
    method_getNumberOfArguments :: (m: Method) -> u32 #foreign libobjc;
}

#if CPU == .X64 {
    method_getReturnType :: (m: Method, dst: *u8, dst_len: u64) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    method_getReturnType :: (m: Method, dst: *u8, dst_len: u64) -> void #foreign libobjc;
}

#if CPU == .X64 {
    method_getTypeEncoding :: (m: Method) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    method_getTypeEncoding :: (m: Method) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    method_invoke :: (receiver: id, m: Method, __args: ..Any) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    method_invoke :: () -> void #foreign libobjc;
}

#if CPU == .X64 {
    method_invoke_stret :: (receiver: id, m: Method, __args: ..Any) -> void #foreign libobjc;
}

#if CPU == .X64 {
    method_setImplementation :: (m: Method, imp: IMP) -> IMP #foreign libobjc;
}

#if CPU == .ARM64 {
    method_setImplementation :: (m: Method, imp: IMP) -> IMP #foreign libobjc;
}

NSFullUserName :: () -> *NSString #foreign Foundation_Framework;

NSHomeDirectory :: () -> *NSString #foreign Foundation_Framework;

NSHomeDirectoryForUser :: (username: *NSString) -> *NSString #foreign Foundation_Framework;

NSUserName :: () -> *NSString #foreign Foundation_Framework;

#if CPU == .ARM64 {
    objc_addLoadImageFunc :: (func: objc_func_loadImage) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_allocateClassPair :: (superclass: Class, name: *u8, extraBytes: u64) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_allocateClassPair :: (superclass: Class, name: *u8, extraBytes: u64) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    objc_allocateProtocol :: (name: *u8) -> *Protocol #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_allocateProtocol :: (name: *u8) -> *Protocol #foreign libobjc;
}

#if CPU == .X64 {
    objc_constructInstance :: (cls: Class, bytes: *void) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_constructInstance :: (cls: Class, bytes: *void) -> id #foreign libobjc;
}

#if CPU == .X64 {
    objc_copyClassList :: (outCount: *u32) -> *Class #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_copyClassList :: (outCount: *u32) -> *Class #foreign libobjc;
}

#if CPU == .X64 {
    objc_copyClassNamesForImage :: (image: *u8, outCount: *u32) -> **u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_copyClassNamesForImage :: (image: *u8, outCount: *u32) -> **u8 #foreign libobjc;
}

#if CPU == .X64 {
    objc_copyImageNames :: (outCount: *u32) -> **u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_copyImageNames :: (outCount: *u32) -> **u8 #foreign libobjc;
}

#if CPU == .X64 {
    objc_copyProtocolList :: (outCount: *u32) -> **Protocol #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_copyProtocolList :: (outCount: *u32) -> **Protocol #foreign libobjc;
}

#if CPU == .X64 {
    objc_destructInstance :: (obj: id) -> *void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_destructInstance :: (obj: id) -> *void #foreign libobjc;
}

#if CPU == .X64 {
    objc_disposeClassPair :: (cls: Class) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_disposeClassPair :: (cls: Class) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_duplicateClass :: (original: Class, name: *u8, extraBytes: u64) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_duplicateClass :: (original: Class, name: *u8, extraBytes: u64) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    objc_enumerationMutation :: (obj: id) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_enumerationMutation :: (obj: id) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_getAssociatedObject :: (object: id, key: *void) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_getAssociatedObject :: (object: id, key: *void) -> id #foreign libobjc;
}

#if CPU == .X64 {
    objc_getClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_getClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    objc_getClassList :: (buffer: *Class, bufferCount: s32) -> s32 #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_getClassList :: (buffer: *Class, bufferCount: s32) -> s32 #foreign libobjc;
}

#if CPU == .X64 {
    objc_getFutureClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_getFutureClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    objc_getMetaClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_getMetaClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    objc_getProtocol :: (name: *u8) -> *Protocol #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_getProtocol :: (name: *u8) -> *Protocol #foreign libobjc;
}

#if CPU == .X64 {
    objc_getRequiredClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_getRequiredClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    objc_loadWeak :: (location: *id) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_loadWeak :: (location: *id) -> id #foreign libobjc;
}

#if CPU == .X64 {
    objc_lookUpClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_lookUpClass :: (name: *u8) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    objc_msgSend :: (self: id, op: SEL, __args: ..Any) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_msgSend :: () -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_msgSend_stret :: (self: id, op: SEL, __args: ..Any) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_msgSendSuper :: (super: *objc_super, op: SEL, __args: ..Any) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_msgSendSuper :: () -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_msgSendSuper_stret :: (super: *objc_super, op: SEL, __args: ..Any) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_registerClassPair :: (cls: Class) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_registerClassPair :: (cls: Class) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_registerProtocol :: (proto: *Protocol) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_registerProtocol :: (proto: *Protocol) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_removeAssociatedObjects :: (object: id) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_removeAssociatedObjects :: (object: id) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_retainedObject :: (obj: objc_objectptr_t) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_retainedObject :: (obj: objc_objectptr_t) -> id #foreign libobjc;
}

#if CPU == .X64 {
    objc_setAssociatedObject :: (object: id, key: *void, value: id, policy: objc_AssociationPolicy) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_setAssociatedObject :: (object: id, key: *void, value: id, policy: objc_AssociationPolicy) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_setEnumerationMutationHandler :: (handler: #type (unknown0: id) -> void #c_call) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_setEnumerationMutationHandler :: (handler: #type (unknown0: id) -> void #c_call) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_setForwardHandler :: (fwd: *void, fwd_stret: *void) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_setForwardHandler :: (fwd: *void, fwd_stret: *void) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_setHook_getClass :: (newValue: objc_hook_getClass, outOldValue: *objc_hook_getClass) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_setHook_getImageName :: (newValue: objc_hook_getImageName, outOldValue: *objc_hook_getImageName) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_setHook_lazyClassNamer :: (newValue: objc_hook_lazyClassNamer, oldOutValue: *objc_hook_lazyClassNamer) -> void #foreign libobjc;
}

#if CPU == .X64 {
    objc_storeWeak :: (location: *id, obj: id) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_storeWeak :: (location: *id, obj: id) -> id #foreign libobjc;
}

#if CPU == .X64 {
    objc_unretainedObject :: (obj: objc_objectptr_t) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_unretainedObject :: (obj: objc_objectptr_t) -> id #foreign libobjc;
}

#if CPU == .X64 {
    objc_unretainedPointer :: (obj: id) -> objc_objectptr_t #foreign libobjc;
}

#if CPU == .ARM64 {
    objc_unretainedPointer :: (obj: id) -> objc_objectptr_t #foreign libobjc;
}

#if CPU == .X64 {
    object_copy :: (obj: id, size: u64) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    object_copy :: (obj: id, size: u64) -> id #foreign libobjc;
}

#if CPU == .X64 {
    object_copyFromZone :: (anObject: id, nBytes: u64, z: *void) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    object_copyFromZone :: (anObject: id, nBytes: u64, z: *void) -> id #foreign libobjc;
}

#if CPU == .X64 {
    object_dispose :: (obj: id) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    object_dispose :: (obj: id) -> id #foreign libobjc;
}

#if CPU == .X64 {
    object_getClass :: (obj: id) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    object_getClass :: (obj: id) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    object_getClassName :: (obj: id) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    object_getClassName :: (obj: id) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    object_getIndexedIvars :: (obj: id) -> *void #foreign libobjc;
}

#if CPU == .ARM64 {
    object_getIndexedIvars :: (obj: id) -> *void #foreign libobjc;
}

#if CPU == .X64 {
    object_getInstanceVariable :: (obj: id, name: *u8, outValue: **void) -> Ivar #foreign libobjc;
}

#if CPU == .ARM64 {
    object_getInstanceVariable :: (obj: id, name: *u8, outValue: **void) -> Ivar #foreign libobjc;
}

#if CPU == .X64 {
    object_getIvar :: (obj: id, ivar: Ivar) -> id #foreign libobjc;
}

#if CPU == .ARM64 {
    object_getIvar :: (obj: id, ivar: Ivar) -> id #foreign libobjc;
}

#if CPU == .X64 {
    object_isClass :: (obj: id) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    object_isClass :: (obj: id) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    object_setClass :: (obj: id, cls: Class) -> Class #foreign libobjc;
}

#if CPU == .ARM64 {
    object_setClass :: (obj: id, cls: Class) -> Class #foreign libobjc;
}

#if CPU == .X64 {
    object_setInstanceVariable :: (obj: id, name: *u8, value: *void) -> Ivar #foreign libobjc;
}

#if CPU == .ARM64 {
    object_setInstanceVariable :: (obj: id, name: *u8, value: *void) -> Ivar #foreign libobjc;
}

#if CPU == .X64 {
    object_setInstanceVariableWithStrongDefault :: (obj: id, name: *u8, value: *void) -> Ivar #foreign libobjc;
}

#if CPU == .ARM64 {
    object_setInstanceVariableWithStrongDefault :: (obj: id, name: *u8, value: *void) -> Ivar #foreign libobjc;
}

#if CPU == .X64 {
    object_setIvar :: (obj: id, ivar: Ivar, value: id) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    object_setIvar :: (obj: id, ivar: Ivar, value: id) -> void #foreign libobjc;
}

#if CPU == .X64 {
    object_setIvarWithStrongDefault :: (obj: id, ivar: Ivar, value: id) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    object_setIvarWithStrongDefault :: (obj: id, ivar: Ivar, value: id) -> void #foreign libobjc;
}

#if CPU == .X64 {
    property_copyAttributeList :: (property: objc_property_t, outCount: *u32) -> *objc_property_attribute_t #foreign libobjc;
}

#if CPU == .ARM64 {
    property_copyAttributeList :: (property: objc_property_t, outCount: *u32) -> *objc_property_attribute_t #foreign libobjc;
}

#if CPU == .X64 {
    property_copyAttributeValue :: (property: objc_property_t, attributeName: *u8) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    property_copyAttributeValue :: (property: objc_property_t, attributeName: *u8) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    property_getAttributes :: (property: objc_property_t) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    property_getAttributes :: (property: objc_property_t) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    property_getName :: (property: objc_property_t) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    property_getName :: (property: objc_property_t) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    protocol_addMethodDescription :: (proto: *Protocol, name: SEL, types: *u8, isRequiredMethod: BOOL, isInstanceMethod: BOOL) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_addMethodDescription :: (proto: *Protocol, name: SEL, types: *u8, isRequiredMethod: BOOL, isInstanceMethod: BOOL) -> void #foreign libobjc;
}

#if CPU == .X64 {
    protocol_addProperty :: (proto: *Protocol, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_addProperty :: (proto: *Protocol, name: *u8, attributes: *objc_property_attribute_t, attributeCount: u32, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> void #foreign libobjc;
}

#if CPU == .X64 {
    protocol_addProtocol :: (proto: *Protocol, addition: *Protocol) -> void #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_addProtocol :: (proto: *Protocol, addition: *Protocol) -> void #foreign libobjc;
}

#if CPU == .X64 {
    protocol_conformsToProtocol :: (proto: *Protocol, other: *Protocol) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_conformsToProtocol :: (proto: *Protocol, other: *Protocol) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    protocol_copyMethodDescriptionList :: (proto: *Protocol, isRequiredMethod: BOOL, isInstanceMethod: BOOL, outCount: *u32) -> *objc_method_description #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_copyMethodDescriptionList :: (proto: *Protocol, isRequiredMethod: BOOL, isInstanceMethod: BOOL, outCount: *u32) -> *objc_method_description #foreign libobjc;
}

#if CPU == .X64 {
    protocol_copyPropertyList :: (proto: *Protocol, outCount: *u32) -> *objc_property_t #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_copyPropertyList :: (proto: *Protocol, outCount: *u32) -> *objc_property_t #foreign libobjc;
}

#if CPU == .X64 {
    protocol_copyPropertyList2 :: (proto: *Protocol, outCount: *u32, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> *objc_property_t #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_copyPropertyList2 :: (proto: *Protocol, outCount: *u32, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> *objc_property_t #foreign libobjc;
}

#if CPU == .X64 {
    protocol_copyProtocolList :: (proto: *Protocol, outCount: *u32) -> **Protocol #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_copyProtocolList :: (proto: *Protocol, outCount: *u32) -> **Protocol #foreign libobjc;
}

#if CPU == .X64 {
    protocol_getMethodDescription :: (proto: *Protocol, aSel: SEL, isRequiredMethod: BOOL, isInstanceMethod: BOOL) -> objc_method_description #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_getMethodDescription :: (proto: *Protocol, aSel: SEL, isRequiredMethod: BOOL, isInstanceMethod: BOOL) -> objc_method_description #foreign libobjc;
}

#if CPU == .X64 {
    protocol_getName :: (proto: *Protocol) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_getName :: (proto: *Protocol) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    protocol_getProperty :: (proto: *Protocol, name: *u8, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> objc_property_t #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_getProperty :: (proto: *Protocol, name: *u8, isRequiredProperty: BOOL, isInstanceProperty: BOOL) -> objc_property_t #foreign libobjc;
}

#if CPU == .X64 {
    protocol_isEqual :: (proto: *Protocol, other: *Protocol) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    protocol_isEqual :: (proto: *Protocol, other: *Protocol) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    sel_getName :: (sel: SEL) -> *u8 #foreign libobjc;
}

#if CPU == .ARM64 {
    sel_getName :: (sel: SEL) -> *u8 #foreign libobjc;
}

#if CPU == .X64 {
    sel_getUid :: (str: *u8) -> SEL #foreign libobjc;
}

#if CPU == .ARM64 {
    sel_getUid :: (str: *u8) -> SEL #foreign libobjc;
}

#if CPU == .X64 {
    sel_isEqual :: (lhs: SEL, rhs: SEL) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    sel_isEqual :: (lhs: SEL, rhs: SEL) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    sel_isMapped :: (sel: SEL) -> BOOL #foreign libobjc;
}

#if CPU == .ARM64 {
    sel_isMapped :: (sel: SEL) -> BOOL #foreign libobjc;
}

#if CPU == .X64 {
    sel_registerName :: (str: *u8) -> SEL #foreign libobjc;
}

#if CPU == .ARM64 {
    sel_registerName :: (str: *u8) -> SEL #foreign libobjc;
}

// Convenience operations

autorelease :: (self: *$instancetype) -> id #no_context {
    Send :: #type (receiver: *void, op: *void) -> id #c_call;
    send := cast(Send) binding_objc_send;
    return send(cast(*void) self, binding_objc_selector("autorelease"));
}

class :: (self: id) -> Class #no_context {
    return object_getClass(self);
}

class :: ($type: Type) -> Class #no_context {
    name := Native.terminated_copy(type_info(type).name);
    defer Native.release(name);
    return cast(Class) binding_objc_class(name);
}

init_objective_c :: () {
    if !binding_objc_class("NSObject") binding_abort();
}

init_objective_c_foundation :: () {
    if !binding_objc_class("NSString") binding_abort();
}

init_objective_c_selector_struct :: (selectors: *$T) {
    for member: type_info(T).members {
        if (member.flags & .CONSTANT) != 0 continue;
        if member.type != type_info(Selector) binding_abort();

        text := Native.terminated_copy(member.name);
        for i: 0..member.name.count - 1 if text[i] == #char "_" text[i] = #char ":";

        slot := cast(*Selector) (cast(*u8) selectors + member.offset_in_bytes);
        <<slot = binding_objc_selector(text);
        Native.release(text);
    }
}

isEqual :: (self: id, object: id) -> bool #no_context {
    Send :: #type (receiver: *void, op: *void, a0: type_of(object)) -> BOOL #c_call;
    send := cast(Send) binding_objc_send;
    return send(cast(*void) self, binding_objc_selector("isEqual:"), object) != NO;
}

NSMakePoint :: (x: CGFloat, y: CGFloat) -> NSPoint {
    return NSPoint.{ x, y };
}

NSMakeRect :: (x: CGFloat, y: CGFloat, w: CGFloat, h: CGFloat) -> NSRect {
    return NSRect.{ NSPoint.{ x, y }, NSSize.{ w, h } };
}

NSMakeSize :: (w: CGFloat, h: CGFloat) -> NSSize {
    return NSSize.{ w, h };
}

objc_alloc :: (class: Class) -> id #no_context {
    Send :: #type (receiver: *void, op: *void) -> id #c_call;
    send := cast(Send) binding_objc_send;
    return send(cast(*void) class, binding_objc_selector("alloc"));
}

objc_alloc :: ($type: Type) -> *type #no_context {
    Send :: #type (receiver: *void, op: *void) -> *type #c_call;
    binding_name := Native.terminated_copy(type_info(type).name);

    defer Native.release(binding_name);
    send := cast(Send) binding_objc_send;

    return send(cast(*void) binding_objc_class(binding_name), binding_objc_selector("alloc"));
}

objc_allocateClassPair :: (super: Class, name: string, extra_bytes: u64 = 0) -> Class {
    binding_name := Native.terminated_copy(name);
    defer Native.release(binding_name);
    return objc_allocateClassPair(super, binding_name, extra_bytes);
}

objc_copy :: (self: *$instancetype) -> *instancetype #no_context {
    Send :: #type (receiver: *void, op: *void) -> *instancetype #c_call;
    send := cast(Send) binding_objc_send;
    return send(cast(*void) self, binding_objc_selector("copy"));
}

objc_finalize_class :: (my_class: Class) {
    objc_registerClassPair(my_class);
}

objc_getClass :: (name: string) -> Class {
    binding_name := Native.terminated_copy(name);
    defer Native.release(binding_name);
    return objc_getClass(binding_name);
}

objc_getProtocol :: (name: string) -> *Protocol {
    binding_name := Native.terminated_copy(name);
    defer Native.release(binding_name);
    return objc_getProtocol(binding_name);
}

objc_init :: (self: *$type) -> *type #no_context {
    Send :: #type (receiver: *void, op: *void) -> *type #c_call;
    send := cast(Send) binding_objc_send;
    return send(cast(*void) self, binding_objc_selector("init"));
}

objc_lookUpClass :: (name: string) -> Class {
    binding_name := Native.terminated_copy(name);
    defer Native.release(binding_name);
    return objc_lookUpClass(binding_name);
}

objc_msgSend_typed :: inline (target: *void, selector: Selector) -> *void #no_context {
    Send :: #type (target: *void, op: Selector) -> *void #c_call;
    send := cast(Send) binding_objc_send;
    return send(target, selector);
}

objc_new :: ($type: Type) -> *type #no_context {
    return objc_init(objc_alloc(type));
}

release :: (self: *$instancetype) -> id #no_context {
    Send :: #type (receiver: *void, op: *void) -> void #c_call;
    send := cast(Send) binding_objc_send;
    send(cast(*void) self, binding_objc_selector("release"));
    return null;
}

retain :: (self: *$instancetype) -> *instancetype #no_context {
    Send :: #type (receiver: *void, op: *void) -> *instancetype #c_call;
    send := cast(Send) binding_objc_send;
    return send(cast(*void) self, binding_objc_selector("retain"));
}

sel_getUid :: (name: string) -> Selector {
    binding_name := Native.terminated_copy(name);
    defer Native.release(binding_name);
    return sel_getUid(binding_name);
}

sel_registerName :: (name: string) -> Selector {
    binding_name := Native.terminated_copy(name);
    defer Native.release(binding_name);
    return sel_registerName(binding_name);
}

selector :: (sel_str: string) -> Selector {
    return sel_registerName(sel_str);
}

superclass :: (self: id) -> Class #no_context {
    return class_getSuperclass(object_getClass(self));
}

to_string :: (str: *NSString) -> string {
    text := NSString.UTF8String(str);
    if !text return "";
    return .{ data = text, count = cast(s64) binding_strlen(text) };
}
