/* Original native oracle; __int128 is an official Clang target extension. */
int signed_range_exceeded(unsigned long long value, unsigned bytes) {
  if (bytes == 16) {
    __int128 maximum = (((__int128)1 << 126) - 1) * 2 + 1;
    return (__int128)value > maximum;
  }
  if (bytes != 1 && bytes != 2 && bytes != 4 && bytes != 8) return -1;
  return value > ((1ULL << (bytes * 8 - 1)) - 1);
}
