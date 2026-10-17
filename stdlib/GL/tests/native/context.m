// The reviewed backend is OpenGL. Apple's documented macro opts in to this
// deprecated native API without disabling any other compiler warning.
#define GL_SILENCE_DEPRECATION
#import <AppKit/AppKit.h>
#import <OpenGL/gl3.h>

static _Thread_local int window_thread_marker;
void window_gl_probe_set_thread_marker(int value) { window_thread_marker = value; }
int window_gl_probe_get_thread_marker(void) { return window_thread_marker; }

void *window_gl_probe_begin(void) {
    @autoreleasepool {
        if (![NSThread isMainThread]) return NULL;
        if (!NSApplicationLoad()) return NULL;
        NSOpenGLPixelFormatAttribute attributes[] = {
            NSOpenGLPFAOpenGLProfile, NSOpenGLProfileVersion4_1Core,
            NSOpenGLPFAColorSize, 24, NSOpenGLPFAAlphaSize, 8, 0
        };
        NSOpenGLPixelFormat *format = [[NSOpenGLPixelFormat alloc] initWithAttributes:attributes];
        if (!format) return NULL;
        NSOpenGLContext *context = [[NSOpenGLContext alloc] initWithFormat:format shareContext:nil];
        [format release];
        if (!context) return NULL;
        [context makeCurrentContext];
        if ([NSOpenGLContext currentContext] != context || glGetString(GL_VERSION) == NULL) {
            [NSOpenGLContext clearCurrentContext];
            [context release];
            return NULL;
        }
        return context;
    }
}

void window_gl_probe_end(void *opaque) {
    NSOpenGLContext *context = (NSOpenGLContext *)opaque;
    [NSOpenGLContext clearCurrentContext];
    [context release];
}
