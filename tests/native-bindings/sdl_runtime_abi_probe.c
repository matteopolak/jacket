// Complete, pinned official SDL2-compatible SDK header is the native oracle.
#include "SDL.h"
#include "SDL_syswm.h"
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

uint64_t binding_runtime_wm_size(void) { return sizeof(SDL_SysWMinfo); }
uint64_t binding_runtime_wm_offset(void) { return offsetof(SDL_SysWMinfo, info); }
uint64_t binding_runtime_wm_union_size(void) { return sizeof(((SDL_SysWMinfo *)0)->info); }
uint64_t binding_runtime_rw_size(void) { return sizeof(SDL_RWops); }
uint64_t binding_runtime_rw_hidden_offset(void) { return offsetof(SDL_RWops, hidden); }

// Observe real SDK-owned file stream cleanup without replacing the loader.
// The sequential test tracks one live file at a time and keeps its full native
// storage untouched. Restore the original callback before SDL frees the file.
static SDL_RWops *tracked;
static int (SDLCALL *original_close)(SDL_RWops *);
static int closes;

static int SDLCALL counted_close(SDL_RWops *source) {
    if (source != tracked || !original_close) abort();
    source->close = original_close;
    tracked = NULL;
    original_close = NULL;
    ++closes;
    return source->close(source);
}

void binding_runtime_track_close(SDL_RWops *source) {
    if (!source || tracked || !source->close) abort();
    tracked = source;
    original_close = source->close;
    source->close = counted_close;
}

int binding_runtime_close_count(void) { return closes; }
