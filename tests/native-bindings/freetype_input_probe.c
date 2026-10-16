// Focused independent SDK oracle for the unchanged FreeType fixture indices.
#include <ft2build.h>
#include FT_FREETYPE_H
#include <stddef.h>
#include <stdint.h>

uint64_t binding_size(int which) {
  switch (which) {
  case 10:
    return sizeof(FT_FaceRec);
  case 11:
    return sizeof(FT_GlyphSlotRec);
  case 12:
    return sizeof(FT_Bitmap);
  case 13:
    return sizeof(FT_Vector);
  default:
    return UINT64_MAX;
  }
}

uint64_t binding_offset(int which) {
  switch (which) {
  case 5:
    return offsetof(FT_FaceRec, glyph);
  case 6:
    return offsetof(FT_GlyphSlotRec, bitmap);
  default:
    return UINT64_MAX;
  }
}
