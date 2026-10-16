// Independent official-SDK controls. No application is created or activated.
#define GL_SILENCE_DEPRECATION
#import <AppKit/AppKit.h>
#import <objc/runtime.h>
#include <stddef.h>

_Static_assert(sizeof(NSRect) == 32, "NSRect ABI");
_Static_assert(offsetof(NSRect, size) == 16, "NSRect size offset");
_Static_assert(sizeof(NSPoint) == 16, "NSPoint ABI");
_Static_assert(sizeof(NSEventMask) == 8, "NSEventMask ABI");
_Static_assert(sizeof(NSOpenGLPixelFormatAttribute) == 4, "attribute ABI");

void *appkit_probe_application_slot(void) { return &NSApp; }
void *appkit_probe_runloop_value(void) { return (void *)NSDefaultRunLoopMode; }
int appkit_probe_view(void *pointer) {
  NSView *view = (NSView *)pointer;
  NSRect frame = [view frame];
  NSRect bounds = [view bounds];
  return [view isKindOfClass:[NSView class]] && frame.origin.x == 11 &&
         frame.origin.y == 23 && frame.size.width == 71 &&
         frame.size.height == 89 && bounds.size.width == 71 &&
         bounds.size.height == 89 && [[view subviews] count] == 1;
}

int appkit_probe_no_application(void) { return NSApp == nil; }
int appkit_probe_level(int key) { return CGWindowLevelForKey(key); }
int appkit_probe_has_method(const char *name, const char *selector,
                            int class_method) {
  Class cls = objc_getClass(name);
  SEL sel = sel_registerName(selector);
  return (class_method ? class_getClassMethod(cls, sel)
                       : class_getInstanceMethod(cls, sel)) != NULL;
}

void *appkit_probe_key_event(void) {
  @autoreleasepool {
    return [[NSEvent keyEventWithType:NSEventTypeKeyDown
                             location:NSMakePoint(3, 7)
                        modifierFlags:NSEventModifierFlagShift
                            timestamp:0
                         windowNumber:0
                              context:nil
                           characters:@"x"
          charactersIgnoringModifiers:@"x"
                            isARepeat:NO
                              keyCode:42] retain];
  }
}
