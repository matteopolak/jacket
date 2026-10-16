/* Independent native SDK oracle: parse real Objective-C encodings and method
 * signatures rather than inspecting the Jai implementation. */
#import <Foundation/Foundation.h>
#import <objc/runtime.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

int binding_objc_encoding_layout(const char *bytes, uint64_t length,
                                 uint64_t expected_size,
                                 uint64_t expected_alignment) {
  if (!bytes || length == UINT64_MAX)
    return 0;
  char *encoding = malloc((size_t)length + 1);
  if (!encoding)
    return 0;
  memcpy(encoding, bytes, (size_t)length);
  encoding[length] = 0;
  NSUInteger size = 0, alignment = 0;
  const char *end = NSGetSizeAndAlignment(encoding, &size, &alignment);
  int valid =
      end && !*end && size == expected_size && alignment == expected_alignment;
  free(encoding);
  return valid;
}

int binding_objc_method_signature(void *class_handle,
                                  const char *selector_name) {
  Method method = class_getInstanceMethod((Class)class_handle,
                                          sel_registerName(selector_name));
  if (!method)
    return 0;
  NSMethodSignature *signature =
      [NSMethodSignature signatureWithObjCTypes:method_getTypeEncoding(method)];
  if (!signature || signature.numberOfArguments != 3)
    return 0;
  const char *receiver = [signature getArgumentTypeAtIndex:0];
  NSUInteger receiver_size = 0, receiver_alignment = 0;
  const char *end = NSGetSizeAndAlignment(receiver, &receiver_size,
                                        &receiver_alignment);
  /* The reviewed compact encoding preserves an explicit pointer-to-record
   * receiver; it does not replace it with an implicit id spelling. Both must
   * retain the actual native pointer argument footprint. */
  int receiver_valid = (*receiver == '@' || *receiver == '^') && end && !*end &&
                       receiver_size == sizeof(id) &&
                       receiver_alignment == _Alignof(id);
  return signature && signature.numberOfArguments == 3 &&
         strcmp(signature.methodReturnType, @encode(long long)) == 0 &&
         receiver_valid &&
         strcmp([signature getArgumentTypeAtIndex:1], @encode(SEL)) == 0 &&
         strcmp([signature getArgumentTypeAtIndex:2], @encode(long long)) == 0;
}
