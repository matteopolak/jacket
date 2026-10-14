#import <AppKit/AppKit.h>
#include <stdio.h>
#include <limits.h>
#include "stb_image.h"

// Independent native SDK observation of the Jai backend's PNG clipboard payload.
int window_clipboard_probe(int test_case) {
    @autoreleasepool {
        NSData *data = [[NSPasteboard generalPasteboard] dataForType:NSPasteboardTypePNG];
        if (!data) return 1;
        if (data.length > INT_MAX) return 2;
        int width, height, channels;
        unsigned char *pixels = stbi_load_from_memory(data.bytes, (int)data.length,
                                                      &width, &height, &channels, 4);
        if (!pixels) return 3;
        int result = 0;
        if (width != 2 || height != 2) result = 4;
        // Decode numeric PNG samples independently of Cocoa's premultiplication
        // and device color-space conversion. Caller RGBA is straight alpha.
        else if (test_case == 0 && (pixels[0] != 0 || pixels[1] != 0 || pixels[2] != 128 || pixels[3] != 128 ||
                 pixels[12] != 0 || pixels[13] != 64 || pixels[14] != 0 || pixels[15] != 128)) {
            fprintf(stderr, "native PNG first/last RGBA: %u %u %u %u / %u %u %u %u\n",
                    pixels[0], pixels[1], pixels[2], pixels[3], pixels[12], pixels[13], pixels[14], pixels[15]);
            result = 5;
        }
        else if (test_case == 1 && (pixels[0] != 9 || pixels[1] != 1 || pixels[2] != 2 || pixels[3] != 255 ||
                 pixels[12] != 6 || pixels[13] != 7 || pixels[14] != 8 || pixels[15] != 255)) result = 6;
        stbi_image_free(pixels);
        return result;
    }
}
