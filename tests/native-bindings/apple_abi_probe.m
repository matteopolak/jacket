// Independent layout oracle compiled against the installed public Apple SDK.
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <OpenGL/OpenGL.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

uint32_t binding_cgl_token(unsigned token) {
    switch (token) {
        case 0: return kCGLPFAOpenGLProfile;
        case 1: return kCGLOGLPVersion_3_2_Core;
        case 2: return kCGLPFAColorSize;
        case 3: return kCGLPFAAlphaSize;
        case 4: return kCGLPFADepthSize;
        case 5: return kCGLPFASamples;
        case 6: return kCGLPFASampleBuffers;
        default: abort();
    }
}

uint64_t binding_apple_size(unsigned record) {
    switch (record) {
        case 0: return sizeof(MTLOrigin);
        case 1: return sizeof(MTLSize);
        case 2: return sizeof(MTLRegion);
        case 3: return sizeof(MTLViewport);
        case 4: return sizeof(MTLClearColor);
        case 5: return sizeof(NSOperatingSystemVersion);
        default: abort();
    }
}

uint64_t binding_apple_offset(unsigned member) {
    switch (member) {
        case 0: return offsetof(MTLOrigin, z);
        case 1: return offsetof(MTLSize, depth);
        case 2: return offsetof(MTLRegion, size);
        case 3: return offsetof(MTLViewport, zfar);
        case 4: return offsetof(MTLClearColor, alpha);
        case 5: return offsetof(NSOperatingSystemVersion, patchVersion);
        default: abort();
    }
}
