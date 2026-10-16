#import <AppKit/AppKit.h>
#import <objc/runtime.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

_Static_assert(sizeof(CGFloat) == 8, "desktop CGFloat ABI");

void *binding_scale_create_receiver(void) {
  // Read-only SDK screen object; no application, view or window is created.
  @autoreleasepool {
    NSArray<NSScreen *> *screens = [NSScreen screens];
    return screens.count ? [screens[0] retain] : nil;
  }
}

void binding_scale_release_receiver(void *value) {
  [(NSScreen *)value release];
}
CGFloat binding_scale_value(void *value) {
  return [(NSScreen *)value backingScaleFactor];
}

static bool instance_contract(Class owner) {
  Method method = class_getInstanceMethod(owner, @selector(backingScaleFactor));
  if (!method || method_getNumberOfArguments(method) != 2)
    return false;
  char *result = method_copyReturnType(method);
  bool valid = result && strcmp(result, @encode(CGFloat)) == 0;
  free(result);
  return valid;
}

bool binding_scale_contract(void) {
  return NSApp == nil && instance_contract(objc_getClass("NSWindow")) &&
         instance_contract(objc_getClass("NSScreen"));
}
