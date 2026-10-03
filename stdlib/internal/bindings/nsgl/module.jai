// AppKit owns the canonical public class identity.
NSOpenGLContext :: AppKit.NSOpenGLContext;

create :: (major: s32, minor: s32, compatibility: bool, debug: bool,
           wanted_samples: s32) -> *NSOpenGLContext, s32 {
    if major < 0 || minor < 0 || wanted_samples < 0 || (major == 0 && minor != 0) {
        diagnostic("Invalid NSOpenGL version or sample request");
        return null, 0;
    }

    if debug {
        diagnostic("NSOpenGL exposes no debug-context creation attribute");
        return null, 0;
    }

    profiles: [2] u32;
    count: s32 = 1;

    if major == 0 {
        if compatibility profiles[0] = PROFILE_LEGACY;
        else {
            profiles[0] = PROFILE_4_1;
            profiles[1] = PROFILE_3_2;
            count = 2;
        }
    } else if (major == 1 && minor <= 5) || (major == 2 && minor <= 1) {
        profiles[0] = PROFILE_LEGACY;
    } else if !compatibility && major == 3 && (minor == 1 || minor == 2) {
        profiles[0] = PROFILE_3_2;
    } else if !compatibility && ((major == 3 && minor == 3) || (major == 4 && minor <= 1)) {
        profiles[0] = PROFILE_4_1;
    } else {
        diagnostic("Requested OpenGL version/profile cannot be represented by NSOpenGL");
        return null, 0;
    }

    for i: 0..count - 1 {
        samples := wanted_samples;

        while true {
            attributes: [16] u32;
            attributes[0] = PFA_PROFILE;
            attributes[1] = profiles[i];
            attributes[2] = PFA_COLOR_SIZE;
            attributes[3] = 24;
            attributes[4] = PFA_ALPHA_SIZE;
            attributes[5] = 8;
            attributes[6] = PFA_DEPTH_SIZE;
            attributes[7] = 24;
            attributes[8] = PFA_DOUBLE_BUFFER;

            if samples > 0 {
                attributes[9] = PFA_MULTISAMPLE;
                attributes[10] = PFA_SAMPLE_BUFFERS;
                attributes[11] = 1;
                attributes[12] = PFA_SAMPLES;
                attributes[13] = cast(u32) samples;
            }

            format := new_format(attributes.data);
            if format {
                defer release_object(format);
                result := new_context(format);
                if !result {
                    diagnostic("NSOpenGL failed to initialize a context");
                    return null, 0;
                }

                buffers := format_value(format, PFA_SAMPLE_BUFFERS);
                actual_samples: s32;

                if buffers > 0 actual_samples = format_value(format, PFA_SAMPLES);

                return result, actual_samples;
            }

            if samples == 0 break;

            samples = ifx samples > 1 then samples / 2 else 0;
        }
    }

    diagnostic("NSOpenGL could not find a matching pixel format");

    return null, 0;
}

attach :: (value: *NSOpenGLContext, view: *void) {
    Send :: #type (self: *NSOpenGLContext, op: O.Selector, view: *void) #c_call;
    call := cast(Send) message;
    call(value, O.selector("setView:"), view);
}

make_current :: (value: *NSOpenGLContext) {
    Send :: #type (self: *NSOpenGLContext, op: O.Selector) #c_call;
    call := cast(Send) message;
    call(value, O.selector("makeCurrentContext"));
}

release_object :: (value: *void) {
    Send :: #type (self: *void, op: O.Selector) #c_call;
    call := cast(Send) message;
    call(value, O.selector("release"));
}

diagnostic :: (text: *u8) {
    if native_printf("%s\n", text) < 0 native_abort();
    if native_fflush(null) != 0 native_abort();
}

#scope_file
#assert OS == .MACOS, "Native NSOpenGL requires macOS";

O :: #import "Objective_C";

#library,system,link_always "AppKit";
libobjc :: #library,system "objc";
message :: () #foreign libobjc "objc_msgSend";
libc :: #library,system "c";
native_printf :: (fmt: *u8, args: ..Any) -> s32 #foreign libc "printf";
native_fflush :: (stream: *void) -> s32 #foreign libc "fflush";
native_abort :: () #foreign libc "abort";

// Complete native attribute values/signatures come from pinned NSOpenGL.h.
PFA_DOUBLE_BUFFER :: 5;
PFA_COLOR_SIZE :: 8;
PFA_ALPHA_SIZE :: 11;
PFA_DEPTH_SIZE :: 12;
PFA_SAMPLE_BUFFERS :: 55;
PFA_SAMPLES :: 56;
PFA_MULTISAMPLE :: 59;
PFA_PROFILE :: 99;
PROFILE_LEGACY :: 0x1000;
PROFILE_3_2 :: 0x3200;
PROFILE_4_1 :: 0x4100;

new_format :: (attributes: *u32) -> *void {
    allocated := O.objc_alloc(O.objc_getClass("NSOpenGLPixelFormat"));
    if !allocated return null;

    Init :: #type (self: *void, op: O.Selector, attributes: *u32) -> *void #c_call;
    call := cast(Init) message;

    return call(allocated, O.selector("initWithAttributes:"), attributes);
}

new_context :: (format: *void) -> *NSOpenGLContext {
    allocated := O.objc_alloc(O.objc_getClass("NSOpenGLContext"));
    if !allocated return null;

    Init :: #type (self: *void, op: O.Selector, format: *void, share: *void) -> *NSOpenGLContext #c_call;
    call := cast(Init) message;

    return call(allocated, O.selector("initWithFormat:shareContext:"), format, null);
}

format_value :: (format: *void, attribute: u32) -> s32 {
    Get :: #type (self: *void, op: O.Selector, result: *s32, attribute: u32, screen: s32) #c_call;
    call := cast(Get) message;
    result: s32;

    call(format, O.selector("getValues:forAttribute:forVirtualScreen:"), *result, attribute, 0);

    return result;
}

#scope_file
AppKit :: #import "Objective_C/AppKit";
