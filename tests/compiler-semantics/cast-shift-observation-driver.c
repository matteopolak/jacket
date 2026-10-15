// This prints an observation; it does not assert an unreviewed narrow shift width.
#include <inttypes.h>
#include <stdio.h>
#ifdef _WIN32
#include <winsock2.h>
_Static_assert((uint32_t)FIONBIO == UINT32_C(0x8004667e), "SDK control encoding");
#endif

extern uint32_t prefix_control_observation(uint8_t group, uint8_t command);
extern int32_t cast_shift_check(void);

int main(void) {
    if (cast_shift_check() != 0) return 1;
    printf("prefix control observation: %08" PRIx32 "\n",
           prefix_control_observation(0x66, 126));
    return 0;
}
