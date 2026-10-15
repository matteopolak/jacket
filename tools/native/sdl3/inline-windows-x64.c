// Generated altered translation. SDL copyright/license: stdlib/Extensions/SDL3/LICENSE.txt
#define SDL_MAIN_HANDLED 1
#define SDL_DISABLE_OLD_NAMES 1
#define SDL_DISABLE_ALLOCA 1
#define SDL_DISABLE_ANALYZE_MACROS 1
#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3/SDL_vulkan.h>
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_size_mul_check_overflow(size_t a, size_t b, size_t * ret) { return SDL_size_mul_check_overflow(a, b, ret); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_size_add_check_overflow(size_t a, size_t b, size_t * ret) { return SDL_size_add_check_overflow(a, b, ret); }
SDL_DECLSPEC float SDLCALL JAI_SDL3_SDL_SwapFloat(float x) { return SDL_SwapFloat(x); }
SDL_DECLSPEC int SDLCALL JAI_SDL3_SDL_MostSignificantBitIndex32(Uint32 x) { return SDL_MostSignificantBitIndex32(x); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_HasExactlyOneBitSet32(Uint32 x) { return SDL_HasExactlyOneBitSet32(x); }
SDL_DECLSPEC void SDLCALL JAI_SDL3_SDL_RectToFRect(const SDL_Rect * rect, SDL_FRect * frect) { SDL_RectToFRect(rect, frect); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_PointInRect(const SDL_Point * p, const SDL_Rect * r) { return SDL_PointInRect(p, r); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_RectEmpty(const SDL_Rect * r) { return SDL_RectEmpty(r); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_RectsEqual(const SDL_Rect * a, const SDL_Rect * b) { return SDL_RectsEqual(a, b); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_PointInRectFloat(const SDL_FPoint * p, const SDL_FRect * r) { return SDL_PointInRectFloat(p, r); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_RectEmptyFloat(const SDL_FRect * r) { return SDL_RectEmptyFloat(r); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_RectsEqualEpsilon(const SDL_FRect * a, const SDL_FRect * b, float epsilon) { return SDL_RectsEqualEpsilon(a, b, epsilon); }
SDL_DECLSPEC bool SDLCALL JAI_SDL3_SDL_RectsEqualFloat(const SDL_FRect * a, const SDL_FRect * b) { return SDL_RectsEqualFloat(a, b); }
