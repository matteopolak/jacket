// Complete, pinned official SDL2-compatible SDK header is the native oracle.
#include "SDL.h"
#include "SDL_syswm.h"
#include <stddef.h>
#include <stdint.h>

uint64_t binding_runtime_wm_size(void) { return sizeof(SDL_SysWMinfo); }
uint64_t binding_runtime_wm_offset(void) { return offsetof(SDL_SysWMinfo, info); }
uint64_t binding_runtime_wm_union_size(void) { return sizeof(((SDL_SysWMinfo *)0)->info); }
