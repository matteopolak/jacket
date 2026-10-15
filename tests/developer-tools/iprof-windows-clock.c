#include <stddef.h>
#include <windows.h>
_Static_assert(sizeof(LARGE_INTEGER) == 8, "public counter record width");
_Static_assert(_Alignof(LARGE_INTEGER) == 8, "public counter record alignment");
_Static_assert(offsetof(LARGE_INTEGER, QuadPart) == 0, "public counter overlay");
_Static_assert(sizeof(BOOL) == 4, "public counter status width");
int main(void) {
    LARGE_INTEGER frequency, first, last;
    if (!QueryPerformanceFrequency(&frequency) || frequency.QuadPart <= 0 ||
        !QueryPerformanceCounter(&first)) return 1;
    Sleep(10);
    if (!QueryPerformanceCounter(&last) || last.QuadPart < first.QuadPart) return 2;
    double elapsed = (double)(last.QuadPart - first.QuadPart) / (double)frequency.QuadPart;
    return elapsed >= 0.005 ? 0 : 3;
}
