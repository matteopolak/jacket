/* Public SDK layout checks for the reviewed interface prefixes, not a copied
   SDK implementation. No interface is instantiated or invoked here. */
#define CINTERFACE 1
#include <windows.h>
#include <mmreg.h>
#include <dsound.h>
#include <stddef.h>

_Static_assert(sizeof(void *) == 8, "desktop release targets are 64-bit");
_Static_assert(sizeof(WAVEFORMATEX) == 18, "WAVEFORMATEX ABI");
_Static_assert(sizeof(WAVEFORMATEXTENSIBLE) == 40, "extensible format ABI");
_Static_assert(offsetof(WAVEFORMATEXTENSIBLE, dwChannelMask) == 20, "channel mask offset");
_Static_assert(offsetof(WAVEFORMATEXTENSIBLE, SubFormat) == 24, "subformat offset");
_Static_assert(sizeof(DSBUFFERDESC) == 40, "buffer description ABI");
_Static_assert(offsetof(DSBUFFERDESC, lpwfxFormat) == 16, "format pointer offset");
_Static_assert(offsetof(DSBUFFERDESC, guid3DAlgorithm) == 24, "algorithm GUID offset");
_Static_assert(offsetof(IDirectSound8Vtbl, CreateSoundBuffer) == 3 * sizeof(void *), "COM prefix");
_Static_assert(offsetof(IDirectSound8Vtbl, SetCooperativeLevel) == 6 * sizeof(void *), "cooperative level slot");
_Static_assert(offsetof(IDirectSound8Vtbl, GetSpeakerConfig) == 8 * sizeof(void *), "speaker config slot");
_Static_assert(sizeof(IDirectSoundBuffer8Vtbl) == 24 * sizeof(void *), "buffer vtable ABI");
_Static_assert(offsetof(IDirectSoundBuffer8Vtbl, Lock) == 11 * sizeof(void *), "buffer lock slot");
_Static_assert(offsetof(IDirectSoundBuffer8Vtbl, Stop) == 18 * sizeof(void *), "buffer stop slot");
_Static_assert(offsetof(IDirectSoundBuffer8Vtbl, GetObjectInPath) == 23 * sizeof(void *), "buffer final slot");

int main(void) { return 0; }
