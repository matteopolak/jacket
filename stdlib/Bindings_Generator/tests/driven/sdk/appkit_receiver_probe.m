// Real offscreen SDK objects and selectors; no application/window creation.
#import <AppKit/AppKit.h>
#import <objc/runtime.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

bool receiver_create(void **parent, void **child, void **event) {
  NSView *p = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 300, 200)];
  NSView *c = [[NSView alloc] initWithFrame:NSMakeRect(10, 12, 100, 100)];
  NSEvent *e = [[NSEvent otherEventWithType:NSEventTypeApplicationDefined
                                   location:NSZeroPoint
                              modifierFlags:0
                                  timestamp:0
                               windowNumber:0
                                    context:nil
                                    subtype:0
                                      data1:0
                                      data2:0] retain];
  if (!p || !c || !e) {
    [e release];
    [c release];
    [p release];
    return false;
  }
  [p addSubview:c];
  *parent = p;
  *child = c;
  *event = e;
  return true;
}

void receiver_destroy(void *parent, void *child, void *event) {
  [(NSView *)child removeFromSuperview];
  [(NSView *)child release];
  [(NSView *)parent release];
  [(NSEvent *)event release];
}

NSPoint receiver_point(void *view, NSPoint point, void *from) {
  return [(NSView *)view convertPoint:point fromView:(NSView *)from];
}
NSTimeInterval receiver_delay(void) { return [NSEvent keyRepeatDelay]; }
NSTimeInterval receiver_interval(void) { return [NSEvent keyRepeatInterval]; }
NSUInteger receiver_buttons(void) { return [NSEvent pressedMouseButtons]; }

static bool signature(Class cls, bool class_method, SEL sel, unsigned arguments,
                      const char *encoding) {
  Method method = class_method ? class_getClassMethod(cls, sel)
                               : class_getInstanceMethod(cls, sel);
  if (!method || method_getNumberOfArguments(method) != arguments)
    return false;
  char *result = method_copyReturnType(method);
  bool valid = result && strcmp(result, encoding) == 0;
  free(result);
  return valid;
}

bool receiver_contract(void) {
  return NSApp == nil && sizeof(NSPoint) == 16 && sizeof(NSTimeInterval) == 8 &&
         sizeof(NSUInteger) == 8 &&
         signature([NSView class], false, @selector(convertPoint:fromView:), 4,
                   @encode(NSPoint)) &&
         signature([NSEvent class], true, @selector(keyRepeatDelay), 2,
                   @encode(NSTimeInterval)) &&
         signature([NSEvent class], true, @selector(keyRepeatInterval), 2,
                   @encode(NSTimeInterval)) &&
         signature([NSEvent class], true, @selector(pressedMouseButtons), 2,
                   @encode(NSUInteger));
}
