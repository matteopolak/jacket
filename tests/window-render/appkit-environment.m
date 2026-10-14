#import <AppKit/AppKit.h>
#include <stdio.h>

int main(void) {
    @autoreleasepool {
        if (!NSApplicationLoad()) return 1;
        NSApplication *app = [NSApplication sharedApplication];
        printf("policy before: %ld\n", (long)[app activationPolicy]);
        BOOL changed = [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        printf("set regular: %d, policy after: %ld\n", changed, (long)[app activationPolicy]);
        [app finishLaunching];
        NSWindow *window = [[NSWindow alloc]
            initWithContentRect:NSMakeRect(40, 60, 320, 200)
            styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskMiniaturizable | NSWindowStyleMaskResizable
            backing:NSBackingStoreBuffered defer:NO];
        if (!window) return 2;
        [window setReleasedWhenClosed:NO];
        [window makeKeyAndOrderFront:nil];
        NSEvent *initial;
        while ((initial = [app nextEventMatchingMask:NSEventMaskAny untilDate:[NSDate distantPast]
                                             inMode:NSDefaultRunLoopMode dequeue:YES])) [app sendEvent:initial];
        [app updateWindows];
        NSRect rect = [window contentRectForFrameRect:[window frame]];
        printf("native content: %.0f %.0f %.0f %.0f\n", rect.origin.x, rect.origin.y,
               rect.size.width, rect.size.height);
        NSEvent *event = [NSEvent keyEventWithType:NSEventTypeKeyDown location:NSZeroPoint
            modifierFlags:NSEventModifierFlagShift timestamp:0 windowNumber:[window windowNumber]
            context:nil characters:@"😀" charactersIgnoringModifiers:@"a" isARepeat:NO keyCode:0];
        printf("native key characters: %s\n", [[event characters] UTF8String]);
        printf("unmodified native characters: %s\n", [[event charactersIgnoringModifiers] UTF8String]);
        printf("native characters after unmodified query: %s\n", [[event characters] UTF8String]);
        [app postEvent:event atStart:NO];
        NSEvent *queued;
        do {
            queued = [app nextEventMatchingMask:NSEventMaskAny untilDate:[NSDate distantPast]
                                         inMode:NSDefaultRunLoopMode dequeue:YES];
        } while (queued && [queued type] != NSEventTypeKeyDown);
        if (queued) printf("queued native characters: %s\n", [[queued characters] UTF8String]);
        [window close];
        [window release];
        return 0;
    }
}
