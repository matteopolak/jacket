#include "../../../internal/clipboard/native/data.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
  const unsigned char unicode[] = {0, 'a', 0xc3, 0xa9, 0xf0, 0x9f, 0x8c, 0x8d};
  assert(pcb_utf8(unicode, sizeof unicode));
  assert(pcb_utf8(NULL, 0));
  assert(!pcb_utf8(NULL, 1));
  const unsigned char bad[][4] = {{0xc0, 0x80},
                                  {0xed, 0xa0, 0x80},
                                  {0xf4, 0x90, 0x80, 0x80},
                                  {0x80},
                                  {0xe2, 0x82}};
  const size_t lengths[] = {2, 3, 4, 1, 2};
  for (size_t i = 0; i < 5; ++i)
    assert(!pcb_utf8(bad[i], lengths[i]));
  const unsigned char rgb[] = {1, 2, 3, 4, 5, 6};
  const unsigned char rgba[] = {1, 2, 3, 7, 4, 5, 6, 8};
  size_t size = 99;
  unsigned char *bmp = pcb_bitmap(1, 2, rgb, 3, 0, &size);
  assert(bmp && size == 146 && bmp[0] == 'B' && bmp[1] == 'M');
  assert(bmp[10] == 138 && bmp[14] == 124 && bmp[22] == 254);
  const unsigned char expected[] = {3, 2, 1, 255, 6, 5, 4, 255};
  assert(!memcmp(bmp + 138, expected, sizeof expected));
  free(bmp);
  bmp = pcb_bitmap(1, 2, rgba, 4, 1, &size);
  const unsigned char flipped[] = {6, 5, 4, 8, 3, 2, 1, 7};
  assert(bmp && !memcmp(bmp + 138, flipped, sizeof flipped));
  free(bmp);
  assert(!pcb_bitmap(0, 1, rgb, 3, 0, &size) && size == 0);
  assert(!pcb_bitmap(INT32_MAX, INT32_MAX, rgb, 3, 0, &size));
  assert(!pcb_bitmap(1, 1, rgb, 2, 0, &size));
  return 0;
}
