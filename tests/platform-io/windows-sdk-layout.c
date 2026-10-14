// Compile against public native SDK headers; no Jai implementation is an input.
#include <stddef.h>
#include <windows.h>

_Static_assert(sizeof(HANDLE) == 8, "64-bit native handles");
_Static_assert(sizeof(GUID) == 16, "GUID ABI");
_Static_assert(sizeof(LARGE_INTEGER) == 8, "64-bit file positions");
_Static_assert(sizeof(OVERLAPPED) == 32, "overlapped I/O ABI");
_Static_assert(offsetof(OVERLAPPED, hEvent) == 24, "overlapped event offset");
_Static_assert(sizeof(STARTUPINFOW) == 104, "startup ABI");
_Static_assert(sizeof(STARTUPINFOEXW) == 112, "extended startup ABI");
_Static_assert(sizeof(PROCESS_INFORMATION) == 24, "process information ABI");
_Static_assert(sizeof(SECURITY_ATTRIBUTES) == 24, "security attributes ABI");
_Static_assert(sizeof(JOBOBJECT_EXTENDED_LIMIT_INFORMATION) == 144, "job limits ABI");
_Static_assert(offsetof(PROCESSOR_RELATIONSHIP, GroupMask) == 24, "processor masks offset");
_Static_assert(sizeof(GROUP_AFFINITY) == 16, "processor group ABI");
_Static_assert(sizeof(CRITICAL_SECTION) == 40, "critical section ABI");
_Static_assert(sizeof(CONDITION_VARIABLE) == 8, "condition variable ABI");

int main(void) { return 0; }
