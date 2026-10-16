#define GL_SILENCE_DEPRECATION
#import <AppKit/AppKit.h>
#import <objc/message.h>
#import <objc/runtime.h>
#include <stdlib.h>

typedef struct {
  NSRunningApplication *front;
  NSOpenGLContext *context;
} ProbeState;

void *appkit_ui_begin(void) {
  ProbeState *state = calloc(1, sizeof(*state));
  if (!state)
    return NULL;
  state->front = [[[NSWorkspace sharedWorkspace] frontmostApplication] retain];
  state->context = [[NSOpenGLContext currentContext] retain];
  return state;
}
int appkit_ui_finish(void *pointer) {
  ProbeState *state = pointer;
  if (state->context)
    [state->context makeCurrentContext];
  else
    [NSOpenGLContext clearCurrentContext];
  BOOL restored = YES;
  if (state->front && ![state->front isActive])
    restored = [state->front activateWithOptions:0];
  [state->context release];
  [state->front release];
  free(state);
  return restored;
}
void appkit_ui_close(void *window) { [(NSWindow *)window close]; }
void appkit_ui_clear(void) { [NSOpenGLContext clearCurrentContext]; }
int appkit_ui_menu(void *menu) { return [NSApp mainMenu] == (NSMenu *)menu; }
int appkit_ui_context(void *context, void *view) {
  typedef id (*Get)(id, SEL);
  Get get = (Get)objc_msgSend;
  return [NSOpenGLContext currentContext] == context &&
         get((id)context, sel_registerName("view")) == view;
}
