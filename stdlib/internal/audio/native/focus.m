/* AppKit's process-level state is queried without activating the application.
 * Command-line processes have no application focus to lose. */
#define JAI_AUDIO_BUILD 1
#include "output.h"
#import <AppKit/AppKit.h>
#import <CoreGraphics/CoreGraphics.h>
#include <unistd.h>

JAI_AUDIO_API int audio_output_process_active(void) {
    @autoreleasepool {
        NSRunningApplication *application = NSRunningApplication.currentApplication;
        if (!application || application.activationPolicy == NSApplicationActivationPolicyProhibited) return 1;
        CFArrayRef windows = CGWindowListCopyWindowInfo(kCGWindowListOptionAll | kCGWindowListExcludeDesktopElements, kCGNullWindowID);
        if (!windows) return 1; /* No GUI session. */
        int owns_window = 0;
        for (CFIndex index = 0; index < CFArrayGetCount(windows); ++index) {
            CFDictionaryRef window = CFArrayGetValueAtIndex(windows, index);
            CFNumberRef owner = CFDictionaryGetValue(window, kCGWindowOwnerPID);
            CFNumberRef layer = CFDictionaryGetValue(window, kCGWindowLayer);
            int pid = 0, level = -1;
            if (owner) CFNumberGetValue(owner, kCFNumberIntType, &pid);
            if (layer) CFNumberGetValue(layer, kCFNumberIntType, &level);
            if (pid == getpid() && level == 0) { owns_window = 1; break; }
        }
        CFRelease(windows);
        if (!owns_window) return 1;
        return application.active ? 1 : 0;
    }
}
