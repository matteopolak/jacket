/* Independent installed-SDK consumer of pinned public SDL3 and packaged wrappers. */
#include <SDL3/SDL.h>
#include <stdio.h>

#if defined(JAI_CONSUMER_ARM64) && !defined(__aarch64__) && !defined(_M_ARM64)
#error Installed SDL3 ARM64 consumer used a compiler for another architecture
#endif
#if defined(JAI_CONSUMER_X64) && !defined(__x86_64__) && !defined(_M_X64)
#error Installed SDL3 x64 consumer used a compiler for another architecture
#endif
#ifdef __cplusplus
static_assert(sizeof(void *) == 8, "SDL3 packages use 64-bit targets");
extern "C" {
#else
_Static_assert(sizeof(void *) == 8, "SDL3 packages use 64-bit targets");
#endif
extern bool SDLCALL JAI_SDL3_SDL_size_add_check_overflow(size_t a, size_t b, size_t *ret);
#ifdef __cplusplus
}
#endif

int main(void) {
    size_t result = 0;
    if (SDL_GetVersion() != SDL_VERSION || SDL_VERSION != 3004018) {
        fprintf(stderr, "Installed SDL3 header/runtime version mismatch\n"); return 1;
    }
    if (!JAI_SDL3_SDL_size_add_check_overflow(20, 22, &result) || result != 42 ||
        JAI_SDL3_SDL_size_add_check_overflow(SDL_SIZE_MAX, 1, &result)) {
        fprintf(stderr, "Installed SDL3 wrapper linkage/overflow failure\n"); return 2;
    }
    if (!SDL_Init(SDL_INIT_EVENTS)) {
        fprintf(stderr, "Installed SDL3 init: %s\n", SDL_GetError()); return 3;
    }
    SDL_Quit();
    printf("Relocated SDL3 installed-SDK consumer passed\n");
    return 0;
}
