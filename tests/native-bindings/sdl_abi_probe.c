// Official SDL 2.0.1 headers are the layout oracle; no SDK function is replaced.
#include "SDL.h"
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

uint64_t binding_sdl_size(unsigned which) {
    switch (which) {
        case 0: return sizeof(SDL_version);
        case 1: return sizeof(SDL_Rect);
        case 2: return sizeof(SDL_PixelFormat);
        case 3: return sizeof(SDL_Surface);
        case 4: return sizeof(SDL_Event);
        case 5: return sizeof(SDL_RWops);
        case 6: return sizeof(SDL_UserEvent);
        default: abort();
    }
}

void binding_sdl_user_event(SDL_Event *event, int32_t code, void *data) {
    event->user.type = SDL_USEREVENT;
    event->user.code = code;
    event->user.data1 = data;
}

int32_t binding_sdl_user_matches(const SDL_Event *event, int32_t code, void *data) {
    return event->user.type == SDL_USEREVENT && event->user.code == code && event->user.data1 == data;
}

uint64_t binding_sdl_offset(unsigned which) {
    switch (which) {
        case 0: return offsetof(SDL_PixelFormat, palette);
        case 1: return offsetof(SDL_PixelFormat, refcount);
        case 2: return offsetof(SDL_Surface, pixels);
        case 3: return offsetof(SDL_Surface, clip_rect);
        case 4: return offsetof(SDL_RWops, type);
        default: abort();
    }
}

uint64_t binding_sdl_rw_offset(unsigned which) {
    static const size_t fields[] = {
        offsetof(SDL_RWops, size), offsetof(SDL_RWops, seek),
        offsetof(SDL_RWops, read), offsetof(SDL_RWops, write),
        offsetof(SDL_RWops, close), offsetof(SDL_RWops, type),
        offsetof(SDL_RWops, hidden)
    };
    if (which >= sizeof(fields) / sizeof(fields[0])) abort();
    return fields[which];
}

uint64_t binding_sdl_alignment(unsigned which) {
    switch (which) {
        case 0: return _Alignof(SDL_RWops);
        case 1: return _Alignof(SDL_UserEvent);
        default: abort();
    }
}

uint64_t binding_sdl_user_offset(unsigned which) {
    static const size_t fields[] = {
        offsetof(SDL_UserEvent, type), offsetof(SDL_UserEvent, timestamp),
        offsetof(SDL_UserEvent, windowID), offsetof(SDL_UserEvent, code),
        offsetof(SDL_UserEvent, data1), offsetof(SDL_UserEvent, data2)
    };
    if (which >= sizeof(fields) / sizeof(fields[0])) abort();
    return fields[which];
}

void binding_sdl_fill_native_user(SDL_UserEvent *event, void *data1, void *data2) {
    event->type = SDL_USEREVENT;
    event->timestamp = UINT32_C(0x89abcdef);
    event->windowID = 19;
    event->code = -42;
    event->data1 = data1;
    event->data2 = data2;
}

int32_t binding_sdl_check_size(unsigned which, uint64_t actual) {
    uint64_t expected = binding_sdl_size(which);
    if (actual == expected) return 1;
    if (fprintf(stderr,"SDL record %u: Jai size %llu, native SDK size %llu\n",which,
                (unsigned long long)actual,(unsigned long long)expected) < 0) abort();
    return 0;
}

int binding_sdl_fixture(const char *path) {
    // A 2x1, 24-bit BMP with an eight-byte padded pixel row.
    static const unsigned char bmp[62] = {
        'B','M',62,0,0,0, 0,0,0,0, 54,0,0,0,
        40,0,0,0, 2,0,0,0, 1,0,0,0, 1,0,24,0,
        0,0,0,0, 8,0,0,0, 0,0,0,0, 0,0,0,0,
        0,0,0,0, 0,0,0,0, 0,0,255, 0,255,0, 0,0
    };
    FILE *stream = fopen(path,"wb");
    if (!stream) return -1;
    size_t written = fwrite(bmp,1,sizeof(bmp),stream);
    int closed = fclose(stream);
    return written == sizeof(bmp) && closed == 0 ? 0 : -1;
}
