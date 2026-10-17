// Independent Windows SDK oracle. Build only with real Windows SDK headers.
#include <stddef.h>
#include <stdio.h>
#include <windows.h>

#define LAYOUT(T)                                                              \
  do {                                                                         \
    printf(#T ".size %zu\n", sizeof(T));                                       \
    printf(#T ".align %zu\n", (size_t)_Alignof(T));                            \
  } while (0)
#define MEMBER(T, name)                                                        \
  do {                                                                         \
    printf(#T "." #name ".offset %zu\n", offsetof(T, name));                   \
    printf(#T "." #name ".size %zu\n", sizeof(((T *)0)->name));                \
  } while (0)

int main(void) {
  TIME_ZONE_INFORMATION zone = {0};
  zone.Bias = -720;
  zone.StandardBias = -1;
  zone.DaylightBias = -60;
  LAYOUT(SYSTEMTIME);
  LAYOUT(TIME_ZONE_INFORMATION);
  MEMBER(SYSTEMTIME, wYear);
  MEMBER(SYSTEMTIME, wMonth);
  MEMBER(SYSTEMTIME, wDayOfWeek);
  MEMBER(SYSTEMTIME, wDay);
  MEMBER(SYSTEMTIME, wHour);
  MEMBER(SYSTEMTIME, wMinute);
  MEMBER(SYSTEMTIME, wSecond);
  MEMBER(SYSTEMTIME, wMilliseconds);
  MEMBER(TIME_ZONE_INFORMATION, Bias);
  MEMBER(TIME_ZONE_INFORMATION, StandardName);
  MEMBER(TIME_ZONE_INFORMATION, StandardDate);
  MEMBER(TIME_ZONE_INFORMATION, StandardBias);
  MEMBER(TIME_ZONE_INFORMATION, DaylightName);
  MEMBER(TIME_ZONE_INFORMATION, DaylightDate);
  MEMBER(TIME_ZONE_INFORMATION, DaylightBias);
  printf("Bias.signed %d\nStandardBias.signed %d\nDaylightBias.signed %d\n",
         zone.Bias < 0, zone.StandardBias < 0, zone.DaylightBias < 0);
  printf("StandardName.count %zu\nDaylightName.count %zu\n",
         sizeof(zone.StandardName) / sizeof(zone.StandardName[0]),
         sizeof(zone.DaylightName) / sizeof(zone.DaylightName[0]));
  return 0;
}
