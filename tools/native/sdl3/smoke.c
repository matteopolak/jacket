/* Jai SDL3 package smoke test. Uses only official public SDL3 API. */
#include <SDL3/SDL.h>
#include <stdio.h>
#include JAI_SDL3_LAYOUT_HEADER
#if defined(JAI_SDL3_CPU_ARM64) && !defined(__aarch64__) && !defined(_M_ARM64)
#error SDL3 arm64 build used a compiler for the wrong CPU
#endif
#if defined(JAI_SDL3_CPU_X64) && !defined(__x86_64__) && !defined(_M_X64)
#error SDL3 x64 build used a compiler for the wrong CPU
#endif
_Static_assert(sizeof(void *) == 8, "SDL3 release targets are 64-bit");
/* Exercise the packaged wrappers, not just this TU's public header inlines. */
extern bool SDLCALL JAI_SDL3_SDL_size_mul_check_overflow(size_t a, size_t b, size_t *ret);
extern bool SDLCALL JAI_SDL3_SDL_size_add_check_overflow(size_t a, size_t b, size_t *ret);

static bool check_size_overflow_wrappers(void) {
    size_t value;
    if (!JAI_SDL3_SDL_size_mul_check_overflow(0, SDL_SIZE_MAX, &value) || value != 0) return false;
    if (!JAI_SDL3_SDL_size_mul_check_overflow(SDL_SIZE_MAX, 1, &value) || value != SDL_SIZE_MAX) return false;
    if (!JAI_SDL3_SDL_size_add_check_overflow(SDL_SIZE_MAX, 0, &value) || value != SDL_SIZE_MAX) return false;
    if (!JAI_SDL3_SDL_size_add_check_overflow(20, 22, &value) || value != 42) return false;
    /* SDL only specifies the result output on success; do not assume an output
       value on overflow, which can differ between portable and builtin forms. */
    if (JAI_SDL3_SDL_size_mul_check_overflow(SDL_SIZE_MAX, 2, &value)) return false;
    if (JAI_SDL3_SDL_size_add_check_overflow(SDL_SIZE_MAX, 1, &value)) return false;
    return true;
}
int main(void) {
    if (SDL_GetVersion() != SDL_VERSION) { fprintf(stderr,"SDL version mismatch\n"); return 1; }
    if (!SDL_Init(SDL_INIT_EVENTS)) { fprintf(stderr,"SDL_Init: %s\n",SDL_GetError()); return 2; }
    if (!check_size_overflow_wrappers()) {
        fprintf(stderr,"SDL size overflow wrappers failed\n"); SDL_Quit(); return 4;
    }
    SDL_Event event = {0}, received = {0};
    event.type = SDL_EVENT_USER;
    event.user.code = 71;
    if (!SDL_PushEvent(&event) || !SDL_PollEvent(&received) || received.user.code != 71) {
        fprintf(stderr,"SDL event queue: %s\n",SDL_GetError()); SDL_Quit(); return 3;
    }
    SDL_Quit();
    printf("SDL3 %d static/shared headless initialization passed\n",SDL_GetVersion());
    return 0;
}
