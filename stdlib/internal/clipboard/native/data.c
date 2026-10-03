// Independently implemented UTF-8 validation and BMP V5 clipboard encoding.
#include "data.h"
#include <limits.h>
#include <stdlib.h>

int pcb_utf8(const unsigned char *text, size_t size) {
  if (!text && size)
    return 0;
  for (size_t i = 0; i < size;) {
    uint32_t value = text[i++], minimum;
    unsigned tail;
    if (value < 128)
      continue;
    if (value >= 0xc2 && value <= 0xdf) {
      value &= 31;
      tail = 1;
      minimum = 128;
    } else if (value >= 0xe0 && value <= 0xef) {
      value &= 15;
      tail = 2;
      minimum = 2048;
    } else if (value >= 0xf0 && value <= 0xf4) {
      value &= 7;
      tail = 3;
      minimum = 65536;
    } else
      return 0;
    if (size - i < tail)
      return 0;
    while (tail--) {
      unsigned char byte = text[i++];
      if ((byte & 0xc0) != 0x80)
        return 0;
      value = (value << 6) | (byte & 63);
    }
    if (value < minimum || value > 0x10ffff ||
        (value >= 0xd800 && value <= 0xdfff))
      return 0;
  }
  return 1;
}

static void le32(unsigned char *out, uint32_t value) {
  for (unsigned i = 0; i < 4; ++i)
    out[i] = (unsigned char)(value >> (8 * i));
}

unsigned char *pcb_bitmap(int32_t width, int32_t height,
                          const unsigned char *pixels, int32_t channels,
                          int flip, size_t *size) {
  *size = 0;
  if (width <= 0 || height <= 0 || !pixels || (channels != 3 && channels != 4))
    return NULL;
  uint64_t extent = (uint64_t)width * (uint64_t)height * 4;
  if (extent > UINT32_MAX - 138 || extent + 138 > SIZE_MAX)
    return NULL;
  unsigned char *bmp = calloc(1, (size_t)extent + 138);
  if (!bmp)
    return NULL;
  bmp[0] = 'B';
  bmp[1] = 'M';
  le32(bmp + 2, (uint32_t)extent + 138);
  le32(bmp + 10, 138); // file header + BITMAPV5HEADER
  le32(bmp + 14, 124);
  le32(bmp + 18, (uint32_t)width);
  le32(bmp + 22, (uint32_t)-height); // top-down native rows
  bmp[26] = 1;
  bmp[28] = 32;
  le32(bmp + 30, 3); // BI_BITFIELDS; explicit alpha mask
  le32(bmp + 34, (uint32_t)extent);
  le32(bmp + 54, 0x00ff0000);
  le32(bmp + 58, 0x0000ff00);
  le32(bmp + 62, 0x000000ff);
  le32(bmp + 66, 0xff000000);
  le32(bmp + 70, 0x73524742); // LCS_sRGB
  le32(bmp + 122, 4);         // LCS_GM_IMAGES
  for (int32_t y = 0; y < height; ++y) {
    size_t source_row = (size_t)(flip ? height - y - 1 : y) * (size_t)width;
    for (int32_t x = 0; x < width; ++x) {
      const unsigned char *src =
          pixels + (source_row + (size_t)x) * (size_t)channels;
      unsigned char *dst =
          bmp + 138 + ((size_t)y * (size_t)width + (size_t)x) * 4;
      dst[0] = src[2];
      dst[1] = src[1];
      dst[2] = src[0];
      dst[3] = channels == 4 ? src[3] : 255;
    }
  }
  *size = (size_t)extent + 138;
  return bmp;
}
