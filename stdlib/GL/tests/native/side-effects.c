/* Observes a genuine SDK call; returns the native result unchanged. */
#define GL_SILENCE_DEPRECATION
#include <OpenGL/gl3.h>

static unsigned int unmap_count;

GLboolean portable_test_unmap_buffer(GLenum target) {
    unmap_count += 1;
    return glUnmapBuffer(target);
}

unsigned int portable_test_unmap_count(void) { return unmap_count; }
